use crate::parser::*;
use core::fmt::Display;
use core::marker::PhantomData;
use core::ops::BitOr;

pub struct Cell<'a, P>
where
    P: Parse<'a>,
{
    parser: P,
    marker: PhantomData<&'a ()>,
}

impl<'a, P> Cell<'a, P>
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

    #[inline]
    pub fn take(self) -> P {
        self.parser
    }

    #[inline]
    pub fn map<B, F>(self, f: F) -> Cell<'a, Map<'a, P, F, P::Output, B>>
    where
        F: Fn(P::Output) -> B,
    {
        Cell::new(Map::new(self.take(), f))
    }

    #[inline]
    pub fn or<RHS>(self, rhs: Cell<'a, RHS>) -> Cell<'a, Or<'a, P, RHS>>
    where
        RHS: Parse<'a, Output = P::Output>,
    {
        Cell::new(Or::new(self.take(), rhs.take()))
    }

    #[inline]
    pub fn then<RHS>(self, rhs: Cell<'a, RHS>) -> Cell<'a, And<'a, P, RHS, P::Output, RHS::Output>>
    where
        RHS: Parse<'a>,
    {
        Cell::new(And::new(self.take(), rhs.take()))
    }

    #[inline]
    pub fn skip<RHS>(self, rhs: Cell<'a, RHS>) -> Cell<'a, Skip<'a, P, RHS>>
    where
        RHS: Parse<'a>,
    {
        Cell::new(Skip::new(self.take(), rhs.take()))
    }

    #[inline]
    pub fn skip_left<RHS>(self, rhs: Cell<'a, RHS>) -> Cell<'a, Skip<'a, RHS, P>>
    where
        RHS: Parse<'a>,
    {
        Cell::new(Skip::new(rhs.take(), self.take()))
    }
}

impl<'a, P, RHS, O> BitOr<Cell<'a, RHS>> for Cell<'a, P>
where
    RHS: Parse<'a, Output = O>,
    P: Parse<'a, Output = O>,
{
    type Output = Cell<'a, Or<'a, P, RHS>>;

    #[inline]
    fn bitor(self, rhs: Cell<'a, RHS>) -> Self::Output {
        self.or(rhs)
    }
}

impl<'a, P> Parse<'a> for Cell<'a, P>
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
pub fn cell<'a, P>(parser: P) -> Cell<'a, P>
where
    P: Parse<'a>,
{
    Cell::new(parser)
}

#[inline]
pub fn state<'a, F, T>(f: F) -> Cell<'a, State<F, T>>
where
    F: Fn() -> T,
{
    Cell::new(State::new(f))
}

#[inline]
pub fn many0<'a, P>(parser: P) -> Cell<'a, Many0<'a, P>>
where
    P: Parse<'a>,
{
    Cell::new(Many0::new(parser))
}

#[inline]
pub fn many1<'a, P>(parser: P) -> Cell<'a, Many1<'a, P>>
where
    P: Parse<'a>,
{
    Cell::new(Many1::new(parser))
}

#[inline]
pub fn skip<'a, P1, P2>(p1: P1, p2: P2) -> Cell<'a, Skip<'a, P1, P2>>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    Cell::new(Skip::new(p1, p2))
}

#[inline]
pub fn skip_left<'a, P1, P2>(p1: P1, p2: P2) -> Cell<'a, Skip<'a, P2, P1>>
where
    P1: Parse<'a>,
    P2: Parse<'a>,
{
    Cell::new(Skip::new(p2, p1))
}

#[inline]
pub fn take_until<'a, P>(parser: P) -> Cell<'a, TakeUntil<'a, P>>
where
    P: Parse<'a>,
{
    Cell::new(TakeUntil::new(parser))
}

#[inline]
pub fn any_char<'a>() -> Cell<'a, AnyChar> {
    Cell::new(AnyChar::new())
}

#[inline]
pub fn any_digit<'a>() -> Cell<'a, AnyDigit> {
    Cell::new(AnyDigit::new())
}

#[inline]
pub fn byte<'a>(byte: u8) -> Cell<'a, Byte> {
    Cell::new(Byte::new(byte))
}

#[inline]
pub fn char_<'a>(ch: char) -> Cell<'a, Char> {
    Cell::new(Char::new(ch))
}

#[inline]
pub fn string<'a, S>(s: S) -> Cell<'a, Str<'a>>
where
    S: Display,
{
    Cell::new(Str::new(s))
}
