mod shared;
use shared::shared;
use minimal_fidl_parser::{type_collection, BasicContext, Rules};


#[test]
fn test_type_collection_1() {
    let src = r#"<**
	@description: This reference type collection uses all kinds of type definitions
	              which can be done within one type collection.
**>
typeCollection MyTypeCollection10 {}"#;
    let result = shared(src, type_collection::<BasicContext>, Rules::type_collection); 
    assert_eq!(result, (true, src.len() as u32));
}