use minimal_fidl_collect::EnumValue;
use pyo3::prelude::*;

use crate::{annotation::FidlAnnotation, diff::FidlDiff};

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
    fn diff(&self, other: &Self) -> FidlDiff {
        if self.name != other.name || self.value != other.value {
            FidlDiff::MAJOR
        } else {
            FidlAnnotation::diff_fidl_annotation_list(&self.annotations, &other.annotations)
        }
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
