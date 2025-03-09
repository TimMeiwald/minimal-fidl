use serde::{Serialize, Deserialize};
use binary_serde::{binary_serde_bitfield, BinarySerde, Endianness};
pub mod Primitives {
    pub use u8 as UInt8;
    pub use i8 as Int8;
    pub use u16 as UInt16;
    pub use i16 as Int16;
    pub use u32 as UInt32;
    pub use i32 as Int32;
    pub use u64 as UInt64;
    pub use i64 as Int64;
    pub use f32 as Float;
    pub use f64 as Double;
}
pub trait FidlContext {
}
pub mod MyInterface {
    use super::Primitives::*;
    use super::*;
    use super::FidlContext;
    pub const VERSION_MAJOR: u32 = 0;
    pub const VERSION_MINOR: u32 = 0;
    use Double as CustomDouble;
    fn set_some_value(ctx: impl FidlContext, some_value: UInt8) { 
    }
    pub fn get_some_value() { 
    }
    pub fn thing(ctx: impl FidlContext, param: ThingStruct) -> (CustomDouble, Double) {
        (0.0, 0.0)
    }
    #[derive(Debug, Serialize, Deserialize, BinarySerde, PartialEq)]
    #[repr(C)]
    pub struct ThingStruct { 
        pub some_value: UInt16,
        pub some_value2: Float,
    }
    #[derive(Debug, Serialize, Deserialize, BinarySerde, PartialEq, Eq)]
    #[repr(u8)]
    enum aEnum { 
        A,
        B,
        C,
        D,
        E,
    }
}
