
pub trait FidlContext {
}
pub mod MustHaveName {
    use u8 as UInt8;
    use i8 as Int8;
    use u16 as UInt16;
    use i16 as Int16;
    use u32 as UInt32;
    use i32 as Int32;
    use u64 as UInt64;
    use i64 as Int64;
    use f32 as Float;
    use f64 as Double;
    pub const VERSION_MAJOR: u32 = 0;
    pub const VERSION_MINOR: u32 = 0;
    use Int16 as aTypedef;
    pub struct thing { 
        p1: u8,
        p2: u8,
    }
    pub enum aEnum { 
        A = 3,
        B,
        C,
        D,
        E = 10,
    }
}