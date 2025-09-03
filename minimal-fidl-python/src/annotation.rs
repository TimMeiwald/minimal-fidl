use std::collections::HashMap;

use minimal_fidl_collect::Annotation;
use pyo3::prelude::*;

use crate::diff::FidlDiff;

#[pyclass(name = "FidlAnnotation", frozen)]
#[derive(Clone, Debug)]
pub struct FidlAnnotation {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub contents: String,
}

impl FidlAnnotation{
    pub fn diff_fidl_annotation_list(annotations: &Vec<FidlAnnotation>, other_annotations: &Vec<FidlAnnotation>) -> FidlDiff {
        todo!("Do the diff for two lists of fidl annotations.");

    }
}

#[pymethods]
impl FidlAnnotation {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }

    fn diff(&self, other: &Self) -> FidlDiff {
        if self.name != other.name {
            FidlDiff::MAJOR
        } else if self.contents != other.contents {
            // Whether contents change matters can often depend on the contents.
            // Add exceptions here
            if self.name == "details" {
                FidlDiff::MAJOR
            } else {
                FidlDiff::MINOR
            }
        } else {
            FidlDiff::IDENTICAL
        }
    }
}
impl From<&Annotation> for FidlAnnotation {
    fn from(item: &Annotation) -> Self {
        FidlAnnotation {
            name: item.name.clone(),
            contents: item.contents.clone(),
        }
    }
}
