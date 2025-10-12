use std::collections::{BTreeMap, HashSet};

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

impl FidlAnnotation {
    pub fn diff_list(
        annotations: &Vec<FidlAnnotation>,
        other_annotations: &Vec<FidlAnnotation>,
    ) -> FidlDiff {
        let map: BTreeMap<_, _> = annotations
            .into_iter()
            .map(|data| (data.name.clone(), data))
            .collect();

        let other_map: BTreeMap<_, _> = other_annotations
            .into_iter()
            .map(|data| (data.name.clone(), data))
            .collect();

        // Get unique set of keys
        let mut all_keys: HashSet<String> = map.iter().map(|(key, _)| key.clone()).collect();
        all_keys.extend(other_map.iter().map(|(key, _)| key.clone()));

        let mut result: FidlDiff = FidlDiff::IDENTICAL;
        for key in all_keys {
            let s = map.get(&key);
            let o = map.get(&key);
            let mut diff: FidlDiff = FidlDiff::IDENTICAL;
            if s.is_none() {
                // If the left hand side is None, then it means we added an annotation
                // Which is a minor change
                diff = FidlDiff::MINOR;
            } else if o.is_none() {
                // If the right hand side is None, then it means we removed an annotation
                // Which is a major change(since things can depend on annotations.)
                if s.expect("Should already be checked above").name == "details" {
                    // Details field is used in generation code so major change
                    diff = FidlDiff::MAJOR;
                } else {
                    // For now other fields are considered minor as they're descriptive. This may change.
                    diff = FidlDiff::MINOR;
                }
            } else {
                let s = s.expect("Should already be checked above");
                let o = o.expect("Should already be checked above");
                diff = s.diff(o);
            }
            if diff > result {
                result = diff
            }
        }
        return result;
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
