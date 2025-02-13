extern crate proc_macro;
extern crate quote;
extern crate syn;

use std::{
    path::{self, Path},
    str::FromStr,
};

use proc_macro2::{Span, TokenStream};
use syn::{ext, parse_macro_input, Ident, LitStr};
#[proc_macro]
pub fn test_grammar_files_in_dir(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let constant_directory = parse_macro_input!(input as LitStr).value();
    println!("{:?}", constant_directory);
    let dir = Path::new(&constant_directory);
    let err_msg = format!(
        "Expected grammar_test_files dir exists: {:?}\nWarning: Path is relative to workspace root not file root for now",
        dir
    );
    let files = std::fs::read_dir(dir).expect(&err_msg);
    let mut return_stream = TokenStream::new();
    for path in files {
        // Actually read and test each file
        let path = path.expect("Should exist").path();
        let file_name = Path::new(&path)
            .file_name()
            .expect("Expected file to exist")
            .to_str()
            .expect("Should be able to convert to string");
        let file_name_for_ident = Path::new(&path)
            .file_stem()
            .expect("Expected file to exist")
            .to_str()
            .expect("Should be able to convert to string");
        let file_name_for_ident = file_name_for_ident.to_lowercase().replace('-', "_");
        // constant_directory is a filepath for now
        let file_name_for_ident = format!("test_grammar_file_{}", file_name_for_ident);
        let test_ident = syn::Ident::new(&file_name_for_ident, Span::call_site());
        let cwd = std::env::current_dir().unwrap();
        let filepath = path::Path::new(&cwd)
            .join(&constant_directory)
            .join(file_name);
        let filepath = filepath.to_str().expect("Should be able to turn into str");
        let filepath = syn::LitStr::new(filepath, Span::call_site());
        let stream = quote::quote!(
            #[test]
            fn #test_ident(){
                let file_path = #filepath;
                let src = fs::read_to_string(file_path);
                let src = match src {
                    Err(err) => {
                        panic!("Error: {:?}", err)
                    }
                    Ok(src) => src,
                };
                let result = shared(&src, grammar::<BasicContext>, Rules::Grammar);
                if result.0 != true{
                    println!("Result is {:?}", result)
                }
                assert_eq!(result, (true, src.len() as u32))
        });
        return_stream.extend(stream);
    }

    return_stream.into()
}

// let traits: Vec<&str> = vec!["std::default::Default", "std::fmt::Debug"];
// syn::parse_str::<syn::Type>(&traits.join(" + ")).unwrap();
