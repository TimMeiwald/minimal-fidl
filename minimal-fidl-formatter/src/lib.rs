use minimal_fidl_parser::{
    BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, grammar, RULES_SIZE,
};
mod formatter;
use std::cell::RefCell;
use formatter::Formatter;

pub fn parse(input: &str) -> Option<BasicPublisher> {
    let string = input.to_string();
    let src_len = string.len() as u32;
    let source = Source::new(&string);
    let position: u32 = 0;
    let result: (bool, u32);
    let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
    {
        let executor = _var_name(Rules::Grammar, &context, grammar);
        result = executor(Key(0), &source, position);
    }
    if result != (true, src_len){
        println!("Failed with : {:?}", result);
        return None;
    }

    let publisher = context.into_inner().get_publisher().clear_false();
    Some(publisher)
}


#[test]
fn test_formatter(){
    let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { }	";
    let publisher = parse(src).unwrap();
    publisher.print(Key(0), Some(true));
    let fmt = formatter::Formatter::new(src, &publisher);
    let output = fmt.format();
    println!("Formatted:\n\n{}", output.unwrap());
}

#[test]
fn test_formatter2(){
    let src = "// This do be a comment\npackage org.javaohjavawhyareyouso        // This do be a comment2\n
	interface endOfPlaylist { }	// This do be a comment\n
    interface endOfPlaylist { }	// This do be a comment\n";
    let publisher = parse(src).unwrap();
    publisher.print(Key(0), Some(true));
    let fmt = formatter::Formatter::new(src, &publisher);
    let output = fmt.format();
    println!("Formatted:\n\n{}", output.unwrap());
}