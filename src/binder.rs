use std::marker::PhantomData;

use crate::expr::{CoExpr, Expr};
use crate::machine::Command;
use crate::types::{BangValue, Bot, Neg, Par, Pos, Tensor, Whynot, With};

// =============================================================================
// Binder types
// =============================================================================

/// Positive μ-binder `μ⁺α.c` at scope `'s`.
///
/// In classical System L, `μ⁺α.c` binds a covariable `α` of type `A::Dual`
/// (negative).  The body receives `A::Dual::CoValue<'s>` — the co-value
/// form for the dual of `A`.  For atomic types this is a negative `Var`.
pub struct MuPos<'s, A: Pos, F> {
    pub body: F,
    pub(crate) _marker: PhantomData<&'s ()>,
    pub(crate) _type: PhantomData<A>,
}

/// Negative μ-binder `μx⁻.c` at scope `'s`.
///
/// The body receives `N::Dual::Value<'s>` — the introduction form for the
/// dual of `N`.  For atomic negative types this is a positive variable.
pub struct MuNeg<'s, N: Neg, F> {
    pub body: F,
    pub(crate) _marker: PhantomData<&'s ()>,
    pub(crate) _type: PhantomData<N>,
}

/// Bottom destructor `μ().c` at scope `'s`.
pub struct MuUnit<'s, F> {
    pub body: F,
    pub(crate) _marker: PhantomData<&'s ()>,
}

/// Par destructor `μ(x ⅋ y).c` at scope `'s`.
///
/// The body receives `A::Value<'s>` and `B::Value<'s>` — the components
/// of the tensor value that triggered this reduction.  For atoms these are
/// variables; for composite types they are nested pairs.
pub struct MuPar<'s, A: Pos, B: Pos, F> {
    pub body: F,
    pub(crate) _marker: PhantomData<&'s ()>,
    pub(crate) _types: PhantomData<(A, B)>,
}

/// Case destructor `μcase(x ⇒ c₁, y ⇒ c₂)` at scope `'s`.
///
/// The binder carries two bodies, one for each injection.  When cut against
/// a `PlusValue`, the appropriate body is invoked with the injected value.
pub struct MuCase<'s, A: Pos, B: Pos, F1, F2> {
    pub body_left: F1,
    pub body_right: F2,
    pub(crate) _marker: PhantomData<&'s ()>,
    pub(crate) _types: PhantomData<(A, B)>,
}

/// Exponential destructor `μ!x.c` at scope `'s`.
///
/// The body receives a `BangValue<'s, A>` — a duplicable producer of
/// `A::Value<'s>`.  The body may clone the producer any number of times
/// (zero, one, many), or drop it without use.
pub struct MuBang<'s, A: Pos, F> {
    pub body: F,
    pub(crate) _marker: PhantomData<&'s ()>,
    pub(crate) _type: PhantomData<A>,
}

// =============================================================================
// Binder trait implementations
// =============================================================================

