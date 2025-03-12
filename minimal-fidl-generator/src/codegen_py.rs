use std::collections::HashMap;
use std::hash::Hash;
use std::{fmt::format, path::PathBuf};

use crate::codegen_trait::{CodeGenerator, GeneratorError};
use crate::indented_string::IndentedString;
use crate::FidlType;
use minimal_fidl_collect::{fidl_file, FidlProject};
use minimal_fidl_collect::{
    attribute::{self, Attribute},
    enumeration::Enumeration,
    fidl_file::FidlFile,
    interface::Interface,
    method::Method,
    structure::Structure,
    type_collection::{self, TypeCollection},
    type_def::TypeDef,
    variable_declaration::VariableDeclaration,
    version::Version,
};


pub struct PythonCodeGen {
    // Generate file creates a vector of strings because one fidl file can generate multiple source code files
    // in languages where a module is a file. E.g Python
    pub python_code: HashMap<PathBuf, Vec<String>>,
}
impl std::fmt::Debug for PythonCodeGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (path, vec) in &self.python_code{
            write!(f, "\n\n{:?}\n", path)?;
            for src in vec{
                write!(f, "{}", src)?;

            }
        }
        Ok(())
    }
}
impl CodeGenerator for PythonCodeGen {
    fn new() -> Self {
        Self {
            python_code: HashMap::new(),
        }
    }

    fn generate_file(&mut self, path: PathBuf, fidl: FidlFile) -> Result<(), GeneratorError> {
        let file = self.file(&fidl);
        let mut str: String = "".to_string();
        for line in file {
            str += &line.to_string();
        }
        let mut vec: Vec<String> = Vec::new();
        vec.push(str);
        self.python_code.insert(path, vec);
        Ok(())
    }
    fn generate_project(
        &mut self,
        dir: PathBuf,
    ) -> Result<(), GeneratorError> {
        let paths = FidlProject::new(dir);
        for path in paths.unwrap() {
            let fidl = match FidlProject::generate_file(path.clone()){
                Ok(fidl_file) => fidl_file,
                Err(err) => return Err(GeneratorError::FidlFileError(err))
            };

            // This needs to be modified because I want to get each interface and type collection as a 
            // seperate file.  
            // But it's not part of the trait anymore so that's fine. 
            // We then also create a primitive types at root level that the rest can import 
            // We'll need JSON and Little Endian serde for debug and to send to comms
            // Then each method will need to accept the inputs, serialize them
            // Send through some function 
            // Deserialize the returned value. Async/Sync as options. Maybe only async since we can always force sync using async
            // Also need to add annotation block details support. 
            self.generate_file(path, fidl).unwrap();
        }
        Ok(())

    }
}

impl PythonCodeGen {
    fn built_in_types(&self) -> Vec<IndentedString> {
        r#"UInt8 unsigned 8-bit integer (range 0..255)
        Int 8signed 8-bit integer (range -128..127)
        UInt16 unsigned 16-bit integer (range 0..65535)
        Int16 signed 16-bit integer (range -32768..32767)
        UInt32 unsigned 32-bit integer (range 0..4294967295)
        Int32 signed 32-bit integer (range -2147483648..2147483647)
        UInt64 unsigned 64-bit integer
        Int64 signed 64-bit integer
        Integer generic integer (with optional range definition, see below)
        Boolean boolean value, which can take one of two values: false or true.
        Float floating point number (4 bytes, range +/- 3.4e +/- 38, 7 digits)
        Double double precision floating point number (8 bytes, range +/- 1.7e +/- 308,
        15 digits)
        String character string, see caveat below
        ByteBuffer buffer of bytes (aka BLOB), see caveat below"#;
        let mut res: Vec<IndentedString> = Vec::new();

        res.push(IndentedString::new(
            0,
            FidlType::File,
            "pub mod Primitives {".to_string(),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use u8 as UInt8;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use i8 as Int8;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use u16 as UInt16;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use i16 as Int16;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use u32 as UInt32;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use i32 as Int32;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use u64 as UInt64;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use i64 as Int64;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use f32 as Float;"),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::File,
            format!("pub use f64 as Double;"),
        ));
        res.push(IndentedString::new(0, FidlType::File, "}".to_string()));

        res
    }

    fn context_trait(&self) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();

        let module = IndentedString::new(0, FidlType::File, format!("pub trait FidlContext {{",));
        res.push(module);

