use minimal_fidl_collect::Structure;
use pyo3::prelude::*;

use crate::{annotation::FidlAnnotation, variable_declaration::FidlVariableDeclaration};

#[pyclass(name = "FidlStructure", frozen)]
#[derive(Clone, Debug)]
pub struct FidlStructure {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub contents: Vec<FidlVariableDeclaration>,
}
#[pymethods]
impl FidlStructure {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&Structure> for FidlStructure {
    fn from(item: &Structure) -> Self {
        FidlStructure {
            annotations: item
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect(),
            name: item.name.clone(),
            contents: item
                .contents
                .iter()
                .map(|a| FidlVariableDeclaration::from(a))
                .collect(),
        }
    }
}
