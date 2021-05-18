use crate::parser::*;

pub struct Cell<P> {
    parser: P,
}

impl<'a, P> Cell<P> {
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            parser,
        }
    }

    #[inline]
    pub fn take(self) -> P {
        self.parser
    }

    #[inline]
    pub fn map<B, F>(self, f: F) -> Cell<Map<P, F>>
    where
        F: Fn(P::Output) -> B,
        P: Parse<'a>
    {
        Cell::new(Map::new(self.take(), f))
    }

    #[inline]
    pub fn or<RHS>(self, rhs: Cell<RHS>) -> Cell<Or<P, RHS>>
    where
        RHS: Parse<'a, Output = P::Output>,
        P: Parse<'a>
    {
        Cell::new(Or::new(self.take(), rhs.take()))
    }

    #[inline]
    pub fn then<RHS>(self, rhs: Cell<RHS>) -> Cell<And<P, RHS>>
    where
        RHS: Parse<'a>,
    {
        Cell::new(And::new(self.take(), rhs.take()))
    }

    #[inline]
    pub fn skip<RHS>(self, rhs: Cell<RHS>) -> Cell<Skip<P, RHS>>
    where
        RHS: Parse<'a>,
    {
        Cell::new(Skip::new(self.take(), rhs.take()))
    }

    #[inline]
    pub fn skip_left<RHS>(self, rhs: Cell<RHS>) -> Cell<Skip<RHS, P>>
    where
        RHS: Parse<'a>,
    {
        Cell::new(Skip::new(rhs.take(), self.take()))
    }
}

impl<'a, P> Parse<'a> for Cell<P>
where
    P: Parse<'a>,
{
    type Output = P::Output;

    #[inline]
    fn parse(&self, input: &'a [u8]) -> Result<(&'a [u8], Self::Output), &'a [u8]> {
        self.parser.parse(input)
    }
}

#[inline]
pub fn cell<'a, P>(parser: P) -> Cell<P>
where
    P: Parse<'a>,
{
    Cell::new(parser)
}

#[inline]
pub fn state<'a, F, T>(f: F) -> Cell<State<F>>
where
    F: Fn() -> T,
{
    Cell::new(State::new(f))
}

#[inline]
pub fn many0<'a, P>(parser: P) -> Cell<Many0<P>>
where
    P: Parse<'a>,
{
    Cell::new(Many0::new(parser))
}

#[inline]
pub fn many1<'a, P>(parser: P) -> Cell<Many1<P>>
where
    P: Parse<'a>,
{
    Cell::new(Many1::new(parser))
}

#[inline]
pub fn skip<'a, P1, P2>(p1: P1, p2: P2) -> Cell<Skip<P1, P2>>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    Cell::new(Skip::new(p1, p2))
}

#[inline]
pub fn skip_left<'a, P1, P2>(p1: P1, p2: P2) -> Cell<Skip<P2, P1>>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    Cell::new(Skip::new(p2, p1))
}

#[inline]
pub fn take_until<'a, P>(parser: P) -> Cell<TakeUntil<P>>
where
    P: Parse<'a>,
{
    Cell::new(TakeUntil::new(parser))
}

#[inline]
pub fn any_char<'a>() -> Cell<AnyChar> {
    Cell::new(AnyChar::new())
}

#[inline]
pub fn any_digit<'a>() -> Cell<AnyDigit> {
    Cell::new(AnyDigit::new())
}

#[inline]
pub fn byte<'a>(byte: u8) -> Cell<Byte> {
    Cell::new(Byte::new(byte))
}

#[inline]
pub fn chr<'a>(ch: char) -> Cell<Char> {
    Cell::new(Char::new(ch))
}

#[inline]
pub fn slice<'a>(bytes: &[u8]) -> Cell<Slice> {
    Cell::new(Slice::new(bytes))
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.2"
))]
pub mod simd {
    use super::*;
    use crate::parser::simd::*;

    #[inline]
    pub fn take_until_literal<'a>(bytes: &[u8]) -> Cell<'a, TakeUntilLiteral> {
        Cell::new(TakeUntilLiteral::new(bytes))
    }
}
