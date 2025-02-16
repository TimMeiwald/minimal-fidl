use minimal_fidl_formatter::Formatter;
use minimal_fidl_parser::{
    BasicContext, Context, Key, Rules, Source, _var_name, grammar, RULES_SIZE,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::path::Path;
use std::process::exit;
use std::time::Instant;


pub fn minimal_fidl_fmt(paths: &[PathBuf], dry_run: bool) {
    let paths = walk_dirs(&paths[0]).expect("Some error occurred");
    // Context should be reuseable if cleared after each parse but that's unstable so we pretend we can do this but actualy make a new
    // Context in the format file.
    //Dummy context since it should be reusable but not sure if it is right now
    let ctx: RefCell<BasicContext> =
        RefCell::new(BasicContext::new(0_usize, RULES_SIZE as usize));
    let start = Instant::now();
    let mut success_count = 0;
    let mut err_count = 0;
    for path in paths {
        let instant = Instant::now();
        let formatted_string: Result<String, ()> = format_file(&ctx, &path);
        let instant_after = Instant::now();
        let duration = instant_after - instant;
        println!("Time to format {:#?}", duration);
        match formatted_string {
            Ok(formatted_string) => {
                if dry_run {
                    println!("Dry run: {:?}", path);
                    println!("Formatted text: \n{}", formatted_string);
                    success_count += 1;
                } else {
                    let write_result = std::fs::write(&path, formatted_string);
                    match write_result {
                        Ok(()) => {
                            println!("Successfully wrote out formatted file: {:?}", path);
                            success_count += 1;
                        }
                        Err(_e) => {
                            err_count += 1;
                            println!("Error writing file: {:?}", path);
                        }
                    }
                }
            }
            Err(_e) => {
                err_count +=1 ;
                println!("Error formatting file: {:?}", path);
            }
        }
    }
    let end = Instant::now();
    println!("Successfully formatted: {}/{}", success_count, err_count + success_count);
    println!("Total time elapsed {:#?}", end-start);
    if success_count == (err_count + success_count){
        exit(0)
    }
    else{
        exit(1)
    }


}

fn is_fidl_file(path: &Path) -> bool {
    let extension = path.extension();
    match extension {
        Some(extension) => {
            extension == "fidl"
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

fn format_file(_ctx: &RefCell<BasicContext>, path: &PathBuf) -> Result<String, ()> {
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
        println!(
            "Successfully parsed up to char: {:?} out of total chars: {src_len}",
            result.1
        );
        println!("Error failed to parse: {:?}\n", path);
        return Err(());
    }
    let publisher = ctx.into_inner().get_publisher().clear_false();
    // let formatted_text = publisher.print(Key(0), Some(true));
    println!("\nParsing file: {:?}", path);
    let fmt = Formatter::new(&string, &publisher);
    let formatted_text = fmt.format();
    match formatted_text {
        Err(_formatter_err) => {
            println!("Could not format");
            Err(())
        }
        Ok(formatted_text) => {
            println!("Wooo formatted file: {:?}", path);
            Ok(formatted_text)
        }
    }
}
