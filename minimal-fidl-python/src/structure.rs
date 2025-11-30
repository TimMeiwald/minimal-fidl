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
            return FidlDiff::MAJOR
        } 
        let diff_var_decl = FidlVariableDeclaration::diff_list(&self.contents, &other.contents);
        let diff_fidl_annotation = FidlAnnotation::diff_list(&self.annotations, &other.annotations);
        if diff_var_decl > diff_fidl_annotation{
            diff_var_decl
        }
        else{
            diff_fidl_annotation
        }
    
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
