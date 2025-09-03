use minimal_fidl_collect::Method;
use pyo3::prelude::*;

use crate::{annotation::FidlAnnotation, variable_declaration::FidlVariableDeclaration};

#[pyclass(name = "FidlMethod", frozen)]
#[derive(Clone, Debug)]
pub struct FidlMethod {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub input_parameters: Vec<FidlVariableDeclaration>,
    #[pyo3(get)]
    pub output_parameters: Vec<FidlVariableDeclaration>,
}
#[pymethods]
impl FidlMethod {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&Method> for FidlMethod {
    fn from(item: &Method) -> Self {
        FidlMethod {
            annotations: item
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect(),
            name: item.name.clone(),
            input_parameters: item
                .input_parameters
                .iter()
                .map(|a| FidlVariableDeclaration::from(a))
                .collect(),
            output_parameters: item
                .output_parameters
                .iter()
                .map(|a| FidlVariableDeclaration::from(a))
                .collect(),
        }
    }
}
