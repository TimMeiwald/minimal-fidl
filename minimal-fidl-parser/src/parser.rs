#![allow(non_camel_case_types)] // Generated Code kinda annoying to deal with so w/e
#![allow(unused_variables)] // Generated Code also, since everything passes stuff
#![allow(unused_imports)] // Generated Code also, since everything passes stuff
use crate::*;
use std::cell::RefCell;
#[allow(dead_code)]
pub fn ws<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::multiline_comment, context, multiline_comment);
	let closure_2 = _var_name(Rules::comment, context, comment);
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	let closure_4 = _terminal(b' ');
	let closure_5 = _ordered_choice(&closure_3, &closure_4);
	let closure_6 = _terminal(b'\t');
	let closure_7 = _ordered_choice(&closure_5, &closure_6);
	let closure_8 = _terminal(b'\r');
	let closure_9 = _ordered_choice(&closure_7, &closure_8);
	let closure_10 = _subexpression(&closure_9);
	let closure_11 = _zero_or_more(&closure_10);
	closure_11(parent, source, position)

} #[allow(dead_code)]
pub fn wsn<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::multiline_comment, context, multiline_comment);
	let closure_2 = _var_name(Rules::comment, context, comment);
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	let closure_4 = _terminal(b' ');
	let closure_5 = _ordered_choice(&closure_3, &closure_4);
	let closure_6 = _terminal(b'\t');
	let closure_7 = _ordered_choice(&closure_5, &closure_6);
	let closure_8 = _terminal(b'\r');
	let closure_9 = _ordered_choice(&closure_7, &closure_8);
	let closure_10 = _terminal(b'\n');
	let closure_11 = _ordered_choice(&closure_9, &closure_10);
	let closure_12 = _subexpression(&closure_11);
	let closure_13 = _zero_or_more(&closure_12);
	closure_13(parent, source, position)

} #[allow(dead_code)]
pub fn ws_atlone<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _terminal(b' ');
	let closure_2 = _terminal(b'\t');
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	let closure_4 = _subexpression(&closure_3);
	let closure_5 = _one_or_more(&closure_4);
	closure_5(parent, source, position)

} #[allow(dead_code)]
pub fn cwsn<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::multiline_comment, context, multiline_comment);
	let closure_2 = _var_name(Rules::comment, context, comment);
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	let closure_4 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_5 = _ordered_choice(&closure_3, &closure_4);
	closure_5(parent, source, position)

} #[allow(dead_code)]
pub fn ascii<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _ordered_choice_match_range(0, 255);
	closure_1(parent, source, position)

} #[allow(dead_code)]
pub fn multiline_comment<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'/',b'*']);
	let closure_2 = _string_terminal_opt_ascii(&[b'*',b'/']);
	let closure_3 = _not_predicate(&closure_2);
	let closure_4 = move |parent: Key, source: &Source, position: u32| ascii(parent, context, source, position);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _subexpression(&closure_5);
	let closure_7 = _zero_or_more(&closure_6);
	let closure_8 = _sequence(&closure_1, &closure_7);
	let closure_9 = _string_terminal_opt_ascii(&[b'*',b'/']);
	let closure_10 = _sequence(&closure_8, &closure_9);
	closure_10(parent, source, position)

} #[allow(dead_code)]
pub fn comment<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'/',b'/']);
	let closure_2 = _terminal(b'\n');
	let closure_3 = _not_predicate(&closure_2);
	let closure_4 = move |parent: Key, source: &Source, position: u32| ascii(parent, context, source, position);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _subexpression(&closure_5);
	let closure_7 = _zero_or_more(&closure_6);
	let closure_8 = _sequence(&closure_1, &closure_7);
	closure_8(parent, source, position)

} #[allow(dead_code)]
pub fn digit<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _ordered_choice_match_range(48, 57);
	closure_1(parent, source, position)

} #[allow(dead_code)]
pub fn digits<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| digit(parent, context, source, position);
	let closure_2 = _one_or_more(&closure_1);
	closure_2(parent, source, position)

} #[allow(dead_code)]
pub fn integer<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::sign, context, sign);
	let closure_2 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _var_name(Rules::digits, context, digits);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _var_name(Rules::exponent, context, exponent);
	let closure_7 = _optional(&closure_6);
	let closure_8 = _sequence(&closure_5, &closure_7);
	closure_8(parent, source, position)

} #[allow(dead_code)]
pub fn float<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::sign, context, sign);
	let closure_2 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _var_name(Rules::digits, context, digits);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _var_name(Rules::fraction, context, fraction);
	let closure_7 = _sequence(&closure_5, &closure_6);
	let closure_8 = _var_name(Rules::exponent, context, exponent);
	let closure_9 = _optional(&closure_8);
	let closure_10 = _sequence(&closure_7, &closure_9);
	closure_10(parent, source, position)

} #[allow(dead_code)]
pub fn fraction<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _terminal(b'.');
	let closure_2 = _var_name(Rules::digits, context, digits);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _subexpression(&closure_3);
	closure_4(parent, source, position)

} #[allow(dead_code)]
pub fn exponent<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_2 = _terminal(b'E');
	let closure_3 = _terminal(b'e');
	let closure_4 = _ordered_choice(&closure_2, &closure_3);
	let closure_5 = _subexpression(&closure_4);
	let closure_6 = _sequence(&closure_1, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::integer, context, integer);
	let closure_10 = _sequence(&closure_8, &closure_9);
	closure_10(parent, source, position)

} #[allow(dead_code)]
pub fn sign<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _terminal(b'+');
	let closure_2 = _terminal(b'-');
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	let closure_4 = _subexpression(&closure_3);
	let closure_5 = _optional(&closure_4);
	closure_5(parent, source, position)

} #[allow(dead_code)]
pub fn hex_char<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| digit(parent, context, source, position);
	let closure_2 = _ordered_choice_match_range(65, 70);
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	let closure_4 = _ordered_choice_match_range(97, 102);
	let closure_5 = _ordered_choice(&closure_3, &closure_4);
	closure_5(parent, source, position)

} #[allow(dead_code)]
pub fn hex<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'0',b'x']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| hex_char(parent, context, source, position);
	let closure_3 = _one_or_more(&closure_2);
	let closure_4 = _sequence(&closure_1, &closure_3);
	closure_4(parent, source, position)

} #[allow(dead_code)]
pub fn bin_char<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _terminal(b'0');
	let closure_2 = _terminal(b'1');
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	closure_3(parent, source, position)

} #[allow(dead_code)]
pub fn binary<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'0',b'b']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| bin_char(parent, context, source, position);
	let closure_3 = _one_or_more(&closure_2);
	let closure_4 = _sequence(&closure_1, &closure_3);
	closure_4(parent, source, position)

} #[allow(dead_code)]
pub fn number<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::hex, context, hex);
	let closure_2 = _var_name(Rules::binary, context, binary);
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	let closure_4 = _var_name(Rules::float, context, float);
	let closure_5 = _ordered_choice(&closure_3, &closure_4);
	let closure_6 = _var_name(Rules::integer, context, integer);
	let closure_7 = _ordered_choice(&closure_5, &closure_6);
	closure_7(parent, source, position)

} #[allow(dead_code)]
pub fn annotation_block<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'<',b'*',b'*']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _string_terminal_opt_ascii(&[b'*',b'*',b'>']);
	let closure_5 = _not_predicate(&closure_4);
	let closure_6 = _var_name(Rules::annotation, context, annotation);
	let closure_7 = _sequence(&closure_5, &closure_6);
	let closure_8 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_9 = _sequence(&closure_7, &closure_8);
	let closure_10 = _subexpression(&closure_9);
	let closure_11 = _one_or_more(&closure_10);
	let closure_12 = _sequence(&closure_3, &closure_11);
	let closure_13 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_14 = _sequence(&closure_12, &closure_13);
	let closure_15 = _string_terminal_opt_ascii(&[b'*',b'*',b'>']);
	let closure_16 = _sequence(&closure_14, &closure_15);
	closure_16(parent, source, position)

} #[allow(dead_code)]
pub fn annotation<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _terminal(b'@');
	let closure_2 = _var_name(Rules::annotation_name, context, annotation_name);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _terminal(b':');
	let closure_7 = _sequence(&closure_5, &closure_6);
	let closure_8 = _var_name(Rules::annotation_content, context, annotation_content);
	let closure_9 = _sequence(&closure_7, &closure_8);
	closure_9(parent, source, position)

} #[allow(dead_code)]
pub fn annotation_content<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _terminal(b'@');
	let closure_2 = _not_predicate(&closure_1);
	let closure_3 = _string_terminal_opt_ascii(&[b'*',b'*',b'>']);
	let closure_4 = _not_predicate(&closure_3);
	let closure_5 = _sequence(&closure_2, &closure_4);
	let closure_6 = move |parent: Key, source: &Source, position: u32| ascii(parent, context, source, position);
	let closure_7 = _sequence(&closure_5, &closure_6);
	let closure_8 = _subexpression(&closure_7);
	let closure_9 = _zero_or_more(&closure_8);
	closure_9(parent, source, position)

} #[allow(dead_code)]
pub fn annotation_name<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {
	// type char because same semantically and inlined anyway
	let closure_1 = move |parent: Key, source: &Source, position: u32| type_char(parent, context, source, position);
	let closure_2 = _one_or_more(&closure_1);
	closure_2(parent, source, position)

} #[allow(dead_code)]
pub fn type_char<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _terminal(b'_');
	let closure_2 = _ordered_choice_match_range(65, 90);
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	let closure_4 = _ordered_choice_match_range(97, 122);
	let closure_5 = _ordered_choice(&closure_3, &closure_4);
	closure_5(parent, source, position)

} #[allow(dead_code)]
pub fn type_char_with_num<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| digit(parent, context, source, position);
	let closure_2 = move |parent: Key, source: &Source, position: u32| type_char(parent, context, source, position);
	let closure_3 = _ordered_choice(&closure_1, &closure_2);
	closure_3(parent, source, position)

} #[allow(dead_code)]
pub fn type_name<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| type_char(parent, context, source, position);
	let closure_2 = move |parent: Key, source: &Source, position: u32| type_char_with_num(parent, context, source, position);
	let closure_3 = _zero_or_more(&closure_2);
	let closure_4 = _sequence(&closure_1, &closure_3);
	closure_4(parent, source, position)

} #[allow(dead_code)]
pub fn type_dec<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| type_name(parent, context, source, position);
	closure_1(parent, source, position)

} #[allow(dead_code)]
pub fn array<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_2 = _terminal(b'[');
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _terminal(b']');
	let closure_7 = _sequence(&closure_5, &closure_6);
	closure_7(parent, source, position)

} #[allow(dead_code)]
pub fn type_ref<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| type_name(parent, context, source, position);
	let closure_2 = _terminal(b'.');
	let closure_3 = move |parent: Key, source: &Source, position: u32| type_name(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _subexpression(&closure_4);
	let closure_6 = _zero_or_more(&closure_5);
	let closure_7 = _sequence(&closure_1, &closure_6);
	let closure_8 = _var_name(Rules::array, context, array);
	let closure_9 = _optional(&closure_8);
	let closure_10 = _sequence(&closure_7, &closure_9);
	closure_10(parent, source, position)

} #[allow(dead_code)]
pub fn variable_name<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| type_name(parent, context, source, position);
	closure_1(parent, source, position)

} #[allow(dead_code)]
pub fn file_path<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _terminal(b'"');
	let closure_2 = _terminal(b'"');
	let closure_3 = _not_predicate(&closure_2);
	let closure_4 = move |parent: Key, source: &Source, position: u32| ascii(parent, context, source, position);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _subexpression(&closure_5);
	let closure_7 = _zero_or_more(&closure_6);
	let closure_8 = _sequence(&closure_1, &closure_7);
	let closure_9 = _terminal(b'"');
	let closure_10 = _sequence(&closure_8, &closure_9);
	closure_10(parent, source, position)

} #[allow(dead_code)]
pub fn wildcard<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'.',b'*']);
	closure_1(parent, source, position)

} #[allow(dead_code)]
pub fn package<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {
	// Describes the package import
	let closure_1 = _string_terminal_opt_ascii(&[b'p',b'a',b'c',b'k',b'a',b'g',b'e']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| ws_atlone(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _var_name(Rules::type_ref, context, type_ref);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_7 = _sequence(&closure_5, &closure_6);
	closure_7(parent, source, position)

} #[allow(dead_code)]
pub fn import_namespace<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'i',b'm',b'p',b'o',b'r',b't']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| ws_atlone(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _var_name(Rules::type_ref, context, type_ref);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _var_name(Rules::wildcard, context, wildcard);
	let closure_7 = _sequence(&closure_5, &closure_6);
	let closure_8 = move |parent: Key, source: &Source, position: u32| ws_atlone(parent, context, source, position);
	let closure_9 = _sequence(&closure_7, &closure_8);
	let closure_10 = _string_terminal_opt_ascii(&[b'f',b'r',b'o',b'm']);
	let closure_11 = _sequence(&closure_9, &closure_10);
	let closure_12 = move |parent: Key, source: &Source, position: u32| ws_atlone(parent, context, source, position);
	let closure_13 = _sequence(&closure_11, &closure_12);
	let closure_14 = _var_name(Rules::file_path, context, file_path);
	let closure_15 = _sequence(&closure_13, &closure_14);
	let closure_16 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_17 = _sequence(&closure_15, &closure_16);
	closure_17(parent, source, position)

} #[allow(dead_code)]
pub fn import_model<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'i',b'm',b'p',b'o',b'r',b't']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| ws_atlone(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _string_terminal_opt_ascii(&[b'm',b'o',b'd',b'e',b'l']);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = move |parent: Key, source: &Source, position: u32| ws_atlone(parent, context, source, position);
	let closure_7 = _sequence(&closure_5, &closure_6);
	let closure_8 = _var_name(Rules::file_path, context, file_path);
	let closure_9 = _sequence(&closure_7, &closure_8);
	let closure_10 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_11 = _sequence(&closure_9, &closure_10);
	closure_11(parent, source, position)

} #[allow(dead_code)]
pub fn attribute<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b'a',b't',b't',b'r',b'i',b'b',b'u',b't',b'e']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| ws_atlone(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::type_ref, context, type_ref);
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| ws_atlone(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	let closure_13 = _var_name(Rules::variable_name, context, variable_name);
	let closure_14 = _sequence(&closure_12, &closure_13);
	let closure_15 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_16 = _sequence(&closure_14, &closure_15);
	closure_16(parent, source, position)

} #[allow(dead_code)]
pub fn variable_declaration<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _var_name(Rules::type_ref, context, type_ref);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::variable_name, context, variable_name);
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	closure_12(parent, source, position)

} #[allow(dead_code)]
pub fn input_params<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b'i',b'n']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _terminal(b'{');
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	let closure_13 = _var_name(Rules::variable_declaration, context, variable_declaration);
	let closure_14 = _zero_or_more(&closure_13);
	let closure_15 = _sequence(&closure_12, &closure_14);
	let closure_16 = _terminal(b'}');
	let closure_17 = _sequence(&closure_15, &closure_16);
	closure_17(parent, source, position)

} #[allow(dead_code)]
pub fn output_params<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b'o',b'u',b't']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _terminal(b'{');
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	let closure_13 = _var_name(Rules::variable_declaration, context, variable_declaration);
	let closure_14 = _zero_or_more(&closure_13);
	let closure_15 = _sequence(&closure_12, &closure_14);
	let closure_16 = _terminal(b'}');
	let closure_17 = _sequence(&closure_15, &closure_16);
	closure_17(parent, source, position)

} #[allow(dead_code)]
pub fn method<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b'm',b'e',b't',b'h',b'o',b'd']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::variable_name, context, variable_name);
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	let closure_13 = _terminal(b'{');
	let closure_14 = _sequence(&closure_12, &closure_13);
	let closure_15 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_16 = _sequence(&closure_14, &closure_15);
	let closure_17 = _var_name(Rules::input_params, context, input_params);
	let closure_18 = _optional(&closure_17);
	let closure_19 = _sequence(&closure_16, &closure_18);
	let closure_20 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_21 = _sequence(&closure_19, &closure_20);
	let closure_22 = _var_name(Rules::output_params, context, output_params);
	let closure_23 = _optional(&closure_22);
	let closure_24 = _sequence(&closure_21, &closure_23);
	let closure_25 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_26 = _sequence(&closure_24, &closure_25);
	let closure_27 = _terminal(b'}');
	let closure_28 = _sequence(&closure_26, &closure_27);
	closure_28(parent, source, position)

} #[allow(dead_code)]
pub fn typedef<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b't',b'y',b'p',b'e',b'd',b'e',b'f']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::type_dec, context, type_dec);
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	let closure_13 = _string_terminal_opt_ascii(&[b'i',b's']);
	let closure_14 = _sequence(&closure_12, &closure_13);
	let closure_15 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_16 = _sequence(&closure_14, &closure_15);
	let closure_17 = _var_name(Rules::type_ref, context, type_ref);
	let closure_18 = _sequence(&closure_16, &closure_17);
	closure_18(parent, source, position)

} #[allow(dead_code)]
pub fn structure<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b's',b't',b'r',b'u',b'c',b't']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::type_dec, context, type_dec);
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	let closure_13 = _terminal(b'{');
	let closure_14 = _sequence(&closure_12, &closure_13);
	let closure_15 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_16 = _sequence(&closure_14, &closure_15);
	let closure_17 = _var_name(Rules::variable_declaration, context, variable_declaration);
	let closure_18 = _zero_or_more(&closure_17);
	let closure_19 = _sequence(&closure_16, &closure_18);
	let closure_20 = _terminal(b'}');
	let closure_21 = _sequence(&closure_19, &closure_20);
	closure_21(parent, source, position)

} #[allow(dead_code)]
pub fn enumeration<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b'e',b'n',b'u',b'm',b'e',b'r',b'a',b't',b'i',b'o',b'n']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::type_dec, context, type_dec);
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	let closure_13 = _terminal(b'{');
	let closure_14 = _sequence(&closure_12, &closure_13);
	let closure_15 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_16 = _sequence(&closure_14, &closure_15);
	let closure_17 = _var_name(Rules::enum_value, context, enum_value);
	let closure_18 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_19 = _sequence(&closure_17, &closure_18);
	let closure_20 = _subexpression(&closure_19);
	let closure_21 = _zero_or_more(&closure_20);
	let closure_22 = _sequence(&closure_16, &closure_21);
	let closure_23 = _terminal(b'}');
	let closure_24 = _sequence(&closure_22, &closure_23);
	closure_24(parent, source, position)

} #[allow(dead_code)]
pub fn enum_value<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _var_name(Rules::variable_name, context, variable_name);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _terminal(b'=');
	let closure_10 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_11 = _sequence(&closure_9, &closure_10);
	let closure_12 = _var_name(Rules::number, context, number);
	let closure_13 = _sequence(&closure_11, &closure_12);
	let closure_14 = _subexpression(&closure_13);
	let closure_15 = _optional(&closure_14);
	let closure_16 = _sequence(&closure_8, &closure_15);
	let closure_17 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_18 = _sequence(&closure_16, &closure_17);
	let closure_19 = _terminal(b',');
	let closure_20 = _optional(&closure_19);
	let closure_21 = _sequence(&closure_18, &closure_20);
	closure_21(parent, source, position)

} #[allow(dead_code)]
pub fn version<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'v',b'e',b'r',b's',b'i',b'o',b'n']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _terminal(b'{');
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_7 = _sequence(&closure_5, &closure_6);
	let closure_8 = _var_name(Rules::major, context, major);
	let closure_9 = _sequence(&closure_7, &closure_8);
	let closure_10 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_11 = _sequence(&closure_9, &closure_10);
	let closure_12 = _var_name(Rules::minor, context, minor);
	let closure_13 = _sequence(&closure_11, &closure_12);
	let closure_14 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_15 = _sequence(&closure_13, &closure_14);
	let closure_16 = _terminal(b'}');
	let closure_17 = _sequence(&closure_15, &closure_16);
	closure_17(parent, source, position)

} #[allow(dead_code)]
pub fn major<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'm',b'a',b'j',b'o',b'r']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _var_name(Rules::digits, context, digits);
	let closure_5 = _sequence(&closure_3, &closure_4);
	closure_5(parent, source, position)

} #[allow(dead_code)]
pub fn minor<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _string_terminal_opt_ascii(&[b'm',b'i',b'n',b'o',b'r']);
	let closure_2 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = _var_name(Rules::digits, context, digits);
	let closure_5 = _sequence(&closure_3, &closure_4);
	closure_5(parent, source, position)

} #[allow(dead_code)]
pub fn interface<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b'i',b'n',b't',b'e',b'r',b'f',b'a',b'c',b'e']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::variable_name, context, variable_name);
	let closure_10 = _sequence(&closure_8, &closure_9);
	let closure_11 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_12 = _sequence(&closure_10, &closure_11);
	let closure_13 = _terminal(b'{');
	let closure_14 = _sequence(&closure_12, &closure_13);
	let closure_15 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_16 = _sequence(&closure_14, &closure_15);
	let closure_17 = _var_name(Rules::version, context, version);
	let closure_18 = _optional(&closure_17);
	let closure_19 = _sequence(&closure_16, &closure_18);
	let closure_20 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_21 = _sequence(&closure_19, &closure_20);
	let closure_22 = _var_name(Rules::method, context, method);
	let closure_23 = _var_name(Rules::typedef, context, typedef);
	let closure_24 = _ordered_choice(&closure_22, &closure_23);
	let closure_25 = _var_name(Rules::structure, context, structure);
	let closure_26 = _ordered_choice(&closure_24, &closure_25);
	let closure_27 = _var_name(Rules::attribute, context, attribute);
	let closure_28 = _ordered_choice(&closure_26, &closure_27);
	let closure_29 = _var_name(Rules::enumeration, context, enumeration);
	let closure_30 = _ordered_choice(&closure_28, &closure_29);
	let closure_31 = _subexpression(&closure_30);
	let closure_32 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_33 = _sequence(&closure_31, &closure_32);
	let closure_34 = _subexpression(&closure_33);
	let closure_35 = _zero_or_more(&closure_34);
	let closure_36 = _sequence(&closure_21, &closure_35);
	let closure_37 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_38 = _sequence(&closure_36, &closure_37);
	let closure_39 = _terminal(b'}');
	let closure_40 = _sequence(&closure_38, &closure_39);
	closure_40(parent, source, position)

} #[allow(dead_code)]
pub fn type_collection<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = _var_name(Rules::annotation_block, context, annotation_block);
	let closure_2 = _optional(&closure_1);
	let closure_3 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_4 = _sequence(&closure_2, &closure_3);
	let closure_5 = _string_terminal_opt_ascii(&[b't',b'y',b'p',b'e',b'C',b'o',b'l',b'l',b'e',b'c',b't',b'i',b'o',b'n']);
	let closure_6 = _sequence(&closure_4, &closure_5);
	let closure_7 = move |parent: Key, source: &Source, position: u32| ws(parent, context, source, position);
	let closure_8 = _sequence(&closure_6, &closure_7);
	let closure_9 = _var_name(Rules::variable_name, context, variable_name);
	let closure_10 = _optional(&closure_9);
	let closure_11 = _sequence(&closure_8, &closure_10);
	let closure_12 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_13 = _sequence(&closure_11, &closure_12);
	let closure_14 = _terminal(b'{');
	let closure_15 = _sequence(&closure_13, &closure_14);
	let closure_16 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_17 = _sequence(&closure_15, &closure_16);
	let closure_18 = _var_name(Rules::version, context, version);
	let closure_19 = _optional(&closure_18);
	let closure_20 = _sequence(&closure_17, &closure_19);
	let closure_21 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_22 = _sequence(&closure_20, &closure_21);
	let closure_23 = _var_name(Rules::typedef, context, typedef);
	let closure_24 = _var_name(Rules::structure, context, structure);
	let closure_25 = _ordered_choice(&closure_23, &closure_24);
	let closure_26 = _var_name(Rules::enumeration, context, enumeration);
	let closure_27 = _ordered_choice(&closure_25, &closure_26);
	let closure_28 = _subexpression(&closure_27);
	let closure_29 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_30 = _sequence(&closure_28, &closure_29);
	let closure_31 = _subexpression(&closure_30);
	let closure_32 = _zero_or_more(&closure_31);
	let closure_33 = _sequence(&closure_22, &closure_32);
	let closure_34 = _terminal(b'}');
	let closure_35 = _sequence(&closure_33, &closure_34);
	closure_35(parent, source, position)

} #[allow(dead_code)]
pub fn grammar<T: Context>(parent: Key, context: &RefCell<T>, source: &Source, position: u32) -> (bool, u32) {

	let closure_1 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_2 = _var_name(Rules::package, context, package);
	let closure_3 = _sequence(&closure_1, &closure_2);
	let closure_4 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_5 = _sequence(&closure_3, &closure_4);
	let closure_6 = _var_name(Rules::import_model, context, import_model);
	let closure_7 = _var_name(Rules::import_namespace, context, import_namespace);
	let closure_8 = _ordered_choice(&closure_6, &closure_7);
	let closure_9 = _subexpression(&closure_8);
	let closure_10 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_11 = _sequence(&closure_9, &closure_10);
	let closure_12 = _subexpression(&closure_11);
	let closure_13 = _zero_or_more(&closure_12);
	let closure_14 = _sequence(&closure_5, &closure_13);
	let closure_15 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_16 = _sequence(&closure_14, &closure_15);
	let closure_17 = _var_name(Rules::interface, context, interface);
	let closure_18 = _var_name(Rules::type_collection, context, type_collection);
	let closure_19 = _ordered_choice(&closure_17, &closure_18);
	let closure_20 = _subexpression(&closure_19);
	let closure_21 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_22 = _sequence(&closure_20, &closure_21);
	let closure_23 = _subexpression(&closure_22);
	let closure_24 = _zero_or_more(&closure_23);
	let closure_25 = _sequence(&closure_16, &closure_24);
	let closure_26 = move |parent: Key, source: &Source, position: u32| wsn(parent, context, source, position);
	let closure_27 = _sequence(&closure_25, &closure_26);
	closure_27(parent, source, position)

} 