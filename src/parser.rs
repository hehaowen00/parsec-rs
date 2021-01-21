use core::fmt::Display;
use core::marker::PhantomData;
use core::ops::BitOr;

pub trait Parse<'a> {
    type Output;

    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]>;

    #[inline]
    fn map<B, F>(self, f: F) -> Map<'a, Self, F, Self::Output, B>
    where
        Self: Sized,
        F: Fn(Self::Output) -> B,
    {
        Map::new(self, f)
    }

    #[inline]
    fn or<P2>(self, p2: P2) -> Or<'a, Self, P2>
    where
        Self: Sized,
        P2: Parse<'a>,
    {
        Or::new(self, p2)
    }

    #[inline]
    fn then<P2>(self, p2: P2) -> (Self, P2)
    where
        Self: Sized,
        P2: Parse<'a>,
    {
        (self, p2)
    }

    #[inline]
    fn skip<P2>(self, p2: P2) -> Skip<'a, Self, P2>
    where
        Self: Sized,
        P2: Parse<'a>,
    {
        Skip::new(self, p2)
    }

    #[inline]
    fn skip_left<P2>(self, p2: P2) -> Skip<'a, P2, Self>
    where
        Self: Sized,
        P2: Parse<'a>,
    {
        Skip::new(p2, self)
    }
}

impl<'a, P1, P2, A, B> Parse<'a> for (P1, P2)
where
    P1: Parse<'a, Output = A>,
    P2: Parse<'a, Output = B>,
{
    type Output = (A, B);

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        let (input, a) = self.0.parse(input)?;
        let (input, b) = self.1.parse(input)?;

        Ok((input, (a, b)))
    }
}

pub struct Map<'a, P, F, A, B>
where
    P: Parse<'a, Output = A>,
    F: Fn(A) -> B + 'a,
{
    parser: P,
    f: F,
    marker: PhantomData<&'a ()>,
}

impl<'a, P, F, A, B> Map<'a, P, F, A, B>
where
    P: Parse<'a, Output = A>,
    F: Fn(A) -> B,
{
    #[inline]
    pub fn new(parser: P, f: F) -> Self {
        Self {
            parser,
            f,
            marker: PhantomData,
        }
    }
}

impl<'a, P, F, A, B> Parse<'a> for Map<'a, P, F, A, B>
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

pub struct Or<'a, P1, P2>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    parser1: P1,
    parser2: P2,
    marker: PhantomData<&'a ()>,
}
impl<'a, P1, P2> Or<'a, P1, P2>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    #[inline]
    pub fn new(parser1: P1, parser2: P2) -> Self {
        Self {
            parser1,
            parser2,
            marker: PhantomData,
        }
    }
}

impl<'a, P1, P2, O> Parse<'a> for Or<'a, P1, P2>
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

pub struct Many0<'a, P>
where
    P: Parse<'a>,
{
    parser: P,
    marker: PhantomData<&'a ()>,
}

impl<'a, P> Many0<'a, P>
where
    P: Parse<'a>,
{
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<'a, P> Parse<'a> for Many0<'a, P>
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

pub struct Many1<'a, P>
where
    P: Parse<'a>,
{
    parser: P,
    marker: PhantomData<&'a ()>,
}

impl<'a, P> Many1<'a, P>
where
    P: Parse<'a>,
{
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<'a, P> Parse<'a> for Many1<'a, P>
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

pub struct Skip<'a, P1, P2>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    parser1: P1,
    parser2: P2,
    marker: PhantomData<&'a ()>,
}

impl<'a, P1, P2> Skip<'a, P1, P2>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    #[inline]
    pub fn new(parser1: P1, parser2: P2) -> Self {
        Self {
            parser1,
            parser2,
            marker: PhantomData,
        }
    }
}

impl<'a, P1, P2> Parse<'a> for Skip<'a, P1, P2>
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

pub struct TakeUntil<'a, P>
where
    P: Parse<'a>,
{
    parser: P,
    marker: PhantomData<&'a ()>,
}

impl<'a, P> TakeUntil<'a, P>
where
    P: Parse<'a>,
{
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<'a, P> Parse<'a> for TakeUntil<'a, P>
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

pub struct Byte {
    byte: u8,
}

impl Byte {
    #[inline]
    pub fn new(byte: u8) -> Self {
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
    pub fn new(ch: char) -> Self {
        Self { ch }
    }
}

impl<'a> Parse<'a> for Char {
    type Output = &'a u8;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() == 0 {
            return Err(input);
        }

        if self.ch == input[0] as char {
            return Ok((&input[1..], &input[0]));
        }

        Err(input)
    }
}

pub struct Str<'a> {
    chars: Vec<char>,
    len: usize,
    marker: PhantomData<&'a ()>,
}

impl<'a> Str<'a> {
    #[inline]
    pub fn new<S>(s: S) -> Self
    where
        S: Display,
    {
        let chars: Vec<_> = s.to_string().chars().collect();
        let len = chars.len();

        Self {
            chars,
            len,
            marker: PhantomData,
        }
    }
}

impl<'a> Parse<'a> for Str<'a> {
    type Output = &'a str;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        if input.len() < self.len {
            return Err(input);
        }

        for idx in 0..self.len {
            if self.chars[idx] != input[idx] as char {
                return Err(input);
            }
        }

        let output = unsafe { std::str::from_utf8_unchecked(&input[0..self.len]) };

        Ok((&input[self.len..], output))
    }
}

pub struct State<'a, F, T>
where
    F: Fn() -> T,
{
    f: F,
    marker: PhantomData<&'a ()>,
}

impl<'a, F, T> State<'a, F, T>
where
    F: Fn() -> T,
{
    #[inline]
    pub fn new(f: F) -> Self {
        Self {
            f,
            marker: PhantomData,
        }
    }
}

impl<'a, F, T> Parse<'a> for State<'a, F, T>
where
    F: Fn() -> T,
{
    type Output = T;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        Ok((input, (self.f)()))
    }
}

impl<'a, A, B, C, T> BitOr<C> for Or<'a, A, B>
where
    A: Parse<'a, Output = T>,
    B: Parse<'a, Output = T>,
    C: Parse<'a, Output = T>,
{
    type Output = Or<'a, Or<'a, A, B>, C>;

    #[inline]
    fn bitor(self, rhs: C) -> Self::Output {
        self.or(rhs)
    }
}

impl<'a> BitOr for Str<'a> {
    type Output = Or<'a, Str<'a>, Str<'a>>;

    #[inline]
    fn bitor(self, rhs: Str<'a>) -> Self::Output {
        self.or(rhs)
    }
}