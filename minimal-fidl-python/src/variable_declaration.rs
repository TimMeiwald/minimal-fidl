use std::collections::{BTreeMap, HashSet};

use crate::{annotation::FidlAnnotation, diff::FidlDiff};
use minimal_fidl_collect::{VariableDeclaration};
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
impl FidlVariableDeclaration {
    pub fn diff_list(
        variable_declaration: &Vec<FidlVariableDeclaration>,
        other_variable_declarations: &Vec<FidlVariableDeclaration>,
    ) -> FidlDiff {
        let map: BTreeMap<_, _> = variable_declaration
            .into_iter()
            .map(|data| (data.name.clone(), data))
            .collect();

        let other_map: BTreeMap<_, _> = other_variable_declarations
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
                // If the left hand side is None, then it means we added a variable declaration to a struct/method.
                // Since this changes the size or function signature it's a major.
                diff = FidlDiff::MAJOR;
            } else if o.is_none() {
                // If the right hand side is None, then it means we removed a variable declaration from a struct/method
                // Which is a major change
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
impl FidlVariableDeclaration {
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
