use minimal_fidl_collect::TypeCollection;
use pyo3::prelude::*;

use crate::{
    annotation::FidlAnnotation, enumeration::FidlEnumeration, structure::FidlStructure,
    type_def::FidlTypeDef, version::FidlVersion,
};

#[pyclass(name = "FidlTypeCollection", frozen)]
#[derive(Clone, Debug)]
pub struct FidlTypeCollection {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub version: Option<FidlVersion>,
    #[pyo3(get)]
    pub typedefs: Vec<FidlTypeDef>,
    #[pyo3(get)]
    pub structures: Vec<FidlStructure>,
    #[pyo3(get)]
    pub enumerations: Vec<FidlEnumeration>,
}
#[pymethods]
impl FidlTypeCollection {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&TypeCollection> for FidlTypeCollection {
    fn from(iface: &TypeCollection) -> Self {
        let version = match &iface.version {
            None => None,
            Some(version) => Some(FidlVersion::from(version)),
        };
        let annotations = iface
            .annotations
            .iter()
            .map(|a| FidlAnnotation::from(a))
            .collect();
        FidlTypeCollection {
            name: iface.name.clone(),
            version,
            annotations,
            structures: iface
                .structures
                .iter()
                .map(|a| FidlStructure::from(a))
                .collect(),
            typedefs: iface
                .typedefs
                .iter()
                .map(|a| FidlTypeDef::from(a))
                .collect(),
            enumerations: iface
                .enumerations
                .iter()
                .map(|a| FidlEnumeration::from(a))
                .collect(),
        }
    }
}
