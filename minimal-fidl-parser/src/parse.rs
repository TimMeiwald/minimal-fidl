use crate::parser::grammar;
use crate::var_name::_var_name;
use crate::{context::BasicContext, rules::RULES_SIZE, Key};

use super::*;
use core::cell::RefCell;
use std::env;
use std::fs::{canonicalize, read_to_string};

pub fn parse(source: &String) -> (bool, u32, BasicPublisher) {
    let src_len = source.len() as u32;
    let source = Source::new(&source);
    let position: u32 = 0;
    let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
    let result: (bool, u32);
    {
        let executor = _var_name(Rules::Grammar, &context, grammar);
        result = executor(Key(0), &source, position);
    }
    let gen_code = context.into_inner().get_publisher().clear_false();
    (result.0, result.1, gen_code)
}