        let module = IndentedString::new(0, FidlType::File, format!("}}",));
        res.push(module);
        res
    }

    fn project(&self, dir: &PathBuf) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        res.push(IndentedString::new(
            0,
            FidlType::File,
            "use serde::{Serialize, Deserialize};".to_string(),
        ));
        res.push(IndentedString::new(
            0,
            FidlType::File,
            "use binary_serde::{binary_serde_bitfield, BinarySerde, Endianness};".to_string(),
        ));

        res.extend(self.built_in_types());
        res.extend(self.context_trait());

        res
    }

    fn file(&self, file: &FidlFile) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();

        // Below is temporary, file should really be called by and from project not this way around.
        let dir_path = PathBuf::new();
        res.extend(self.project(&dir_path));
        // End temporary

        for type_collection in &file.type_collections {
            let x = self.type_collection(&type_collection);
            res.extend(x);
        }
        for interface in &file.interfaces {
            let x = self.interface(&interface);
            res.extend(x);
        }

        res
    }

    fn version(&self, version: &Option<Version>) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        match version {
            Some(version) => {
                res.push(IndentedString::new(
                    1,
                    FidlType::File,
                    format!(
                        "pub const VERSION_MAJOR: u32 = {:?};",
                        version.major.expect("Should exist")
                    ),
                ));
                res.push(IndentedString::new(
                    1,
                    FidlType::File,
                    format!(
                        "pub const VERSION_MINOR: u32 = {:?};",
                        version.minor.expect("Should exist")
                    ),
                ));
            }
            None => {
                res.push(IndentedString::new(
                    1,
                    FidlType::File,
                    format!("pub const VERSION_MAJOR: u32 = {:?};", 0),
                ));
                res.push(IndentedString::new(
                    1,
                    FidlType::File,
                    format!("pub const VERSION_MINOR: u32 = {:?};", 0),
                ));
            }
        }
        res
    }

    fn type_collection(&self, type_collection: &TypeCollection) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        // An interface is equivalent to a Rust Module
        let module = IndentedString::new(
            0,
            FidlType::File,
            format!("pub mod {} {{", type_collection.name),
        );
        res.push(module);
        res.push(IndentedString::new(
            1,
            FidlType::Interface,
            "use super::Primitives::*;".to_string(),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::Interface,
            "use super::*;".to_string(),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::Interface,
            "use super::FidlContext;".to_string(),
        ));

        res.extend(self.version(&type_collection.version));
        for typedef in &type_collection.typedefs {
            let typedef: Vec<IndentedString> = self
                .typedef(typedef, true)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(typedef)
        }
        for structure in &type_collection.structures {
            let structure: Vec<IndentedString> = self
                .structure(structure, true)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(structure)
        }
        for enumeration in &type_collection.enumerations {
            let enumeration: Vec<IndentedString> = self
                .enumeration(enumeration, true)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(enumeration)
        }
        let end_bracket = IndentedString::new(0, FidlType::Interface, format!("}}"));
        res.push(end_bracket);

        res
    }

    fn interface(&self, interface: &Interface) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        // An interface is equivalent to a Rust Module
        let module = IndentedString::new(
            0,
            FidlType::Interface,
            format!("pub mod {} {{", interface.name),
        );
        res.push(module);
        res.push(IndentedString::new(
            1,
            FidlType::Interface,
            "use super::Primitives::*;".to_string(),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::Interface,
            "use super::*;".to_string(),
        ));
        res.push(IndentedString::new(
            1,
            FidlType::Interface,
            "use super::FidlContext;".to_string(),
        ));

        res.extend(self.version(&interface.version));
        for typedef in &interface.typedefs {
            let typedef: Vec<IndentedString> = self
                .typedef(typedef, false)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(typedef)
        }
        for attribute in &interface.attributes {
            let attr: Vec<IndentedString> = self
                .attribute(attribute)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(attr);
        }
        for method in &interface.methods {
            let method: Vec<IndentedString> = self
                .method(method)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(method)
        }
        for structure in &interface.structures {
            let structure: Vec<IndentedString> = self
                .structure(structure, false)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(structure)
        }
        for enumeration in &interface.enumerations {
            let enumeration: Vec<IndentedString> = self
                .enumeration(enumeration, false)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(enumeration)
        }
        let end_bracket = IndentedString::new(0, FidlType::Interface, format!("}}"));
        res.push(end_bracket);
        res
    }
    fn attribute(&self, attribute: &Attribute) -> Vec<IndentedString> {
        // An attribute is some data the interface holds on the provider side, since we must communicate with a binary protocol
        // This gets converted into a get and set method for the attribute.
        let mut res: Vec<IndentedString> = Vec::new();
        let header = IndentedString::new(
            0,
            FidlType::Structure,
            format!(
                "fn set_{}(ctx: impl FidlContext, {}: {}) {{ ",
                attribute.name,
                attribute.name.to_lowercase(),
                attribute.type_n
            ),
        );
        res.push(header);
        let header = IndentedString::new(0, FidlType::Structure, format!("}}"));
        res.push(header);
        let header = IndentedString::new(
            0,
            FidlType::Structure,
            format!("pub fn get_{}() {{ ", attribute.name),
        );
        res.push(header);
        let header = IndentedString::new(0, FidlType::Structure, format!("}}"));
        res.push(header);

        res
    }

    fn structure(&self, structure: &Structure, public: bool) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        res.push(IndentedString::new(
            0,
            FidlType::Structure,
            "#[derive(Debug, Serialize, Deserialize, BinarySerde, PartialEq)]".to_string(),
        ));
        res.push(IndentedString::new(
            0,
            FidlType::Structure,
            "#[repr(C)]".to_string(),
        ));

        let header: IndentedString;
        if public {
            header = IndentedString::new(
                0,
                FidlType::Structure,
                format!("pub struct {} {{ ", structure.name),
            );
        } else {
            header = IndentedString::new(
                0,
                FidlType::Structure,
                format!("pub struct {} {{ ", structure.name),
            );
        }

        res.push(header);
        for var_dec in &structure.contents {
            if var_dec.is_array {
                let var_dec = format!("pub {}: [{}; 0],", var_dec.name, var_dec.type_n);
                res.push(IndentedString::new(1, FidlType::Structure, var_dec));
            } else {
                let var_dec = format!("pub {}: {},", var_dec.name, var_dec.type_n);
                res.push(IndentedString::new(1, FidlType::Structure, var_dec));
            }
        }
        let header = IndentedString::new(0, FidlType::Structure, format!("}}"));
        res.push(header);
        res
    }

    fn typedef(&self, typedef: &TypeDef, public: bool) -> Vec<IndentedString> {
        vec![IndentedString::new(
            0,
            FidlType::File,
            format!("use {} as {};", typedef.type_n, typedef.name),
        )]
    }

    fn method(&self, method: &Method) -> Vec<IndentedString> {
        let mut input_params = "".to_string();
        for param in &method.input_parameters {
            input_params += &param.name;
            input_params += ": ";
            input_params += &param.type_n;
            input_params += ", "
        }
        if input_params.len() != 0 {
            input_params = input_params[0..input_params.len() - 2].to_string();
        }

        let mut output_params = "".to_string();
        match method.output_parameters.len() {
            0 => {
                output_params += "()";
            }
            1 => {
                let single_param = &method.output_parameters[0];
                output_params = format!("{}", single_param.type_n);
            }
            e => {
                output_params.push('(');
                for param in &method.output_parameters {
                    output_params += &param.type_n;
                    output_params += ", "
                }
                output_params = output_params[0..output_params.len() - 2].to_string();
                output_params.push(')');
            }
        }
        let mut res: Vec<IndentedString> = Vec::new();
        res.push(IndentedString::new(
            0,
            FidlType::Method,
            format!(
                "pub fn {}(ctx: impl FidlContext, {}) -> {} {{",
                method.name, input_params, output_params
            )
            .to_string(),
        ));

        res.push(IndentedString::new(0, FidlType::Method, format!("}}")));
        res
    }
    fn enumeration(&self, enumeration: &Enumeration, public: bool) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        res.push(IndentedString::new(
            0,
            FidlType::Enumeration,
            "#[derive(Debug, Serialize, Deserialize, BinarySerde, PartialEq, Eq)]".to_string(),
        ));
        res.push(IndentedString::new(
            0,
            FidlType::Structure,
            "#[repr(u8)]".to_string(),
        ));

        let header: IndentedString;
        if public {
            header = IndentedString::new(
                0,
                FidlType::Enumeration,
                format!("pub enum {} {{ ", enumeration.name),
            );
        } else {
            header = IndentedString::new(
                0,
                FidlType::Enumeration,
                format!("enum {} {{ ", enumeration.name),
            );
        }
        res.push(header);
        for enum_value in &enumeration.values {
            let var_dec: String;
            match enum_value.value {
                Some(value) => {
                    println!("Warning: Value: {:?} for {} in {}, Enum Values behaviour is currently language defined.", value, enum_value.name, enumeration.name);
                    var_dec = format!("{} = {:?},", enum_value.name, value);
                }
                None => {
                    var_dec = format!("{},", enum_value.name);
                }
            }
            res.push(IndentedString::new(1, FidlType::Enumeration, var_dec));
        }
        let header = IndentedString::new(0, FidlType::Enumeration, format!("}}"));
        res.push(header);
        res
    }
}
