use std::collections::{BTreeMap, HashSet};

use minimal_fidl_collect::Method;
use pyo3::prelude::*;

use crate::{
    annotation::FidlAnnotation, diff::FidlDiff, variable_declaration::FidlVariableDeclaration,
};

#[pyclass(name = "FidlMethod", frozen)]
#[derive(Clone, Debug)]
pub struct FidlMethod {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub input_parameters: Vec<FidlVariableDeclaration>,
    #[pyo3(get)]
    pub output_parameters: Vec<FidlVariableDeclaration>,
}
impl FidlMethod {
    pub fn diff_list(methods: &Vec<FidlMethod>, other_methods: &Vec<FidlMethod>) -> FidlDiff {
        let map: BTreeMap<_, _> = methods
            .into_iter()
            .map(|data| (data.name.clone(), data))
            .collect();

        let other_map: BTreeMap<_, _> = other_methods
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
                // If the left hand side is None, then it means we added a method to a interface/type colection
                // Since this doesn't change any existing code it's minor
                diff = FidlDiff::MINOR;
            } else if o.is_none() {
                // If the right hand side is None, then it means we removed a method from a interface/type collection
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
impl FidlMethod {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
    fn diff(&self, other: &Self) -> FidlDiff {
        if self.name != other.name {
            FidlDiff::MAJOR
        } else {
            let mut diff = FidlDiff::IDENTICAL;
            let result = FidlAnnotation::diff_list(&self.annotations, &other.annotations);
            if result > diff {
                diff = result;
            }
            let result =
                FidlVariableDeclaration::diff_list(&self.input_parameters, &other.input_parameters);
            if result > diff {
                diff = result;
            }
            let result = FidlVariableDeclaration::diff_list(
                &self.output_parameters,
                &other.output_parameters,
            );
            if result > diff {
                diff = result;
            }
            diff
        }
    }
}
impl From<&Method> for FidlMethod {
    fn from(item: &Method) -> Self {
        FidlMethod {
            annotations: item
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect(),
            name: item.name.clone(),
            input_parameters: item
                .input_parameters
                .iter()
                .map(|a| FidlVariableDeclaration::from(a))
                .collect(),
            output_parameters: item
                .output_parameters
                .iter()
                .map(|a| FidlVariableDeclaration::from(a))
                .collect(),
        }
    }
}
