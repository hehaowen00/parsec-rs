pub trait Parse<'a> {
    type Output;

    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]>;
}

pub struct State<F> {
    f: F,
}

impl<F> State<F> {
    #[inline]
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<'a, F, T> Parse<'a> for State<F>
where
    F: Fn() -> T,
{
    type Output = T;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        Ok((input, (self.f)()))
    }
}

pub struct Map<P, F> {
    parser: P,
    f: F,
}

impl<P, F> Map<P, F> {
    #[inline]
    pub fn new(parser: P, f: F) -> Self {
        Self { parser, f }
    }
}

impl<'a, P, F, A, B> Parse<'a> for Map<P, F>
where
    P: Parse<'a, Output = A>,
    F: Fn(A) -> B,
{
    type Output = B;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        self.parser.parse(input).map(|(next, a)| {
            let b = (self.f)(a);
            (next, b)
        })
    }
}

pub struct And<P1, P2> {
    parser1: P1,
    parser2: P2,
}

impl<P1, P2> And<P1, P2> {
    #[inline]
    pub fn new(parser1: P1, parser2: P2) -> Self {
        Self {
            parser1,
            parser2,
        }
    }
}

impl<'a, P1, P2, A, B> Parse<'a> for And<P1, P2>
where
    P1: Parse<'a, Output = A>,
    P2: Parse<'a, Output = B>,
{
    type Output = (A, B);

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        let (input, a) = self.parser1.parse(input)?;
        let (input, b) = self.parser2.parse(input)?;

        Ok((input, (a, b)))
    }
}

pub struct Or<P1, P2> {
    parser1: P1,
    parser2: P2,
}

impl<P1, P2> Or<P1, P2> {
    #[inline]
    pub fn new(parser1: P1, parser2: P2) -> Self {
        Self {
            parser1,
            parser2,
        }
    }
}

impl<'a, P1, P2, O> Parse<'a> for Or<P1, P2>
where
    P1: Parse<'a, Output = O>,
    P2: Parse<'a, Output = O>,
{
    type Output = O;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        match self.parser1.parse(input) {
            res @ Ok(_) => res,
            Err(_) => self.parser2.parse(input),
        }
    }
}

pub struct Many0<P> {
    parser: P,
}

impl<P> Many0<P> {
    #[inline]
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<'a, P> Parse<'a> for Many0<P>
where
    P: Parse<'a>,
{
    type Output = Vec<P::Output>;

    #[inline]
    fn parse(&self, mut input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        let mut xs = Vec::new();

        while let Ok((next, item)) = self.parser.parse(input) {
            xs.push(item);
            input = next;
        }

        Ok((input, xs))
    }
}

pub struct Many1<P> {
    parser: P,
}

impl<P> Many1<P>
{
    #[inline]
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<'a, P> Parse<'a> for Many1<P>
where
    P: Parse<'a>,
{
    type Output = Vec<P::Output>;

    #[inline]
    fn parse(&self, mut input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        let mut xs = Vec::new();

        match self.parser.parse(input) {
            Ok((next, item)) => {
                xs.push(item);
                input = next;
            }
            Err(_) => {
                return Err(input);
            }
        }

        while let Ok((next, item)) = self.parser.parse(input) {
            xs.push(item);
            input = next;
        }

        return Ok((input, xs));
    }
}

pub struct Skip<P1, P2> {
    parser1: P1,
    parser2: P2,
}

impl<'a, P1, P2> Skip<P1, P2> {
    #[inline]
    pub fn new(parser1: P1, parser2: P2) -> Self {
        Self {
            parser1,
            parser2,
        }
    }
}

impl<'a, P1, P2> Parse<'a> for Skip<P1, P2>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    type Output = P1::Output;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        let (bytes, res) = self.parser1.parse(input)?;
        let (bytes, _) = self.parser2.parse(bytes)?;
        Ok((bytes, res))
    }
}

pub struct TakeUntil<P> {
    parser: P,
}

impl<P> TakeUntil<P> {
    #[inline]
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<'a, P> Parse<'a> for TakeUntil<P>
where
    P: Parse<'a>,
{
    type Output = &'a [u8];

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        let mut count = 0;
        let mut temp = input;

        while let Err(xs) = self.parser.parse(temp) {
            if temp.len() == 0 {
                return Err(input);
            }

            temp = &xs[1..];
            count += 1;
        }

        Ok((&input[count..], &input[0..count]))
    }
}

pub struct AnyChar;

impl AnyChar {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Parse<'a> for AnyChar {
    type Output = char;

    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        let ch = input[0] as char;

        match ch.is_ascii_alphabetic() {
            true => Ok((&input[1..], ch)),
            false => Err(input),
        }
    }
}

pub struct AnyDigit;

impl AnyDigit {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Parse<'a> for AnyDigit {
    type Output = char;

    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        let digit = input[0] as char;

        match digit.is_numeric() {
            true => Ok((&input[1..], digit)),
            false => Err(input),
        }
    }
}

pub struct Byte {
    byte: u8,
}

impl Byte {
    #[inline]
    pub fn new<'a>(byte: u8) -> Self {
        Self { byte }
    }
}

impl<'a> Parse<'a> for Byte {
    type Output = &'a u8;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        match input[0] == self.byte {
            true => Ok((&input[1..], &input[0])),
            false => Err(input),
        }
    }
}

pub struct Char {
    ch: char,
}

impl Char {
    #[inline]
    pub fn new<'a>(ch: char) -> Self {
        Self { ch }
    }
}

impl<'a> Parse<'a> for Char {
    type Output = char;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        let ch = input[0] as char;

        match self.ch == ch {
            true => Ok((&input[1..], ch)),
            false => Err(input),
        }
    }
}

pub struct Slice {
    bytes: Box<[u8]>,
}

impl Slice {
    pub fn new(slice: &[u8]) -> Self {
        Self {
            bytes: slice.into(),
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl<'a> Parse<'a> for Slice {
    type Output = &'a [u8];

    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() < self.len() {
            return Err(input);
        }

        for idx in 0..self.len() {
            if self.bytes[idx] != input[idx] {
                return Err(input);
            }
        }
        let output = &input[0..self.len()];

        Ok((&input[self.len()..], output))
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.2"
))]
pub mod simd {
    use super::*;
    use crate::simd;

    pub struct Slice {
        bytes: Box<[u8]>,
    }

    impl Slice {
        pub fn new(bytes: &[u8]) -> Self {
            assert!(bytes.len() < 16);
            Self {
                bytes: bytes.into(),
            }
        }

        pub fn len(&self) -> usize {
            self.bytes.len()
        }
    }

    impl<'a> Parse<'a> for Slice {
        type Output = &'a [u8];

        fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
            if input.len() < self.len() {
                return Err(input);
            }

            if let None = simd::compare(&input[0..self.len()], &self.bytes) {
                return Err(input);
            }

            let output = &input[0..self.len()];

            Ok((&input[self.len()..], output))
        }
    }

    pub struct TakeUntilLiteral {
        bytes: Box<[u8]>,
    }

    impl TakeUntilLiteral {
        pub fn new(bytes: &[u8]) -> Self {
            assert!(bytes.len() < 16);
            Self {
                bytes: bytes.to_vec().into_boxed_slice(),
            }
        }
    }

    impl<'a> Parse<'a> for TakeUntilLiteral {
        type Output = &'a [u8];

        fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
            match simd::compare(input, &self.bytes) {
                Some(idx) => Ok((&input[idx..], &input[0..idx])),
                None => Err(input),
            }
        }
    }
}

