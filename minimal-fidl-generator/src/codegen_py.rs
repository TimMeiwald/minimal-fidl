use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::{fmt::format, path::PathBuf};

use crate::codegen_trait::{CodeGenerator, GeneratorError};
use crate::indented_string::IndentedString;
use crate::FidlType;
use minimal_fidl_collect::annotation::Annotation;
use minimal_fidl_collect::enum_value::EnumValue;
use minimal_fidl_collect::{annotation, enum_value, fidl_file, FidlProject};
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
use num_traits::int;

pub struct PythonCodeGen {
    // Generate file creates a vector of strings because one fidl file can generate multiple source code files
    // in languages where a module is a file. E.g Python
    pub python_code: HashMap<PathBuf, Vec<IndentedString>>,
}
impl std::fmt::Debug for PythonCodeGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (path, vec) in &self.python_code {
            write!(f, "\n\n{:?}\n", path)?;
            for src in vec {
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

    fn emit_project(&self, target_dir: PathBuf) -> Result<(), GeneratorError> {
        match std::fs::create_dir_all(&target_dir) {
            Err(err) => return Err(GeneratorError::IoError(err)),
            _ => {}
        };
        for (path, content) in &self.python_code {
            let new_path = target_dir.clone().join(path);
            println!("{:?}", new_path);
            let parent = new_path.parent();
            if parent.is_some() {
                // Mkdirs if needed
                let parent = parent.unwrap();
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::File::create(new_path)?;
            let str = self.create_string(content);
            file.write(str.as_bytes())?;
        }
        Ok(())
    }

    // fn generate_file(&mut self, path: PathBuf, fidl: FidlFile) -> Result<(), GeneratorError> {
    //     let file = self.file(path.clone(), &fidl);
    //     let mut str: String = "".to_string();

    //     let mut vec: Vec<String> = Vec::new();
    //     vec.push(str);
    //     self.python_code.insert(path, vec);
    //     Ok(())
    // }
    fn generate_project(&mut self, dir: PathBuf) -> Result<(), GeneratorError> {
        let dir_clone = dir.clone();
        let paths = FidlProject::new(dir);
        self.project(&dir_clone);
        for path in paths.unwrap() {
            let fidl = FidlProject::generate_file(path.clone())?;
            // This needs to be modified because I want to get each interface and type collection as a
            // seperate file.
            // But it's not part of the trait anymore so that's fine.
            // We then also create a primitive types at root level that the rest can import
            // We'll need JSON and Little Endian serde for debug and to send to comms
            // Then each method will need to accept the inputs, serialize them
            // Send through some function
            // Deserialize the returned value. Async/Sync as options. Maybe only async since we can always force sync using async
            // Also need to add annotation block details support.
            let mut p = path.clone();
            p.set_extension("");
            self.file(p, &fidl);
        }
        Ok(())
    }
}

impl PythonCodeGen {
    fn create_string(&self, input: &Vec<IndentedString>) -> String {
        let mut str = "".to_string();
        for line in input {
            str += &line.to_string();
        }
        str
    }

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
        let built_ins = include_str!("../common_python/built_in_fidl_types.py");

        let mut res: Vec<IndentedString> = Vec::new();

        res.push(IndentedString::new(
            0,
            FidlType::File,
            built_ins.to_string(),
        ));
        res
    }

    fn context_trait(&self) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();

        let module = IndentedString::new(0, FidlType::File, format!("class Comms:",));
        res.push(module);

        let module = IndentedString::new(1, FidlType::File, format!("pass",));
        res.push(module);
        res
    }

    fn project(&mut self, dir: &PathBuf) -> () {
        let init_path = dir.clone().join("__init__.py");
        self.python_code.insert(init_path, Vec::new());
        let built_ins = self.built_in_types();
        let path = dir.join("built_in_fidl_types.py");
        self.python_code.insert(dir.with_file_name(path), built_ins);
        let comm_handler = self.context_trait();
        let path = dir.join("comm_handler.py");
        self.python_code
            .insert(dir.with_file_name(path), comm_handler);
    }

    fn file(&mut self, path: PathBuf, file: &FidlFile) -> () {
        let init_path = path.clone().join("__init__.py");
        self.python_code.insert(init_path, Vec::new());

        for type_collection in &file.type_collections {
            let type_collection_name = &type_collection.name;
            let x = self.type_collection(&type_collection);
            let mut p = path.clone();
            p.push(type_collection_name);
            p.set_extension(".py");
            self.python_code.insert(p, x);
        }
        for interface in &file.interfaces {
            let interface_name = &interface.name;
            let x = self.interface(&interface);
            let mut p = path.clone();
            p.push(interface_name);
            p.set_extension(".py");
            self.python_code.insert(p, x);
        }
    }

