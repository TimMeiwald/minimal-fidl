use minimal_fidl_parser::{import_model, BasicContext, Rules};
mod shared;
use shared::shared;
#[test]
fn test_import_model_1() {
    let src = r#"import model "csm_t.fidl""#;
    let result = shared(src, import_model::<BasicContext>, Rules::import_model); 
    assert_eq!(result, (true, src.len() as u32));
}
