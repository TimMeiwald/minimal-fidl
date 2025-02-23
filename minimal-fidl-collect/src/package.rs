use crate::fidl_file::FileError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Package {
    path: Vec<String>,
}
impl Package {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::package);
        let mut path: Result<Vec<String>, FileError> = Err(
            FileError::InternalLogicError("Uninitialized value in Package::new".to_string()),
        );
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {},
                Rules::type_ref => {
                    let res: String = child.get_string(source);
                    path = Ok(res.split(".").map(|string| {string.to_string()}).collect())
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Package::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { path: path? })
    }
    pub fn push_if_not_exists_else_err(self, package: &mut Option<Package>) -> Result<(), FileError>{
        // Set would be a more appropriate name but push is more consistent naming.
        // TODO: This never happens because the parser does not allow more than one version.
        // However, the error messages in that case would be far less useful so it might be worth modifying the
        // grammar to allow more than one version so we can print an appropriate error instead. 
        match package{
            None => {
                *package = Some(self);
                Ok(())

            }
            Some(package) => {
                Err(FileError::PackageAlreadyExists(package.clone()))

            }
        }
    }
}
