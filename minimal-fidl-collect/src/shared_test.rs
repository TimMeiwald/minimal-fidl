use minimal_fidl_parser::{_var_name, BasicContext, BasicPublisher, Context, Key, Node, RULES_SIZE, Rules, Source};
use std::cell::RefCell;
pub fn shared<'a>(
    input: &str,
    func: for<'c> fn(Key, &RefCell<BasicContext>, &Source<'c>, u32) -> (bool, u32),
    rule: Rules,
) -> (&str, BasicPublisher, Key) {
    let string = input.to_string();
    let src_len = string.len() as u32;
    let source = Source::new(&string);
    let position: u32 = 0;
    let result: (bool, u32);
    let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
    {
        let executor = _var_name(rule, &context, func);
        result = executor(Key(0), &source, position);
    }
    println!("Result: {:?}", result);
    //context.borrow().print_cache();
    //context.borrow().print_publisher();
    //context.borrow().print_node(Key(0));
    let publisher = context.into_inner().get_publisher().clear_false();
    publisher.print(Key(0), Some(true));
    assert_eq!(src_len, result.1, "Tests in minimal-fidl-collect assume successful parsing");
    assert!(result.0, "Tests in minimal-fidl-collect assume successful parsing");
    assert_eq!(publisher.get_node(Key(0)).get_children().len(), 1, "Root node is grammar, we assume only one node is created.");

    // Get the nodes key so we can use it for testing.
    let node_key = publisher.get_node(Key(0)).get_children()[0];
    (input, publisher, node_key)
}
