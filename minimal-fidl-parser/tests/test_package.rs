use minimal_fidl_parser::{package, BasicContext, Rules};
mod shared;
use shared::shared;
#[test]
fn test_package_1() {
    let src = r#"package org.reference     "#;
    let result = shared(src, package::<BasicContext>, Rules::package); 
    assert_eq!(result, (true, src.len() as u32));
}
