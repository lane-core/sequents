use std::marker::PhantomData;

use crate::expr::{CoExpr, Expr};
use crate::machine::Command;
use crate::types::{BangIntro, Bot, Coterm, Neg, One, Par, Pos, Tensor, Term, Whynot, With};

// =============================================================================
// Binder types
// =============================================================================

/// Positive μ-binder `μ⁺α.c` at scope `'s`.
///
/// In classical System L, `μ⁺α.c` binds a covariable `α` of type `A::Dual`
/// (negative).  The body receives `Coterm<'s, A::Dual>` — the coterm
/// form for the dual of `A`.
pub struct MuPos<'s, A: Pos, F> {
    pub body: F,
    pub(crate) _marker: PhantomData<&'s ()>,
    pub(crate) _type: PhantomData<A>,
}

/// Negative μ-binder `μx⁻.c` at scope `'s`.
///
/// The body receives `Term<'s, N::Dual>` — the term form for the
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
/// The body receives `Term<'s, A>` and `Term<'s, B>` — the components
/// of the tensor term that triggered this reduction.
pub struct MuPar<'s, A: Pos, B: Pos, F> {
    pub body: F,
    pub(crate) _marker: PhantomData<&'s ()>,
    pub(crate) _types: PhantomData<(A, B)>,
}

/// Case destructor `μcase(x ⇒ c₁, y ⇒ c₂)` at scope `'s`.
///
/// The binder carries two bodies, one for each injection.  When cut against
/// a `PlusIntro`, the appropriate body is invoked with the injected term.
pub struct MuCase<'s, A: Pos, B: Pos, F1, F2> {
    pub body_left: F1,
    pub body_right: F2,
    pub(crate) _marker: PhantomData<&'s ()>,
    pub(crate) _types: PhantomData<(A, B)>,
}

/// Exponential destructor `μ!x.c` at scope `'s`.
///
/// The body receives a `BangIntro<'s, A>` — a duplicable producer of
/// `Term<'s, A>`.  The body may clone the producer any number of times
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
    F: FnOnce(Coterm<'s, A::Dual>) -> Command<'s>,
{
}

impl<'s, N: Neg, F> CoExpr<'s, N> for MuNeg<'s, N, F>
where
    N::Dual: Pos,
    F: FnOnce(Term<'s, N::Dual>) -> Command<'s>,
{
}

impl<'s, F> CoExpr<'s, Bot> for MuUnit<'s, F> where F: FnOnce() -> Command<'s> {}

impl<'s, A: Pos, B: Pos, F> CoExpr<'s, Par<A::Dual, B::Dual>> for MuPar<'s, A, B, F>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F: FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s>,
{
}

impl<'s, A: Pos, B: Pos, F1, F2> CoExpr<'s, With<A::Dual, B::Dual>> for MuCase<'s, A, B, F1, F2>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F1: FnOnce(Term<'s, A>) -> Command<'s>,
    F2: FnOnce(Term<'s, B>) -> Command<'s>,
{
}

impl<'s, A: Pos, F> CoExpr<'s, Whynot<A::Dual>> for MuBang<'s, A, F>
where
    A::Dual: Neg,
    F: FnOnce(BangIntro<'s, A>) -> Command<'s>,
{
}

// =============================================================================
// Constructors
// =============================================================================

/// Positive μ-binder: `μ⁺α.c`.
pub fn mu_pos<'s, A: Pos, F>(body: F) -> MuPos<'s, A, F>
where
    A::Dual: Neg,
    F: FnOnce(Coterm<'s, A::Dual>) -> Command<'s>,
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
    F: FnOnce(Term<'s, N::Dual>) -> Command<'s>,
{
    MuNeg {
        body,
        _marker: PhantomData,
        _type: PhantomData,
    }
}

/// Unit introduction: `()`.
///
/// Returns a `Term<'s, One>` wrapping the unit value.
#[allow(clippy::unused_unit)]
pub fn unit<'s>() -> Term<'s, One> {
    Term::Intro(())
}

/// Tensor introduction: `V ⊗ W`.
///
/// Takes anything convertible to `Term` (variables auto-wrap via `From<Var>`)
/// and returns a `Term<'s, Tensor<A, B>>`.
pub fn tensor<'s, A: Pos, B: Pos>(
    v: impl Into<Term<'s, A>>,
    w: impl Into<Term<'s, B>>,
) -> Term<'s, Tensor<A, B>>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    Term::Intro((v.into(), w.into()))
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
    F: FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s>,
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
    F1: FnOnce(Term<'s, A>) -> Command<'s>,
    F2: FnOnce(Term<'s, B>) -> Command<'s>,
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
    F: FnOnce(BangIntro<'s, A>) -> Command<'s>,
{
    MuBang {
        body,
        _marker: PhantomData,
        _type: PhantomData,
    }
}

/// Promote a closed computation to a classical (duplicable) term.
///
/// Takes a producer closure `Fn() -> Term<'s, A>` that can be invoked any
/// number of times, each time yielding a fresh linear term.  The producer
/// must be closed (no free linear variables) — this is enforced by Rust's
/// move semantics on the closure.
pub fn promote<'s, A: Pos, F>(producer: F) -> BangIntro<'s, A>
where
    A::Dual: Neg,
    F: Fn() -> Term<'s, A> + 's,
{
    BangIntro {
        producer: std::rc::Rc::new(producer),
        _marker: PhantomData,
    }
}

/// Derelict: extract a linear term from a classical producer.
///
/// Invokes the producer once, yielding a fresh `Term<'s, A>`.
pub fn derelict<'s, A: Pos>(v: BangIntro<'s, A>) -> Term<'s, A>
where
    A::Dual: Neg,
{
    (v.producer)()
}
