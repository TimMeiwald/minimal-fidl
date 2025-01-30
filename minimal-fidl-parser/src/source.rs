pub struct Source<'a> {
    // Acts as immutable string since Source should never be modified.
    source: &'a str,
    source_len: u32,
}
impl<'a> From<Source<'a>> for String {
    fn from(i: Source) -> String {
        i.source.to_string()
    }
}
impl<'a> From<&Source<'a>> for String {
    fn from(i: &Source) -> String {
        i.source.to_string()
    }
}

impl<'a> Source<'a> {
    pub fn new(source: &'a str) -> Source<'a> {
        let source_len = source.len() as u32;
        Source { source, source_len }
    }

    pub fn get_char(&self, position: u32) -> Option<u8> {
        if position < self.source_len {
            println!("Pos: {:?}", position);
            let src_chr = self.source.as_bytes()[position as usize];
            Some(src_chr)
        } else {
            None
        }
    }

    pub fn get_multiple_chars(&self, position: u32, number_of_chars: u32) -> Option<&[u8]> {
        if position + number_of_chars < self.source_len {
            let src_chrs =
                &self.source.as_bytes()[position as usize..(position + number_of_chars) as usize];
            Some(src_chrs)
        } else {
            None
        }
    }

    pub fn get_len(&self) -> u32 {
        self.source_len
    }
}

#[cfg(test)]
mod tests {
    use crate::source::Source;
    #[test]
    fn test_source() {
        let string = "aaa".to_string();
        let source = Source::new(&string);
        if let Some(chr) = source.get_char(0) {
            assert_eq!(chr, "a".to_string().as_bytes()[0])
        }
        if let Some(chr) = source.get_char(1) {
            assert_eq!(chr, "a".to_string().as_bytes()[0])
        }
        if let Some(chr) = source.get_char(2) {
            assert_eq!(chr, "a".to_string().as_bytes()[0])
        }
        let s = source.get_char(3);
        assert_eq!(s, None);
    }
}
