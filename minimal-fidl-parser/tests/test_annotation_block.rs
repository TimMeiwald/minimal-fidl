mod shared;
use shared::shared;
use minimal_fidl_parser::{annotation_block, BasicContext, Rules};
#[test]
fn test_annotation_block_1() {
    let src = "<** @description: Indicate end of playlist. **>";
    let result = shared(src, annotation_block::<BasicContext>, Rules::annotation_block);
    assert_eq!(result, (true, src.len() as u32));
}



#[test]
fn test_annotation_block_2() {
    let src = "<** @Annotation: block **>";
    let result = shared(src, annotation_block::<BasicContext>, Rules::annotation_block);
    assert_eq!(result, (true, src.len() as u32));
}