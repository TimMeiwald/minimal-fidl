use minimal_fidl_collect::EnumValue;
use pyo3::prelude::*;

use crate::annotation::FidlAnnotation;

#[pyclass(name = "FidlEnumValue", frozen)]
#[derive(Clone, Debug)]
pub struct FidlEnumValue {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub value: Option<u64>,
}
#[pymethods]
impl FidlEnumValue {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&EnumValue> for FidlEnumValue {
    fn from(item: &EnumValue) -> Self {
        FidlEnumValue {
            annotations: item
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect(),
            name: item.name.clone(),
            value: item.value,
        }
    }
}
