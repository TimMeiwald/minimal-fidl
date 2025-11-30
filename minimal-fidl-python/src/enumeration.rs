use minimal_fidl_collect::Enumeration;
use pyo3::prelude::*;

use crate::{annotation::FidlAnnotation, enum_value::FidlEnumValue};

#[pyclass(name = "FidlEnumeration", frozen)]
#[derive(Clone, Debug)]
pub struct FidlEnumeration {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub values: Vec<FidlEnumValue>,
}
#[pymethods]
impl FidlEnumeration {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&Enumeration> for FidlEnumeration {
    fn from(item: &Enumeration) -> Self {
        FidlEnumeration {
            annotations: item
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect(),
            name: item.name.clone(),
            values: item.values.iter().map(|a| FidlEnumValue::from(a)).collect(),
        }
    }
}
