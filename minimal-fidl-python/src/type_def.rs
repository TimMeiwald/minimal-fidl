use minimal_fidl_collect::TypeDef;
use pyo3::prelude::*;

use crate::annotation::FidlAnnotation;

#[pyclass(name = "FidlTypeDef", frozen)]
#[derive(Clone, Debug)]
pub struct FidlTypeDef {
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
impl FidlTypeDef {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&TypeDef> for FidlTypeDef {
    fn from(item: &TypeDef) -> Self {
        FidlTypeDef {
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
