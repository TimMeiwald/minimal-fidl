use clap::Parser;
use minimal_fidl_parser::*;
use std::{fs, path::PathBuf, process::exit};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to Grammar file you would like to generate a parser for.
    #[arg(short, long)]
    source: PathBuf,
}

fn main() {
    let args = Args::parse();

    //let source: Path = args.source.canonicalize().ok_or_else(panic!("Failed to find Source File"));
    let source: PathBuf = match args.source.canonicalize() {
        Err(_) => panic!("Failed to find Source File"),
        Ok(source) => source,
    };
    let source_str = match fs::read(source.clone()) {
        Err(e) => {
            println!("{:?}", e);
            println!("Failed to read file at {:?}", source);
            exit(1)
        }
        Ok(source_str) => {
            println!("Successfully read source file: {:?}", source);
            let source_str = match String::from_utf8(source_str) {
                Err(e) => {
                    println!("{:?}", e);
                    println!("Failed to read file as utf8");
                    exit(2)
                }
                Ok(source_str) => {
                    let result = parse(&source_str);
                    result.2.print(crate::Key(0), Some(true));
                    println!("Result: {:?}", (result.0, result.1));
                    let src_len = source_str.len();
                    if src_len as u32 != result.1 {
                        println!(
                            "Failed to parse to end of file. Parsed {:?} chars from {:?} chars",
                            result.1, src_len
                        );
                    }
                }
            };
        }
    };
}
