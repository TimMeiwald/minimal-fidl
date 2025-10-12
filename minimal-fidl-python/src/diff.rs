use std::ops::Deref;

use pyo3::prelude::*;

/// This enum represents whether a diffed result
/// has a significant, less significant, minor or no change
#[pyclass(name = "FidlDiff", frozen)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum FidlDiff {
    IDENTICAL = 1, // The number order is used by partial ord for semantic ordering
    PATCH = 2,     // I.e MAJOR > MINOR etc. 
    MINOR = 3,
    MAJOR = 4,
}
#[pymethods]
impl FidlDiff {
    fn __int__(&self) -> i32 {
        match self {
            Self::MAJOR => 4,
            Self::MINOR => 3,
            Self::PATCH => 2,
            Self::IDENTICAL => 1,
        }
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.__int__() == other.__int__()
    }
}

#[test]
fn test_fidl_diff() {
    assert!(FidlDiff::MAJOR > FidlDiff::IDENTICAL)
}
#[test]
fn test_fidl_diff2() {
    assert!(FidlDiff::MINOR < FidlDiff::MAJOR)
}
