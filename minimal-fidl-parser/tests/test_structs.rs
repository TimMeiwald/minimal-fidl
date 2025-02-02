use minimal_fidl_parser::{structure, BasicContext, Rules};
mod shared;
use shared::shared;

#[test]
fn test_structure_1() {
    let src = r#"<** @description: Duration in hours, minutes and seconds. **>
	struct Duration {
		UInt8 hours
		UInt8 minutes
		UInt8 seconds
	}"#;
    let result = shared(src, structure::<BasicContext>, Rules::structure);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_structure_2() {
    let src = r#"struct Duration {
		UInt8 hours
		UInt8 minutes
		UInt8 seconds
	}"#;
    let result = shared(src, structure::<BasicContext>, Rules::structure);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_structure_3() {
    let src = r#"struct Duration {
		UInt8 hours
		UInt8 minutes
		UInt8[] seconds
	}"#;
    let result = shared(src, structure::<BasicContext>, Rules::structure);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_structure_4() {
    let src = r#"	// empty struct (only allowed if polymorphic)
	struct MyStruct03 { }"#;
    let result = shared(src, structure::<BasicContext>, Rules::structure);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_structure_5() {
    let src = r#"
		// struct with elements of implicit array type
		struct MyStruct05 {
			UInt8[] se01
			String[] se02
			ByteBuffer[] se03
			MyArray01[] se10
			MyStruct02[] se11
			MyEnum03[] se12
		}"#;
    let result = shared(src, structure::<BasicContext>, Rules::structure);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_structure_6() {
    let src = r#"	
		// struct of enums
		struct MyStruct06 {
			MyEnum01 se01
			MyEnum02 se02
			MyEnum03 se03
			MyEnum10 se10
		}"#;
    let result = shared(src, structure::<BasicContext>, Rules::structure);
    assert_eq!(result, (true, src.len() as u32));
}
