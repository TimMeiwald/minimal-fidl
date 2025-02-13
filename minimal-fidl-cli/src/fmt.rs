use minimal_fidl_formatter::Formatter;
use minimal_fidl_parser::{
    BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, grammar, RULES_SIZE,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::exit;

pub fn minimal_fidl_fmt(paths: &Vec<PathBuf>, dry_run: bool) {
    println!("Paths: {:?}", paths);
    println!("Dry Run: {dry_run}");

    let paths = walk_dirs(&paths[0]).expect("Some error occurred");
    // Context should be reuseable if cleared after each parse but that's unstable so we pretend we can do this but actualy make a new
    // Context in the format file.
    //Dummy context since it should be reusable but not sure if it is right now
    let ctx = RefCell::new(BasicContext::new(0 as usize, RULES_SIZE as usize));
    for path in paths {
        format_file(&ctx, &path);
    }
}

fn is_fidl_file(path: &PathBuf) -> bool {
    let extension = path.extension();
    match extension {
        Some(extension) => {
            if extension == "fidl" {
                true
            } else {
                false
            }
        }
        None => false,
    }
}

fn walk_dirs(path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut ret_vec: Vec<PathBuf> = Vec::new();
    if path.is_dir() {
        for path in std::fs::read_dir(path)? {
            let path = path?;
            let path = path.path();
            if path.is_dir() {
                let paths: Vec<PathBuf> = walk_dirs(&path)?;
                ret_vec.extend(paths);
            } else {
                ret_vec.push(path);
            }
        }
    }
    let ret_vec: Vec<PathBuf> = ret_vec
        .iter()
        .filter(|path| is_fidl_file(path))
        .map(|path| path.to_path_buf())
        .collect();
    Ok(ret_vec)
}

fn format_file(ctx: &RefCell<BasicContext>, path: &PathBuf) -> Result<String, ()> {
    let input = std::fs::read_to_string(path).expect("Expected file to exist");
    let string = input.to_string();
    let src_len = string.len();
    let source = Source::new(&string);
    let position: u32 = 0;
    let result: (bool, u32);
    let ctx = RefCell::new(BasicContext::new(src_len, RULES_SIZE as usize));
    {
        let executor = _var_name(Rules::Grammar, &ctx, grammar);
        result = executor(Key(0), &source, position);
    }
    if !result.0 || result.1 != src_len as u32 {
        // Error failed to parse
        println!("Successfully parsed up to char: {:?} out of total chars: {src_len}", result.1);
        println!("Error failed to parse: {:?}\n", path);
        return Err(())
    }
    let publisher = ctx.into_inner().get_publisher().clear_false();
    // let formatted_text = publisher.print(Key(0), Some(true));
    println!("Parsing file: {:?}", path);
    let fmt = Formatter::new(&string, &publisher);
    let formatted_text = fmt.format();
    match formatted_text {
        Err(formatter_err) => {
            println!("Could not format");
            Err(())
        }
        Ok(formatted_text) => {
            println!("Wooo parsed file: {:?}", path);
            println!("{formatted_text}");
            Ok(formatted_text)
        }
    }
}
