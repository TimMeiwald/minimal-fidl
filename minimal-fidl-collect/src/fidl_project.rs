use minimal_fidl_parser::{
    BasicContext, BasicPublisher, Context, Rules, Source, _var_name, grammar, Key, RULES_SIZE,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::fidl_file::{FidlFile, FileError};
use crate::io::FileLoadError;

/// The result of a directory walk: every `.fidl` path found, plus the
/// directories that could not be read along the way.
///
/// A subdirectory that cannot be read is a reportable problem, not a reason to
/// abandon the walk — the caller still gets every file that was reachable.
#[derive(Debug, Default)]
pub struct Walk {
    /// Sorted, so a project loads in the same order on every platform.
    pub paths: Vec<PathBuf>,
    pub errors: Vec<FileLoadError>,
}

impl Walk {
    fn descend(&mut self, dir: &Path) {
        match std::fs::read_dir(dir) {
            Ok(entries) => self.visit(dir, entries),
            Err(err) => self.errors.push(FileLoadError::unreadable(dir, err)),
        }
    }

    fn visit(&mut self, dir: &Path, entries: std::fs::ReadDir) {
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                // The entry's own name is what failed to read, so the
                // directory is the most specific path we can name.
                Err(err) => {
                    self.errors.push(FileLoadError::unreadable(dir, err));
                    continue;
                }
            };
            let path = entry.path();
            if path.is_dir() {
                self.descend(&path);
            } else if FidlProject::is_fidl_file(&path) {
                self.paths.push(path);
            }
        }
    }
}

#[derive(Debug)]
pub struct FidlProject {}
impl FidlProject {
    /// Every `.fidl` file under `dir`, recursively.
    ///
    /// Errors only if `dir` itself cannot be read — missing, not a directory,
    /// or permission denied. Unreadable *sub*directories land in
    /// [`Walk::errors`].
    pub fn walk(dir: impl Into<PathBuf>) -> Result<Walk, std::io::Error> {
        let root = dir.into();
        // `read_dir` up front rather than `root.is_dir()`: `is_dir()` cannot
        // tell a missing path or a plain file from an empty directory, so the
        // old walk returned `Ok(vec![])` for all three.
        let entries = std::fs::read_dir(&root)?;

        let mut walk = Walk::default();
        walk.visit(&root, entries);
        walk.paths.sort();
        Ok(walk)
    }

    pub fn generate_file_from_string(src: String) -> Result<FidlFile, FileError> {
        let publisher = Self::parse(&src);
        let publisher: BasicPublisher = match publisher {
            None => return Err(FileError::CouldNotParseSourceString(src)),
            Some(res) => res,
        };
        Ok(FidlFile::new(src, &publisher)?)
    }

    pub fn generate_file(path: impl Into<PathBuf>) -> Result<FidlFile, FileError> {
        let path = path.into();
        let src = std::fs::read_to_string(&path);
        let src: String = match src {
            Err(err) => return Err(FileError::CouldNotReadFile(err)),
            Ok(src) => src,
        };
        let publisher = Self::parse(&src);
        let publisher: BasicPublisher = match publisher {
            None => return Err(FileError::CouldNotParseFile(path.clone())),
            Some(res) => res,
        };
        let mut file = FidlFile::new(src, &publisher)?;
        file.path = Some(path);
        Ok(file)
    }

    fn parse(input: &str) -> Option<BasicPublisher> {
        let string = input.to_string();
        let src_len = string.len() as u32;
        let source = Source::new(&string);
        let position: u32 = 0;
        let result: (bool, u32);
        let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
        {
            let executor = _var_name(Rules::Grammar, &context, grammar);
            result = executor(Key(0), &source, position);
        }
        if result != (true, src_len) {
            println!("Failed with : {:?}", result);
            return None;
        }
        let publisher = context.into_inner().get_publisher().clear_false();
        Some(publisher)
    }

    fn is_fidl_file(path: &Path) -> bool {
        let extension = path.extension();
        match extension {
            Some(extension) => extension == "fidl",
            None => false,
        }
    }
}
