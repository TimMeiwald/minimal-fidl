use minimal_fidl_collect::Interface;
use pyo3::prelude::*;

use crate::{
    annotation::FidlAnnotation, attribute::FidlAttribute, enumeration::FidlEnumeration,
    method::FidlMethod, structure::FidlStructure, type_def::FidlTypeDef, version::FidlVersion,
};

#[pyclass(name = "FidlInterface", frozen)]
#[derive(Clone, Debug)]
pub struct FidlInterface {
    #[pyo3(get)]
    pub annotations: Vec<FidlAnnotation>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub version: Option<FidlVersion>,
    #[pyo3(get)]
    pub attributes: Vec<FidlAttribute>,
    #[pyo3(get)]
    pub structures: Vec<FidlStructure>,
    #[pyo3(get)]
    pub typedefs: Vec<FidlTypeDef>,
    #[pyo3(get)]
    pub methods: Vec<FidlMethod>,
    #[pyo3(get)]
    pub enumerations: Vec<FidlEnumeration>,
}
#[pymethods]
impl FidlInterface {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&Interface> for FidlInterface {
    fn from(iface: &Interface) -> Self {
        let version = match &iface.version {
            None => None,
            Some(version) => Some(FidlVersion::from(version)),
        };
        let annotations = iface
            .annotations
            .iter()
            .map(|a| FidlAnnotation::from(a))
            .collect();
        FidlInterface {
            name: iface.name.clone(),
            version,
            annotations,
            attributes: iface
                .attributes
                .iter()
                .map(|a| FidlAttribute::from(a))
                .collect(),
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
            methods: iface.methods.iter().map(|a| FidlMethod::from(a)).collect(),
            enumerations: iface
                .enumerations
                .iter()
                .map(|a| FidlEnumeration::from(a))
                .collect(),
        }
    }
}
