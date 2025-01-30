#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(pub u32);

impl From<Key> for usize {
    fn from(i: Key) -> usize {
        i.0 as usize
    }
}
