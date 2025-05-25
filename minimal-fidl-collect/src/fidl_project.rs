use minimal_fidl_parser::{
    BasicContext, BasicPublisher, Context, Rules, Source, _var_name, grammar, Key, RULES_SIZE,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::fidl_file::{FidlFile, FileError};

#[derive(Debug)]
pub struct FidlProject {}
impl FidlProject {
    pub fn new(dir: impl Into<PathBuf>) -> Result<Vec<PathBuf>, std::io::Error> {
        let files = Self::walk_dirs(&dir.into());
        Ok(files?)
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
        Ok(FidlFile::new(src, &publisher)?)
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

    fn walk_dirs(path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut ret_vec: Vec<PathBuf> = Vec::new();
        if path.is_dir() {
            for path in std::fs::read_dir(path)? {
                let path = path?;
                let path = path.path();
                if path.is_dir() {
                    let paths: Vec<PathBuf> = Self::walk_dirs(&path)?;
                    ret_vec.extend(paths);
                } else {
                    ret_vec.push(path);
                }
            }
        }
        let ret_vec: Vec<PathBuf> = ret_vec
            .iter()
            .filter(|path| Self::is_fidl_file(path))
            .map(|path| path.to_path_buf())
            .collect();
        Ok(ret_vec)
    }
}
