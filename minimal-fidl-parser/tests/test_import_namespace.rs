use minimal_fidl_parser::{import_namespace, BasicContext, Rules};
mod shared;
use shared::shared;
#[test]
fn test_import_namespace_1() {
    let src = r#"import org.franca.omgidl.* from "OMGIDLBase.fidl"
    "#;
    let result = shared(src, import_namespace::<BasicContext>, Rules::import_namespace); 
    assert_eq!(result, (true, src.len() as u32));
}

