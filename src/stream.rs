pub trait Stream<'a> {
    fn to_stream(&'a self) -> &'a [u8];
}

impl<'a> Stream<'a> for String {
    #[inline]
    fn to_stream(&'a self) -> &'a [u8] {
        self.as_bytes()
    }
}

impl<'a> Stream<'a> for &'a str {
    #[inline]
    fn to_stream(&'a self) -> &'a [u8] {
        self.as_bytes()
    }
}

impl<'a> Stream<'a> for Vec<u8> {
    #[inline]
    fn to_stream(&'a self) -> &'a [u8] {
        self.as_slice()
    }
}
