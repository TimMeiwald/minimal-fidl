use minimal_fidl_parser::{
    BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, grammar, RULES_SIZE,
};
mod indented_string;
mod formatter;
use formatter::Formatter;
use std::cell::RefCell;

pub fn parse(input: &str) -> Option<BasicPublisher> {
    let string = input.to_string();
    let src_len = string.len() as u32;
    let source = Source::new(&string);
    let position: u32 = 0;
    let result: (bool, u32);
    let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
    {
        let executor = _var_name(Rules::Grammar, &context, grammar);
        result = executor(Key(0), &source, position);
    }
    if result != (true, src_len) {
        println!("Failed with : {:?}", result);
        return None;
    }

    let publisher = context.into_inner().get_publisher().clear_false();
    Some(publisher)
}

#[cfg(test)]
mod tests {

    use crate::formatter;
    use crate::parse;
    use minimal_fidl_parser::{
        BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, grammar, RULES_SIZE,
    };
    use std::cell::RefCell;

    #[test]
    fn test_formatter_1() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { }	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_2() {
        let src = "// This do be a comment\npackage org.javaohjavawhyareyouso        // This do be a comment2\n
	interface endOfPlaylist { }	// This do be a comment\n
    interface endOfPlaylist { }	// This do be a comment\n";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_3() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { version {major 25 minor 60}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_4() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { version {major 25 minor 60 // Can comment \n}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_5() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { method thing {}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_6() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { method thing {in {param param param3 param3}}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_7() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { method thing {in {param param}  out {param2 param2 org.param3 param3}}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_8() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { method thing {in {param param}  out {param2 param2 org.param3 param3}}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_9() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { version {major 25 minor 60}method thing {in {param param}  out {param2 param2 org.param3 param3}}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_10() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { version {major 25 minor 60}method thing {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param}  out {param2 param2 org.param3 param3}}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_11() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{param20 param20}attribute uint8 thing\n method thing {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param}  out {param2 param2 org.param3 param3}}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_12() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{}attribute uint8 thing\n method thing {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param}  out {param2 param2 org.param3 param3}}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_13() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param}  out {param2 param2 org.param3 param3}}}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_14() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param} 
    out {param2 param2 org.param3 param3}} 	typedef aTypedef is Int16
}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_15() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param} 
    out {param2 param2 org.param3 param3}} 	typedef aTypedef is Int16 
	enumeration aEnum {
		A=3 B C ,D E =10
	}
}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_16() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param} 
    out {param2 param2 org.param3 param3}} 	typedef aTypedef is Int16 
	enumeration aEnum {
		A=3 B C ,D E =10
	}
}	typeCollection tc{}";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_17() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param} 
    out {param2 param2 org.param3 param3}} 	typedef aTypedef is Int16 
	enumeration aEnum {
		A=3 B C ,D E =10
	}
}	typeCollection tc{	typedef aTypedef is Int16 
	enumeration aEnum {
		A=3 B C ,D E =10
	}struct thing{p1 p1 p2 p2}}";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_18() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param} 
    out {param2 param2 org.param3 param3}} 	typedef aTypedef is Int16 
	enumeration aEnum {
		A=3 B C ,D E =10
	}
}	typeCollection{	typedef aTypedef is Int16 
	enumeration aEnum {
		A=3 B C ,D E =10
	}struct thing{p1 p1 p2 p2}}";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }

    #[test]
    fn test_formatter_19() {
        let src = r#"package org.javaohjavawhyareyouso
        import model "Astronomy_t.fidl"
        // This is a comment
	interface endOfPlaylist { }	"#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }
    #[test]
    fn test_formatter_20() {
        let src = r#"package org.javaohjavawhyareyouso // Comment
        import model "Astronomy_t.fidl" 
        import org.franca.omgidl.* from "OMGIDLBase.fidl" 
        // This is a comment
	interface endOfPlaylist { }	"#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }
    #[test]
    fn test_formatter_21() {
        let src = r#"package org.javaohjavawhyareyouso import model "Astronomy_t.fidl"
        import org.franca.omgidl.* from "OMGIDLBase.fidl" // This is a comment
        // This is a comment
	interface endOfPlaylist { }	"#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }
    #[test]
    fn test_formatter_22() {
        let src = r#"package org.javaohjavawhyareyouso 
        import model "Astronomy_t.fidl" // Comment
        import org.franca.omgidl.* from "OMGIDLBase.fidl" 
        // This is a comment
	interface endOfPlaylist { }	"#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = formatter::Formatter::new(src, &publisher);
        let output = fmt.format();
        println!("Formatted:\n\n{}", output.unwrap());
    }
}
