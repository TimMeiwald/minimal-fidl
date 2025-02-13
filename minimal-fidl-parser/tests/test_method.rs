use minimal_fidl_parser::{method, BasicContext, Rules};
mod shared;
use shared::shared;

#[test]
fn test_method_1() {
    let src = r#"<** @description: Set current repeat mode. **>
	method setRepeatMode {
		in {
			RepeatMode mode // huijo
		}
	}"#;
    let result = shared(src, method::<BasicContext>, Rules::method);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_method_2() {
    let src = r#"method setRepeatMode {
		in {
			RepeatMode mode
		}
	}"#;
    let result = shared(src, method::<BasicContext>, Rules::method);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_method_3() {
    let src = r#"<** @description: Switch to the next track (if any). **>
     	method nextTrack { }"#;
    let result = shared(src, method::<BasicContext>, Rules::method);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_method_4() {
    let src = r#"method GetLowBattery {
		out {
			Boolean low_battery
		}
	}"#;
    let result = shared(src, method::<BasicContext>, Rules::method);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_method_5() {
    let src = r#"<**
    @description : Selects an entry of the list of available entries. The spelling process ends and the current location input state is updated accordingly
    @param : starts at the given index 
    @param : inputState returns the location input state
**>
method selectValueListEntry {
    in {
        UInt32 index
    }
    out {
        TInputState inputState
    }
}"#;
    let result = shared(src, method::<BasicContext>, Rules::method);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_method_6() {
    let src = r#"method StartUnit {
        in {
            String name
            String mode
        }
        out {
            String job // huijo
        }
    }"#;
    let result = shared(src, method::<BasicContext>, Rules::method);
    assert_eq!(result, (true, src.len() as u32));
}
