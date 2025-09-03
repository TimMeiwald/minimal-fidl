use crate::annotation::FidlAnnotation;
use minimal_fidl_collect::VariableDeclaration;
use pyo3::prelude::*;

#[pyclass(name = "FidlVariableDeclaration", frozen)]
#[derive(Clone, Debug)]
pub struct FidlVariableDeclaration {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub type_name: String,
    #[pyo3(get)]
    pub is_array: bool,
}
#[pymethods]
impl FidlVariableDeclaration {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&VariableDeclaration> for FidlVariableDeclaration {
    fn from(item: &VariableDeclaration) -> Self {
        FidlVariableDeclaration {
            annotations: item
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect(),
            name: item.name.clone(),
            type_name: item.type_n.clone(),
            is_array: item.is_array,
        }
    }
}
