use std::marker::PhantomData;

use crate::machine::Command;
use crate::types::{
    AtomElim, AtomN, AtomP, BangIntro, Bot, BotElim, Coterm, Neg, One, Par, ParElim, Plus,
    PlusIntro, Pos, Tensor, Term, Whynot, WhynotElim, With, WithElim,
};

// =============================================================================
// μ and μ̃ binders (general continuations)
// =============================================================================

/// μ-binder constructor: `μα.c` on the positive side.
///
/// Builds a `Term::Mu` — a general continuation that receives a coterm
/// of the dual type.
pub fn mu<'s, A: Pos>(body: impl FnOnce(Coterm<'s, A::Dual>) -> Command<'s> + 's) -> Term<'s, A>
where
    A::Dual: Neg,
{
    Term::Mu(Box::new(body))
}

/// μ̃-binder constructor: `μ̃x.c` on the negative side.
///
/// Builds a `Coterm::MuTilde` — a general continuation that receives a
/// term of the dual type.
pub fn mu_tilde<'s, N: Neg>(
    body: impl FnOnce(Term<'s, N::Dual>) -> Command<'s> + 's,
) -> Coterm<'s, N>
where
    N::Dual: Pos,
{
    Coterm::MuTilde(Box::new(body))
}

// =============================================================================
// Positive structural constructors
// =============================================================================

/// Unit introduction: `()`.
#[allow(clippy::unused_unit)]
pub fn unit<'s>() -> Term<'s, One> {
    Term::Intro(())
}

/// Tensor introduction: `V ⊗ W`.
///
/// Takes anything convertible to `Term` (resources auto-wrap via `From<Resource>`)
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

/// Left injection for plus: `inl(V)`.
pub fn inl<'s, A: Pos, B: Pos>(v: impl Into<Term<'s, A>>) -> Term<'s, Plus<A, B>>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    Term::Intro(PlusIntro::Inl(v.into()))
}

/// Right injection for plus: `inr(W)`.
pub fn inr<'s, A: Pos, B: Pos>(w: impl Into<Term<'s, B>>) -> Term<'s, Plus<A, B>>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    Term::Intro(PlusIntro::Inr(w.into()))
}

// =============================================================================
// Negative elim constructors (μ̃-forms with structural patterns)
// =============================================================================

/// Atom elimination: `μ̃x.c` where `x` is a positive term.
///
/// Builds a `Coterm::Elim(AtomElim { body })`.
pub fn mu_atom<'s, X: 'static>(
    body: impl FnOnce(Term<'s, AtomP<X>>) -> Command<'s> + 's,
) -> Coterm<'s, AtomN<X>> {
    Coterm::Elim(AtomElim {
        body: Box::new(body),
    })
}

/// Bottom elimination: `μ̃().c`.
///
/// Builds a `Coterm::Elim(BotElim { body })`.
pub fn mu_unit<'s, F>(body: F) -> Coterm<'s, Bot>
where
    F: FnOnce() -> Command<'s> + 's,
{
    Coterm::Elim(BotElim {
        body: Box::new(body),
    })
}

/// Par elimination: `μ̃(x ⅋ y).c`.
///
/// Builds a `Coterm::Elim(ParElim { body })`.
pub fn mu_par<'s, A: Pos, B: Pos, F>(body: F) -> Coterm<'s, Par<A::Dual, B::Dual>>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F: FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s> + 's,
{
    Coterm::Elim(ParElim {
        body: Box::new(body),
    })
}

/// Case elimination: `μ̃case(x ⇒ c₁, y ⇒ c₂)`.
///
/// Builds a `Coterm::Elim(WithElim { left, right })`.
pub fn mu_case<'s, A: Pos, B: Pos, F1, F2>(
    body_left: F1,
    body_right: F2,
) -> Coterm<'s, With<A::Dual, B::Dual>>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F1: FnOnce(Term<'s, A>) -> Command<'s> + 's,
    F2: FnOnce(Term<'s, B>) -> Command<'s> + 's,
{
    Coterm::Elim(WithElim {
        left: Box::new(body_left),
        right: Box::new(body_right),
    })
}

/// Exponential elimination: `μ̃!x.c`.
///
/// Builds a `Coterm::Elim(WhynotElim { body })`.
pub fn mu_bang<'s, A: Pos, F>(body: F) -> Coterm<'s, Whynot<A::Dual>>
where
    A::Dual: Neg,
    F: FnOnce(BangIntro<'s, A>) -> Command<'s> + 's,
{
    Coterm::Elim(WhynotElim {
        body: Box::new(body),
    })
}

// =============================================================================
// Exponential helpers
// =============================================================================

/// Promote a closed computation to a classical (duplicable) term.
///
/// Takes a producer closure `Fn() -> Term<'s, A>` that can be invoked any
/// number of times, each time yielding a fresh linear term. The producer
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
