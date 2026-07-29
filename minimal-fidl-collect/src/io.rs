//! Reading files in and writing them back out. See `DESIGN.md` §10.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::{print::Mode, FidlFile, FidlProject, FileError};

impl FidlFile {
    /// Parse a `.fidl` file from disk, remembering where it came from so
    /// [`Self::save`] can write it back.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, FileError> {
        FidlProject::generate_file(path.as_ref().to_path_buf())
    }

    /// Parse from a source string. The file has no path until one is set.
    pub fn from_source(source: &str) -> Result<Self, FileError> {
        FidlProject::generate_file_from_string(source.to_string())
    }

    /// Write the formatted file to `path`. Does not change the remembered path.
    pub fn write_to(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        std::fs::write(path, self.to_fidl())
    }

    /// Write the file back where it was read from.
    ///
    /// Errors if the file was built from a string and has no path — use
    /// [`Self::write_to`], or set `path` first.
    pub fn save(&self) -> std::io::Result<()> {
        let path = self.path.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "this FidlFile has no path; use write_to()",
            )
        })?;
        self.write_to(path)
    }

    /// Write back preserving untouched subtrees byte-for-byte, so an edit shows
    /// up as a minimal diff. See `DESIGN.md` §8.
    pub fn save_preserving(&self) -> std::io::Result<()> {
        let path = self.path.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "this FidlFile has no path; use write_to()",
            )
        })?;
        std::fs::write(path, self.to_fidl_with(Mode::Preserve))
    }
}

impl FromStr for FidlFile {
    type Err = FileError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::from_source(source)
    }
}

/// One thing under the project directory that could not be loaded, and why.
///
/// The path is attached the same way [`Project::validate`] attaches one to each
/// diagnostic, so both halves of a load report read alike.
#[derive(Debug)]
pub struct FileLoadError {
    pub path: PathBuf,
    pub error: FileError,
}

impl FileLoadError {
    /// A path that could not be read at all, as opposed to one that read but
    /// would not parse.
    pub(crate) fn unreadable(path: impl Into<PathBuf>, error: std::io::Error) -> Self {
        Self {
            path: path.into(),
            error: FileError::CouldNotReadFile(error),
        }
    }
}

impl std::fmt::Display for FileLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.error)
    }
}

impl std::error::Error for FileLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Every `.fidl` file under a directory, parsed, plus the ones that would not.
#[derive(Debug)]
pub struct Project {
    pub files: Vec<FidlFile>,
    pub errors: Vec<FileLoadError>,
}

impl Project {
    /// Parse every `.fidl` file under `dir`.
    ///
    /// Errors only if `dir` itself cannot be read — missing, not a directory, or
    /// permission denied. Once the directory is open the load always succeeds:
    /// files that fail to read, parse, or build a tree land in [`Self::errors`]
    /// and the rest are returned in [`Self::files`]. One malformed file must not
    /// hide either the other failures or the files that were fine.
    pub fn load(dir: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        let walk = FidlProject::walk(dir)?;
        let mut files = Vec::with_capacity(walk.paths.len());
        let mut errors = walk.errors;
        for path in walk.paths {
            match FidlFile::from_path(&path) {
                Ok(file) => files.push(file),
                Err(error) => errors.push(FileLoadError { path, error }),
            }
        }
        Ok(Self { files, errors })
    }

    /// True if any file under the directory failed to load.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn write_all(&self) -> std::io::Result<()> {
        for file in &self.files {
            file.save()?;
        }
        Ok(())
    }

    /// Every diagnostic across every file, paired with the file it came from.
    pub fn validate(&self) -> Vec<(Option<PathBuf>, crate::Diagnostic)> {
        self.files
            .iter()
            .flat_map(|file| {
                file.validate()
                    .into_iter()
                    .map(move |d| (file.path.clone(), d))
            })
            .collect()
    }

    pub fn file(&self, path: impl AsRef<Path>) -> Option<&FidlFile> {
        let path = path.as_ref();
        self.files.iter().find(|f| f.path.as_deref() == Some(path))
    }

    pub fn file_mut(&mut self, path: impl AsRef<Path>) -> Option<&mut FidlFile> {
        let path = path.as_ref();
        self.files
            .iter_mut()
            .find(|f| f.path.as_deref() == Some(path))
    }
}
