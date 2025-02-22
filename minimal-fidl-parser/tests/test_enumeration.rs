use minimal_fidl_parser::{enumeration, BasicContext, Rules};
mod shared;
use shared::shared;

#[test]
fn test_enumeration_1() {
    let src = r#"<** @description : Repeat modes for playback. **>
	enumeration RepeatMode {
		MODE_REPEAT_NONE   = 0
		MODE_REPEAT_SINGLE = 1
		MODE_REPEAT_ALL    = 2
	}"#;
    let result = shared(src, enumeration::<BasicContext>, Rules::enumeration);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_enumeration_2() {
    let src = r#"<** @description : Repeat modes for playback. **>
	enumeration RepeatMode {
		MODE_REPEAT_NONE
		MODE_REPEAT_SINGLE
		MODE_REPEAT_ALL
	}"#;
    let result = shared(src, enumeration::<BasicContext>, Rules::enumeration);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_enumeration_3() {
    let src = r#"enumeration RepeatMode {
		MODE_REPEAT_NONE
		MODE_REPEAT_SINGLE
		MODE_REPEAT_ALL
	}"#;
    let result = shared(src, enumeration::<BasicContext>, Rules::enumeration);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_enumeration_4() {
    let src = r#"<** @description : Repeat modes for playback. **>
	enumeration RepeatMode {
		MODE_REPEAT_NONE
		MODE_REPEAT_SINGLE
		MODE_REPEAT_ALL
	}"#;
    let result = shared(src, enumeration::<BasicContext>, Rules::enumeration);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_enumeration_5() {
    // Below test obviously doesn't make any sense
    // But it will read as a float in the 2nd case
    // So it'll trivially cause an error if the parser
    // Correctly only handles integers for enum values
    let src = r#"<** @description : Repeat modes for playback. **>
	enumeration RepeatMode {
		MODE_REPEAT_NONE   = 20.5E-5
		MODE_REPEAT_SINGLE = -505.2492140941
		MODE_REPEAT_ALL    = 2
	}"#;
    let result = shared(src, enumeration::<BasicContext>, Rules::enumeration);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_enumeration_6() {
    let src = r#"<** @description : Repeat modes for playback. **>
	enumeration RepeatMode {
		A=0b10100010 C D E, F
	}"#;
    let result = shared(src, enumeration::<BasicContext>, Rules::enumeration);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_enumeration_7() {
    let src = r#"<** @description : Repeat modes for playback. **>
	enumeration RepeatMode {
		A C D E F=0x40
	}"#;
    let result = shared(src, enumeration::<BasicContext>, Rules::enumeration);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_enumeration_8() {
    let src = r#"<** @description : Repeat modes for playback. **>
	enumeration aEnum {
		A = 3
		B = 0x004000
		C = 0b0101001
		D
		E = 10
	}"#;
    let result = shared(src, enumeration::<BasicContext>, Rules::enumeration);
    assert_eq!(result, (true, src.len() as u32));
}