use minimal_fidl_parser::{number, BasicContext, Rules};
mod shared;
use shared::shared;

#[test]
fn test_number_1() {
    let src = r#"120"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_number_2() {
    let src = r#"-120"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_number_3() {
    let src = r#"-"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (false, 0));
}

#[test]
fn test_number_4() {
    let src = r#"-120E5"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_number_5() {
    let src = r#"-120E-5"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_number_6() {
    let src = r#"-120E-525125"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_number_7() {
    let src = r#"0x10AF"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_number_8() {
    let src = r#"0x10af"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_number_9() {
    let src = r#"0x"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, 1)); // Number intepreted this as int for first val
}


#[test]
fn test_number_10() {
    let src = r#"0b10af"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, 4));
}

#[test]
fn test_number_11() {
    let src = r#"0b101010001010010"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true,  src.len() as u32));
}


#[test]
fn test_number_12() {
    let src = r#"- 120E-5"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}


#[test]
fn test_number_13() {
    let src = r#"- 120 E-5"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_number_14() {
    let src = r#"-   120   E-  5"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}
#[test]
fn test_number_15() {
    let src = r#"-120 E    -5"#;
    let result = shared(src, number::<BasicContext>, Rules::number);
    assert_eq!(result, (true, src.len() as u32));
}