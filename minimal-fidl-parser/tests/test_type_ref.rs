use minimal_fidl_parser::{type_ref, BasicContext, Rules};
mod shared;
use shared::shared;

#[test]
fn test_number_1() {
    let src = r#"ref_at.120"#;
    let result = shared(src, type_ref::<BasicContext>, Rules::type_ref);
    assert_eq!(result, (true, 6));
}
#[test]
fn test_number_2() {
    let src = r#"valid_ref.thg"#;
    let result = shared(src, type_ref::<BasicContext>, Rules::type_ref);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_number_3() {
    let src = r#"valid839ref.thah_a.hafiawijfa88"#;
    let result = shared(src, type_ref::<BasicContext>, Rules::type_ref);
    assert_eq!(result, (true, src.len() as u32));
}