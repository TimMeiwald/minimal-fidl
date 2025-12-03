use std::cell::RefCell;
use std::path::PathBuf;
use minimal_fidl_parser::{BasicContext, Source, RULES_SIZE, Rules, _var_name, Context, grammar, Key};
use minimal_fidl_formatter::Formatter;
use tower_lsp::lsp_types::Position;

fn get_position(src: &str) -> Position {
	let mut line_count = 0;
	let mut character_count = 0;
	for i in src.as_bytes(){
		if *i == b'\n'{
			line_count += 1;
			character_count = 0;
		}
		else{
			character_count += 1;
		}
	}
	Position { line: line_count, character: character_count }
}

pub fn format_file_one_shot_context(path: &str) -> Result<(Position, String), String> {
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
        let err_str = format!(
            "Successfully parsed up to char: {:?} out of total chars: {src_len}\n",
            result.1
        );
        let err_str2 = format!("Error failed to parse: {:?}\n", path);
        return Err(err_str + &err_str2);
    }
    let publisher = ctx.into_inner().get_publisher().clear_false();
    let fmt = Formatter::new(&string, &publisher);
    let formatted_text = fmt.format();
    match formatted_text {
        Err(formatter_err) => {
            Err(format!("Pasred but could not format: {:?}", formatter_err).to_string())
        }
        Ok(formatted_text) => {
			let position = get_position(&string);
            Ok((position, formatted_text))
        }
    }
}