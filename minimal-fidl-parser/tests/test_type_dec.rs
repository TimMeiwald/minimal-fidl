use minimal_fidl_parser::{type_dec, BasicContext, Rules};
mod shared;
use shared::shared;

#[test]
fn test_number_1() {
    let src = r#"120"#;
    let result = shared(src, type_dec::<BasicContext>, Rules::type_dec);
    assert_eq!(result, (false, 0));
}
#[test]
fn test_number_2() {
    let src = r#"a"#;
    let result = shared(src, type_dec::<BasicContext>, Rules::type_dec);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_number_3() {
    let src = r#"A"#;
    let result = shared(src, type_dec::<BasicContext>, Rules::type_dec);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_number_4() {
    let src = r#"aaAA"#;
    let result = shared(src, type_dec::<BasicContext>, Rules::type_dec);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_number_5() {
    let src = r#"ThisisAType"#;
    let result = shared(src, type_dec::<BasicContext>, Rules::type_dec);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_number_6() {
    let src = r#"0ThisisAType"#;
    let result = shared(src, type_dec::<BasicContext>, Rules::type_dec);
    assert_eq!(result, (false, 0));
}
#[test]
fn test_number_7() {
    let src = r#"_0ThisisAType"#;
    let result = shared(src, type_dec::<BasicContext>, Rules::type_dec);
    assert_eq!(result, (true, src.len() as u32));
}
