use std::collections::{BTreeMap, HashSet};

use minimal_fidl_collect::TypeDef;
use pyo3::prelude::*;

use crate::{annotation::FidlAnnotation, diff::FidlDiff};

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
impl FidlTypeDef{
    pub fn diff_list(
        fidl_type_defs: &Vec<FidlTypeDef>,
        other_fidl_type_defs: &Vec<FidlTypeDef>,
    ) -> FidlDiff {
        let map: BTreeMap<_, _> = fidl_type_defs
            .into_iter()
            .map(|data| (data.name.clone(), data))
            .collect();

        let other_map: BTreeMap<_, _> = other_fidl_type_defs
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
            let diff: FidlDiff;
            if s.is_none() {
                // If the left hand side is None, then it means we added a type definition to an interface/type collection
                diff = FidlDiff::MINOR;
            } else if o.is_none() {
                // If the right hand side is None, then it means we removed a variable declaration type definition to an interface/type collection
                diff = FidlDiff::MAJOR;
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
impl FidlTypeDef {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
    fn diff(&self, other: &Self) -> FidlDiff {
        if (self.name != other.name)
            || (self.type_name != other.type_name)
            || (self.is_array != other.is_array)
        {   
            // Different name, different type or now an array or vice versa are all major changes.
            FidlDiff::MAJOR
        } else {
            FidlAnnotation::diff_list(&self.annotations, &other.annotations)
        }
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
