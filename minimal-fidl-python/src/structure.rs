use minimal_fidl_collect::Structure;
use pyo3::prelude::*;

use crate::{annotation::FidlAnnotation, diff::FidlDiff, variable_declaration::FidlVariableDeclaration};

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
    fn diff(&self, other: &Self) -> FidlDiff {
        if self.name != other.name {
            FidlDiff::MAJOR
        } 
        FidlVariableDeclaration::diff_fidl_variable_declaration_list(&self.contents, &other.contents)
        FidlAnnotation::diff_fidl_annotation_list(&self.annotations, &other.annotations)
    
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