impl<'s, A: Pos, F> Expr<'s, A> for MuPos<'s, A, F>
where
    A::Dual: Neg,
    F: FnOnce(<A::Dual as Neg>::CoValue<'s>) -> Command<'s>,
{
}

impl<'s, N: Neg, F> CoExpr<'s, N> for MuNeg<'s, N, F>
where
    N::Dual: Pos,
    F: FnOnce(<N::Dual as Pos>::Value<'s>) -> Command<'s>,
{
}

impl<'s, F> CoExpr<'s, Bot> for MuUnit<'s, F> where F: FnOnce() -> Command<'s> {}

impl<'s, A: Pos, B: Pos, F> CoExpr<'s, Par<A::Dual, B::Dual>> for MuPar<'s, A, B, F>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F: FnOnce(A::Value<'s>, B::Value<'s>) -> Command<'s>,
{
}

impl<'s, A: Pos, B: Pos, F1, F2> CoExpr<'s, With<A::Dual, B::Dual>> for MuCase<'s, A, B, F1, F2>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F1: FnOnce(A::Value<'s>) -> Command<'s>,
    F2: FnOnce(B::Value<'s>) -> Command<'s>,
{
}

impl<'s, A: Pos, F> CoExpr<'s, Whynot<A::Dual>> for MuBang<'s, A, F>
where
    A::Dual: Neg,
    F: FnOnce(BangValue<'s, A>) -> Command<'s>,
{
}

// =============================================================================
// Constructors
// =============================================================================

/// Positive μ-binder: `μ⁺α.c`.
pub fn mu_pos<'s, A: Pos, F>(body: F) -> MuPos<'s, A, F>
where
    A::Dual: Neg,
    F: FnOnce(<A::Dual as Neg>::CoValue<'s>) -> Command<'s>,
{
    MuPos {
        body,
        _marker: PhantomData,
        _type: PhantomData,
    }
}

/// Negative μ-binder: `μx⁻.c`.
pub fn mu_neg<'s, N: Neg, F>(body: F) -> MuNeg<'s, N, F>
where
    N::Dual: Pos,
    F: FnOnce(<N::Dual as Pos>::Value<'s>) -> Command<'s>,
{
    MuNeg {
        body,
        _marker: PhantomData,
        _type: PhantomData,
    }
}

/// Unit introduction: `()`.
#[allow(clippy::unused_unit)]
pub fn unit() -> () {
    ()
}

/// Tensor introduction: `V ⊗ W`.
pub fn tensor<'s, A: Pos, B: Pos>(v: A::Value<'s>, w: B::Value<'s>) -> impl Expr<'s, Tensor<A, B>>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    (v, w)
}

/// Bottom destructor: `μ().c`.
pub fn mu_unit<'s, F>(body: F) -> MuUnit<'s, F>
where
    F: FnOnce() -> Command<'s>,
{
    MuUnit {
        body,
        _marker: PhantomData,
    }
}

/// Par destructor: `μ(x ⅋ y).c`.
pub fn mu_par<'s, A: Pos, B: Pos, F>(body: F) -> MuPar<'s, A, B, F>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F: FnOnce(A::Value<'s>, B::Value<'s>) -> Command<'s>,
{
    MuPar {
        body,
        _marker: PhantomData,
        _types: PhantomData,
    }
}

/// Case destructor: `μcase(x ⇒ c₁, y ⇒ c₂)`.
pub fn mu_case<'s, A: Pos, B: Pos, F1, F2>(
    body_left: F1,
    body_right: F2,
) -> MuCase<'s, A, B, F1, F2>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F1: FnOnce(A::Value<'s>) -> Command<'s>,
    F2: FnOnce(B::Value<'s>) -> Command<'s>,
{
    MuCase {
        body_left,
        body_right,
        _marker: PhantomData,
        _types: PhantomData,
    }
}

/// Exponential destructor: `μ!x.c`.
pub fn mu_bang<'s, A: Pos, F>(body: F) -> MuBang<'s, A, F>
where
    A::Dual: Neg,
    F: FnOnce(BangValue<'s, A>) -> Command<'s>,
{
    MuBang {
        body,
        _marker: PhantomData,
        _type: PhantomData,
    }
}

/// Promote a closed computation to a classical (duplicable) value.
///
/// Takes a producer closure `Fn() -> A::Value<'s>` that can be invoked any
/// number of times, each time yielding a fresh linear value.  The producer
/// must be closed (no free linear variables) — this is enforced by Rust's
/// move semantics on the closure.
pub fn promote<'s, A: Pos, F>(producer: F) -> BangValue<'s, A>
where
    A::Dual: Neg,
    F: Fn() -> A::Value<'s> + 's,
{
    BangValue {
        producer: std::rc::Rc::new(producer),
        _marker: PhantomData,
    }
}

/// Derelict: extract a linear value from a classical producer.
///
/// Invokes the producer once, yielding a fresh `A::Value<'s>`.
pub fn derelict<'s, A: Pos>(v: BangValue<'s, A>) -> A::Value<'s>
where
    A::Dual: Neg,
{
    (v.producer)()
}