    fn version(&self, version: &Option<Version>) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        match version {
            Some(version) => {
                res.push(IndentedString::new(
                    0,
                    FidlType::File,
                    format!(
                        "VERSION_MAJOR: int = {:?}",
                        version.major.expect("Should exist")
                    ),
                ));
                res.push(IndentedString::new(
                    0,
                    FidlType::File,
                    format!(
                        "VERSION_MINOR: int = {:?}\n",
                        version.minor.expect("Should exist")
                    ),
                ));
            }
            None => {
                res.push(IndentedString::new(
                    0,
                    FidlType::File,
                    format!("VERSION_MAJOR: int = {:?}", 0),
                ));
                res.push(IndentedString::new(
                    0,
                    FidlType::File,
                    format!("VERSION_MINOR: int = {:?}\n", 0),
                ));
            }
        }
        res
    }

    fn type_collection(&self, type_collection: &TypeCollection) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Enumeration,
            format!(
                "from enum import IntEnum
"
            ),
        );
        res.push(header);
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Enumeration,
            format!("from dataclasses import dataclass"),
        );
        res.push(header);
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Enumeration,
            format!("from comm_handler import Comms"),
        );
        res.push(header);
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Enumeration,
            format!("from built_in_fidl_types import *"),
        );
        res.push(header);
        res.extend(self.version(&type_collection.version));
        for typedef in &type_collection.typedefs {
            let typedef: Vec<IndentedString> = self.typedef(typedef);
            res.extend(typedef)
        }
        for structure in &type_collection.structures {
            let structure: Vec<IndentedString> = self.structure(structure);
            res.extend(structure)
        }
        for enumeration in &type_collection.enumerations {
            let enumeration: Vec<IndentedString> = self.enumeration(enumeration);
            res.extend(enumeration)
        }
        res
    }

    fn interface(&self, interface: &Interface) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        let id = Self::method_and_interface_split_annotation_content(&interface.annotations);
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Enumeration,
            format!("from enum import IntEnum"),
        );
        res.push(header);
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Enumeration,
            format!("from dataclasses import dataclass"),
        );
        res.push(header);
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Enumeration,
            format!("from comm_handler import Comms"),
        );
        res.push(header);
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Enumeration,
            format!("from built_in_fidl_types import *"),
        );
        res.push(header);

        if id.is_some(){
            let id = id.unwrap();
            res.push(IndentedString::new(
                0,
                FidlType::Interface,
                format!("ID = {:?}", id),
            ));
        }
        res.extend(self.version(&interface.version));
        for typedef in &interface.typedefs {
            let typedef: Vec<IndentedString> = self.typedef(typedef);
            res.extend(typedef)
        }
        for attribute in &interface.attributes {
            let attr: Vec<IndentedString> = self.attribute(attribute);
            res.extend(attr);
        }
        for method in &interface.methods {
            let method: Vec<IndentedString> = self.method(method);
            res.extend(method)
        }
        for structure in &interface.structures {
            let structure: Vec<IndentedString> = self.structure(structure);
            res.extend(structure)
        }
        for enumeration in &interface.enumerations {
            let enumeration: Vec<IndentedString> = self.enumeration(enumeration);
            res.extend(enumeration)
        }
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
                "def set_{}(ctx: Comms, {}: {}):",
                attribute.name,
                attribute.name.to_lowercase(),
                attribute.type_n
            ),
        );
        res.push(header);
        res.push(IndentedString::new(
            1,
            FidlType::Attribute,
            "pass".to_string(),
        ));
        res.push(IndentedString::new(1, FidlType::Attribute, "".to_string()));

        let header = IndentedString::new(
            0,
            FidlType::Structure,
            format!("def get_{}() -> {}: ", attribute.name, attribute.type_n),
        );
        res.push(header);
        res.push(IndentedString::new(
            1,
            FidlType::Attribute,
            "pass".to_string(),
        ));
        res.push(IndentedString::new(0, FidlType::Attribute, "".to_string()));

        res
    }

    fn structure(&self, structure: &Structure) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        let header: IndentedString;
        header = IndentedString::new(0, FidlType::Structure, format!("@dataclass(frozen=True)"));

        res.push(header);
        let header: IndentedString;
        header = IndentedString::new(
            0,
            FidlType::Structure,
            format!("class {}():", structure.name),
        );

        res.push(header);

        for var_dec in &structure.contents {
            if var_dec.is_array {
                let var_dec = format!("{}: List[{}]", var_dec.name, var_dec.type_n);
                res.push(IndentedString::new(1, FidlType::Structure, var_dec));
            } else {
                let var_dec = format!("{}: {}", var_dec.name, var_dec.type_n);
                res.push(IndentedString::new(1, FidlType::Structure, var_dec));
            }
        }
        res.push(IndentedString::new(0, FidlType::Structure, "".to_string()));
        res
    }

    fn typedef(&self, typedef: &TypeDef) -> Vec<IndentedString> {
        vec![
            IndentedString::new(
                0,
                FidlType::File,
                format!("class {}({}):", typedef.type_n, typedef.name),
            ),
            IndentedString::new(
                1,
                FidlType::File,
                format!("'''This is a type definition.'''\n"),
            ),
        ]
    }

    fn method(&self, method: &Method) -> Vec<IndentedString> {
        let mut input_params = "".to_string();
        let id = Self::method_and_interface_split_annotation_content(&method.annotations);
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
                "def {}(ctx: Comms, {}) -> {}:",
                method.name, input_params, output_params
            )
            .to_string(),
        ));
        if id.is_some() {
            let id = id.unwrap();
            res.push(IndentedString::new(1, FidlType::Method, format!("Id: int = {:?}", id)));
        }
        res.push(IndentedString::new(1, FidlType::Method, format!("pass\n")));
        res
    }

    fn enumeration_value_gatherer(&self, enumeration: &Enumeration) -> (u64, HashMap<String, u64>) {
        // The goal is to have as compact a representation as possible
        // So we need to do some work to allow for hardcoded enum values and autovalued enum values
        // in the same enum structure that don't waste numbers to keep things compact.

        let mut enum_name_value: HashMap<String, u64> = HashMap::new(); // Largest supported enum value u64, UB if larger.
        let mut exists_already: HashMap<u64, ()> = HashMap::new();

        // Assign all hardcoded values.
        for enum_value in &enumeration.values {
            match enum_value.value {
                Some(value) => {
                    enum_name_value.insert(enum_value.name.clone(), value);

                    let result = exists_already.insert(value, ());
                    if result.is_some() {
                        panic!("Cannot have two identical values assigned to different enum values in {}", enumeration.name);
                    }
                }
                None => {
                    // Do nothing we handle this in next loop
                }
            }
        }

        let mut count: u64 = 0;
        for enum_value in &enumeration.values {
            match enum_value.value {
                Some(value) => {
                    // Do nothing since this has already been handled.
                }
                None => {
                    loop {
                        if exists_already.contains_key(&count) {
                            count += 1;
                        } else {
                            break;
                        }
                    }
                    let value = count;
                    count += 1;
                    enum_name_value.insert(enum_value.name.clone(), value);
                }
            }
        }
        let mut largest_value = 0;
        for (_name, value) in &enum_name_value {
            if *value >= largest_value {
                largest_value = *value;
            }
        }
        (largest_value, enum_name_value)
    }

    fn enumeration_split_annotation_content(annotations: &Vec<Annotation>) -> Option<u64> {
        // Returns size element that can be used to hardcode enum size.
        for annotation in annotations {
            if annotation.name.trim() == "details" {
                let contents = annotation.contents.trim();
                if contents.trim().to_lowercase().starts_with("size"){
                    let contents: Vec<&str> = contents.split("=").collect();
                    assert!(contents.len() == 2, "Expected only one equals sign");
                    let value = EnumValue::convert_string_representation_of_number_to_value(
                        contents[1].trim().to_string(),
                    )
                    .expect("Expected a valid positive integer.");
                    return Some(value);
                }
                
            }
        }
        None
    }
    fn method_and_interface_split_annotation_content(annotations: &Vec<Annotation>) -> Option<u64> {
        // Returns size element that can be used to hardcode enum size.
        for annotation in annotations {
            if annotation.name.trim() == "details" {
                let contents = annotation.contents.trim();
                if contents.trim().to_lowercase().starts_with("id"){
                    let contents: Vec<&str> = contents.split("=").collect();
                    assert!(contents.len() == 2, "Expected only one equals sign");
                    let value = EnumValue::convert_string_representation_of_number_to_value(
                        contents[1].trim().to_string(),
                    )
                    .expect("Expected a valid positive integer.");
                    return Some(value);
                }
                
            }
        }
        None
    }

    fn enumeration(&self, enumeration: &Enumeration) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        let (largest_value, enumeration_map) = self.enumeration_value_gatherer(enumeration);
        let hardcoded_size: Option<u64> = Self::enumeration_split_annotation_content(&enumeration.annotations);
        let mut size = 8;
        if largest_value > 255 {
            size = 16;
        } else if largest_value > 65535 {
            size = 32
        } else if largest_value > 4294967295 {
            size = 64
        }
        if hardcoded_size.is_some(){
            let hardcoded_size = hardcoded_size.unwrap();
            // If the hardcoded sise is not large enough for the number of enum variants we panic
            // due to the mismatch between user demanded behaviour and reality. 
            if size > hardcoded_size {
                panic!("The hardcoded size {:?} is not large enough for the number of enum variants. Expected at least: {:?}", hardcoded_size, size);
            }
            else{
                if hardcoded_size != 8 && hardcoded_size != 16 && hardcoded_size != 32 && hardcoded_size != 64{
                    panic!("The hardcoded size {:?} must be 8, 16, 32 or 64", hardcoded_size);
                }
                size = hardcoded_size
            }
            
        }
        let header = IndentedString::new(
            0,
            FidlType::Structure,
            format!("class {}(u{size}IntEnum):", enumeration.name),
        );
        res.push(header);
        for enum_value in &enumeration.values {
            let value = enumeration_map
                .get(&enum_value.name)
                .expect("We expect them to exist since we put them there in the gather function");
            let header = IndentedString::new(
                1,
                FidlType::Structure,
                format!("{} = {},", enum_value.name, value),
            );
            res.push(header);
        }

        res
    }
}
