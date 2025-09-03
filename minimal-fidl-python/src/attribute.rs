use minimal_fidl_collect::Attribute;
use pyo3::prelude::*;

use crate::{annotation::FidlAnnotation, diff::FidlDiff};

#[pyclass(name = "FidlAttribute", frozen)]
#[derive(Clone, Debug)]
pub struct FidlAttribute {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub type_name: String,
}
#[pymethods]
impl FidlAttribute {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
    fn diff(&self, other: &Self) -> FidlDiff {
        if self.name != other.name || self.type_name != other.type_name {
            FidlDiff::MAJOR
        } else {
            FidlAnnotation::diff_fidl_annotation_list(&self.annotations, &other.annotations)
        }
    }
}
impl From<&Attribute> for FidlAttribute {
    fn from(item: &Attribute) -> Self {
        FidlAttribute {
            annotations: item
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect(),
            name: item.name.clone(),
            type_name: item.type_n.clone(),
        }
    }
}
