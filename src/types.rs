use std::marker::PhantomData;

use crate::machine::Command;
use crate::var::Var;

// =============================================================================
// 0.  Uniform term and coterm enums
// =============================================================================

/// A positive term at scope `'s`: either a variable or an introduction form.
pub enum Term<'s, A: Pos> {
    Var(Var<'s, A>),
    Intro(A::Intro<'s>),
}

/// A negative coterm at scope `'s`: either a variable or a binder body.
pub enum Coterm<'s, N: Neg> {
    Var(Var<'s, N>),
    Body(N::Body<'s>),
}

impl<'s, A: Pos> From<Var<'s, A>> for Term<'s, A> {
    fn from(v: Var<'s, A>) -> Self {
        Term::Var(v)
    }
}

impl<'s, N: Neg> From<Var<'s, N>> for Coterm<'s, N> {
    fn from(v: Var<'s, N>) -> Self {
        Coterm::Var(v)
    }
}

// =============================================================================
// 1.  Polarity traits
// =============================================================================

/// Positive types: atoms `X`, unit `1`, tensor `A ⊗ B`.
///
/// `Dual` computes the De Morgan dual (always in NNF).
/// `Intro<'s>` is the canonical introduction form at scope `'s`
/// (not including variables — atoms use `Infallible`).
pub trait Pos: Sized + 'static {
    /// The De Morgan dual — a negative type.
    type Dual: Neg<Dual = Self>;
    /// The concrete introduction form at scope `'s`.
    /// For atomic types this is `Infallible` (no intro form besides variables).
    type Intro<'s>;
}

/// Negative types: dual atoms `X⊥`, unit `⊥`, par `A ⅋ B`.
///
/// `Dual` computes the De Morgan dual.
/// `Body<'s>` is the binder body shape at scope `'s`
/// (not including variables).
pub trait Neg: Sized + 'static {
    /// The De Morgan dual — a positive type.
    type Dual: Pos<Dual = Self>;
    /// The concrete binder body shape at scope `'s`.
    type Body<'s>;
}

// =============================================================================
// 2.  Atoms
// =============================================================================

/// Positive atom `X`.
pub struct AtomP<X>(PhantomData<X>);

/// Negative atom `X⊥`.
pub struct AtomN<X>(PhantomData<X>);

impl<X: 'static> Pos for AtomP<X> {
    type Dual = AtomN<X>;
    /// Atoms have no introduction form besides variables.
    type Intro<'s> = std::convert::Infallible;
}

impl<X: 'static> Neg for AtomN<X> {
    type Dual = AtomP<X>;
    /// Body of `μx⁻.c`: receives a positive term.
    type Body<'s> = Box<dyn FnOnce(Term<'s, AtomP<X>>) -> Command<'s> + 's>;
}

// =============================================================================
// 3.  Multiplicative units
// =============================================================================

/// Positive unit `1`.
pub struct One;

/// Negative unit `⊥`.
pub struct Bot;

impl Pos for One {
    type Dual = Bot;
    type Intro<'s> = ();
}

impl Neg for Bot {
    type Dual = One;
    /// Body of `μ().c`: receives no argument.
    type Body<'s> = Box<dyn FnOnce() -> Command<'s> + 's>;
}

// =============================================================================
// 4.  Multiplicative connectives
// =============================================================================

/// Tensor `A ⊗ B` (both components positive).
pub struct Tensor<A: Pos, B: Pos>(PhantomData<(A, B)>);

/// Par `A ⅋ B` (both components negative).
pub struct Par<A: Neg, B: Neg>(PhantomData<(A, B)>);

impl<A: Pos, B: Pos> Pos for Tensor<A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    type Dual = Par<A::Dual, B::Dual>;
    /// Tensor introduction: a pair of terms.
    type Intro<'s> = (Term<'s, A>, Term<'s, B>);
}

impl<A: Neg, B: Neg> Neg for Par<A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type Dual = Tensor<A::Dual, B::Dual>;
    /// Body of `μ(x ⅋ y).c`: receives two terms.
    type Body<'s> = Box<dyn FnOnce(Term<'s, A::Dual>, Term<'s, B::Dual>) -> Command<'s> + 's>;
}

// =============================================================================
// 5.  Additive connectives
// =============================================================================

/// Positive sum `A ⊕ B` (choice made at introduction time).
pub struct Plus<A: Pos, B: Pos>(PhantomData<(A, B)>);

/// Negative with `A & B` (choice made at destruction time).
pub struct With<A: Neg, B: Neg>(PhantomData<(A, B)>);

/// Introduction form for `Plus<A, B>`: either left or right injection.
pub enum PlusIntro<'s, A: Pos, B: Pos> {
    Inl(Term<'s, A>),
    Inr(Term<'s, B>),
}

impl<A: Pos, B: Pos> Pos for Plus<A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    type Dual = With<A::Dual, B::Dual>;
    type Intro<'s> = PlusIntro<'s, A, B>;
}

/// Body form for `With<A, B>`: two continuations, one per injection.
pub struct WithBody<'s, A: Neg, B: Neg>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    pub left: Box<dyn FnOnce(Term<'s, A::Dual>) -> Command<'s> + 's>,
    pub right: Box<dyn FnOnce(Term<'s, B::Dual>) -> Command<'s> + 's>,
}

impl<A: Neg, B: Neg> Neg for With<A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type Dual = Plus<A::Dual, B::Dual>;
    type Body<'s> = WithBody<'s, A, B>;
}

// =============================================================================
// 6.  Exponential connectives
// =============================================================================

/// Positive exponential `!A` — duplicable terms.
pub struct Bang<A: Pos>(PhantomData<A>);

/// Negative exponential `?A` — dual of `!A`.
pub struct Whynot<N: Neg>(PhantomData<N>);

/// Introduction form for `Bang<A>`: a duplicable producer of terms.
pub struct BangIntro<'s, A: Pos> {
    pub(crate) producer: std::rc::Rc<dyn Fn() -> Term<'s, A> + 's>,
    pub(crate) _marker: PhantomData<&'s ()>,
}

impl<'s, A: Pos> Clone for BangIntro<'s, A> {
    fn clone(&self) -> Self {
        BangIntro {
            producer: self.producer.clone(),
            _marker: PhantomData,
        }
    }
}

impl<A: Pos> Pos for Bang<A>
where
    A::Dual: Neg,
{
    type Dual = Whynot<A::Dual>;
    type Intro<'s> = BangIntro<'s, A>;
}

impl<N: Neg> Neg for Whynot<N>
where
    N::Dual: Pos,
{
    type Dual = Bang<N::Dual>;
    /// Body of `μ!x.c`: receives a `BangIntro`.
    type Body<'s> = Box<dyn FnOnce(BangIntro<'s, N::Dual>) -> Command<'s> + 's>;
}
