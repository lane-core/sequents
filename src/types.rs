use std::marker::PhantomData;

use crate::var::Var;

// =============================================================================
// 1.  Polarity traits
// =============================================================================

/// Positive types: atoms `X`, unit `1`, tensor `A ⊗ B`.
///
/// `Dual` computes the De Morgan dual (always in NNF).  `Value<'s>` is the
/// type of introduction forms at scope `'s`.
pub trait Pos: Sized + 'static {
    /// The De Morgan dual — a negative type.
    type Dual: Neg<Dual = Self>;
    /// The concrete introduction form at scope `'s`.
    type Value<'s>;
}

/// Negative types: dual atoms `X⊥`, unit `⊥`, par `A ⅋ B`.
///
/// `Dual` computes the De Morgan dual.  `CoValue<'s>` is the type of
/// co-value forms at scope `'s`.
pub trait Neg: Sized + 'static {
    /// The De Morgan dual — a positive type.
    type Dual: Pos<Dual = Self>;
    /// The concrete co-value form at scope `'s`.
    type CoValue<'s>;
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
    /// The variable token *is* the atomic value.
    type Value<'s> = Var<'s, AtomP<X>>;
}

impl<X: 'static> Neg for AtomN<X> {
    type Dual = AtomP<X>;
    /// The covariable token *is* the atomic co-value.
    type CoValue<'s> = Var<'s, AtomN<X>>;
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
    type Value<'s> = ();
}

/// Co-value form for `Bot` (negative unit).
///
/// In the principal-cut fragment, `Bot` has no user-constructible co-value.
/// With commuting conversions, a `μ().c` binder can appear as a co-value,
/// represented as a type-erased continuation.
pub enum BotCoValue<'s> {
    Cont(Box<dyn FnOnce() -> crate::machine::Command<'s> + 's>),
}

impl Neg for Bot {
    type Dual = One;
    type CoValue<'s> = BotCoValue<'s>;
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
    /// Crucial line: the tensor value lives in the intersection of its
    /// components' lifetimes, computed automatically by Rust's covariance.
    type Value<'s> = (A::Value<'s>, B::Value<'s>);
}

/// Co-value form for `Par<A, B>`.
///
/// Names the existential quantification over `MuPar` body closures,
/// allowing a par destructor to be substituted as a co-value in
/// commuting conversions.
#[allow(clippy::type_complexity)]
pub enum ParCoValue<'s, A: Neg, B: Neg> {
    Cont(
        Box<
            dyn FnOnce(
                    <A::Dual as Pos>::Value<'s>,
                    <B::Dual as Pos>::Value<'s>,
                ) -> crate::machine::Command<'s>
                + 's,
        >,
    ),
}

impl<A: Neg, B: Neg> Neg for Par<A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type Dual = Tensor<A::Dual, B::Dual>;
    type CoValue<'s> = ParCoValue<'s, A, B>;
}

// =============================================================================
// 5.  Additive connectives
// =============================================================================

/// Positive sum `A ⊕ B` (choice made at introduction time).
pub struct Plus<A: Pos, B: Pos>(PhantomData<(A, B)>);

/// Negative with `A & B` (choice made at destruction time).
pub struct With<A: Neg, B: Neg>(PhantomData<(A, B)>);

/// Value of a sum type: either left or right injection.
pub enum PlusValue<'s, A: Pos, B: Pos> {
    Inl(A::Value<'s>),
    Inr(B::Value<'s>),
}

impl<A: Pos, B: Pos> Pos for Plus<A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    type Dual = With<A::Dual, B::Dual>;
    type Value<'s> = PlusValue<'s, A, B>;
}

/// Co-value form for `With<A, B>`.
///
/// Names the existential quantification over `MuCase` body closures,
/// allowing a case destructor to be substituted as a co-value in
/// commuting conversions.  Carries two continuations, one per injection.
#[allow(clippy::type_complexity)]
pub enum WithCoValue<'s, A: Neg, B: Neg> {
    Cont {
        left: Box<dyn FnOnce(<A::Dual as Pos>::Value<'s>) -> crate::machine::Command<'s> + 's>,
        right: Box<dyn FnOnce(<B::Dual as Pos>::Value<'s>) -> crate::machine::Command<'s> + 's>,
    },
}

impl<A: Neg, B: Neg> Neg for With<A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type Dual = Plus<A::Dual, B::Dual>;
    type CoValue<'s> = WithCoValue<'s, A, B>;
}

// =============================================================================
// 6.  Exponential connectives
// =============================================================================

/// Positive exponential `!A` — duplicable values.
///
/// `Bang<A>` is a positive type whose values can be cloned and discarded.
/// The underlying value is produced on demand via a closure, so each use
/// obtains a fresh linear value.
pub struct Bang<A: Pos>(PhantomData<A>);

/// Negative exponential `?A` — dual of `!A`.
pub struct Whynot<N: Neg>(PhantomData<N>);

/// The value of `!A` at scope `'s`: a duplicable producer of `A::Value<'s>`.
///
/// Each invocation of the producer yields a fresh linear value.  The `Rc`
/// wrapper provides `Clone` at zero additional cost beyond the reference
/// count increment.
pub struct BangValue<'s, A: Pos> {
    pub(crate) producer: std::rc::Rc<dyn Fn() -> A::Value<'s> + 's>,
    pub(crate) _marker: PhantomData<&'s ()>,
}

impl<'s, A: Pos> Clone for BangValue<'s, A> {
    fn clone(&self) -> Self {
        BangValue {
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
    type Value<'s> = BangValue<'s, A>;
}

/// Co-value form for `Whynot<N>`.
///
/// Names the existential quantification over `MuBang` body closures,
/// allowing an exponential destructor to be substituted as a co-value in
/// commuting conversions.
pub enum WhynotCoValue<'s, N: Neg> {
    Cont(Box<dyn FnOnce(BangValue<'s, N::Dual>) -> crate::machine::Command<'s> + 's>),
}

impl<N: Neg> Neg for Whynot<N>
where
    N::Dual: Pos,
{
    type Dual = Bang<N::Dual>;
    type CoValue<'s> = WhynotCoValue<'s, N>;
}
