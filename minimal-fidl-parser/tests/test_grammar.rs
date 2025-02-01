use test_grammar_proc_macro::test_grammar_files_in_dir;
mod shared;
use shared::shared;
use minimal_fidl_parser::{grammar, BasicContext, Rules};
use std::fs;


#[test]
fn test_grammar_1() {
    let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { }	";
    let result = shared(src, grammar::<BasicContext>, Rules::Grammar);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_grammar_2() {
    let src = "package org.javaohjavawhyareyouso // This do be a comment\n
        interface endOfPlaylist { }	// This do be a comment\n
        interface endOfPlaylist { }	// This do be a comment\n";
    let result = shared(src, grammar::<BasicContext>, Rules::Grammar);
    assert_eq!(result, (true, src.len() as u32));
}



test_grammar_files_in_dir!("minimal-fidl-parser/tests/grammar_test_files");