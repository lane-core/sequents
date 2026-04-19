use std::marker::PhantomData;

use crate::machine::Command;
use crate::types::{
    AtomElim, AtomN, AtomP, BangIntro, Bot, BotElim, Coterm, Negative, One, Par, ParElim, Plus,
    PlusIntro, Positive, Tensor, Term, Whynot, WhynotElim, With, WithElim,
};

// =============================================================================
// μ and μ̃ binders (general continuations)
// =============================================================================

/// μ-binder constructor: `μα.c` on the positive side.
///
/// Builds a [`Term::Mu`] — a general continuation that receives a coterm
/// of the dual type. In the λμ̃μ-calculus, `μα.c` binds a covariable
/// `α` and captures the current evaluation context as a first-class
/// object `Grokking §3.2].
///
/// This is the control operator on the positive side. When `cut` pairs
/// a `Term::Mu` with any coterm, the μ-body consumes that coterm
/// directly — this is the μ-reduction rule [Spiwack, `mu`].
///
/// # Example
/// ```
/// use sequents::*;
///
/// struct X;
/// let x: Resource<'static, AtomP<X>> = Resource::new();
///
/// let pos = mu::<'static, AtomP<X>>(|a: Coterm<'_, AtomN<X>>| {
///     // a is the coterm bound to α
///     cut(Term::Axiom(x), a)
/// });
/// ```
pub fn mu<'s, A: Positive>(
    body: impl FnOnce(Coterm<'s, A::Dual>) -> Command<'s> + 's,
) -> Term<'s, A>
where
    A::Dual: Negative,
{
    Term::Mu(Box::new(body))
}

/// μ̃-binder constructor: `μ̃x.c` on the negative side.
///
/// Builds a [`Coterm::MuTilde`] — a general continuation that receives a
/// term of the dual type. In the λμ̃μ-calculus, `μ̃x.c` binds a variable
/// `x` and captures the current term as a first-class object
/// `Grokking §3.2].
///
/// This is the control operator on the negative side. When `cut` pairs
/// any term with a `Coterm::MuTilde`, the μ̃-body consumes that term
/// directly — this is the μ̃-reduction rule [Spiwack, `mu`].
///
/// # Example
/// ```
/// use sequents::*;
///
/// struct X;
/// let z: Resource<'static, AtomN<X>> = Resource::new();
///
/// let coterm = mu_tilde::<'static, AtomN<X>>(|t: Term<'_, AtomP<X>>| {
///     // t is the term bound to x
///     cut(t, z.into())
/// });
/// ```
pub fn mu_tilde<'s, N: Negative>(
    body: impl FnOnce(Term<'s, N::Dual>) -> Command<'s> + 's,
) -> Coterm<'s, N>
where
    N::Dual: Positive,
{
    Coterm::MuTilde(Box::new(body))
}

// =============================================================================
// Positive structural constructors
// =============================================================================

/// Unit introduction: `()`.
///
/// Returns a `Term<'s, One>` wrapping the unit value.
/// The introduction rule for the multiplicative unit `1`
/// `MMM §7, rule (R1)].
#[allow(clippy::unused_unit)]
pub fn unit<'s>() -> Term<'s, One> {
    Term::Intro(())
}

/// Tensor introduction: `V ⊗ W`.
///
/// Takes anything convertible to `Term` (resources auto-wrap via
/// [`From<Resource>`]) and returns a `Term<'s, Tensor<A, B>>`.
/// The introduction rule for tensor pairs `MMM §7, rule (R⊗)].
///
/// # Example
/// ```
/// use sequents::*;
///
/// struct X;
/// struct Y;
///
/// let x: Resource<'static, AtomP<X>> = Resource::new();
/// let y: Resource<'static, AtomP<Y>> = Resource::new();
/// let pair = tensor(x, y); // Term<Tensor<AtomP<X>, AtomP<Y>>>
/// ```
pub fn tensor<'s, A: Positive, B: Positive>(
    v: impl Into<Term<'s, A>>,
    w: impl Into<Term<'s, B>>,
) -> Term<'s, Tensor<A, B>>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    Term::Intro((v.into(), w.into()))
}

/// Left injection for plus: `inl(V)`.
///
/// The left introduction rule for the additive disjunction `A ⊕ B`.
/// Choice is made at construction time [Spiwack, `iota1`].
pub fn inl<'s, A: Positive, B: Positive>(v: impl Into<Term<'s, A>>) -> Term<'s, Plus<A, B>>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    Term::Intro(PlusIntro::Inl(v.into()))
}

/// Right injection for plus: `inr(W)`.
///
/// The right introduction rule for the additive disjunction `A ⊕ B`.
/// Choice is made at construction time [Spiwack, `iota2`].
pub fn inr<'s, A: Positive, B: Positive>(w: impl Into<Term<'s, B>>) -> Term<'s, Plus<A, B>>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    Term::Intro(PlusIntro::Inr(w.into()))
}

// =============================================================================
// Negative elim constructors (μ̃-forms with structural patterns)
// =============================================================================

/// Atom elimination: `μ̃x.c` where `x` is a positive term.
///
/// Builds a [`Coterm::Elim`] wrapping an [`AtomElim`]. The body receives
/// a [`Term<'s, AtomP<X>>`] — the atomic μ̃-reduction [Spiwack, `mu`].
///
/// This is the specific destructor for atomic negative types. For
/// non-atomic types, use [`mu_tilde`] to build a general μ̃-binder.
pub fn mu_atom<'s, X: 'static>(
    body: impl FnOnce(Term<'s, AtomP<X>>) -> Command<'s> + 's,
) -> Coterm<'s, AtomN<X>> {
    Coterm::Elim(AtomElim {
        body: Box::new(body),
    })
}

/// Bottom elimination: `μ̃().c`.
///
/// Builds a [`Coterm::Elim`] wrapping a [`BotElim`]. The body receives
/// no arguments — the destructor for the unit type `⊥`.
///
/// Reduction: `cut((), μ̃().c)` reduces to `c` `MMM §7, rule (R1)].
pub fn mu_unit<'s>(body: impl FnOnce() -> Command<'s> + 's) -> Coterm<'s, Bot> {
    Coterm::Elim(BotElim {
        body: Box::new(body),
    })
}

/// Par elimination: `μ̃(x ⅋ y).c`.
///
/// Builds a [`Coterm::Elim`] wrapping a [`ParElim`]. The body receives
/// two terms — the components of the tensor pair that triggered this
/// reduction.
///
/// This is a μ̃-form specialized to pattern-matching on tensor pairs:
/// `μ̃(x ⅋ y).c` destructures the pair and binds its components
/// `Grokking §4.1]. Reduction: `cut((v, w), μ̃(x ⅋ y).c)` reduces to
/// `c[v/x, w/y]` `MMM §7, rule (R⊗)].
pub fn mu_par<'s, A: Positive, B: Positive>(
    body: impl FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s> + 's,
) -> Coterm<'s, Par<A::Dual, B::Dual>>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    Coterm::Elim(ParElim {
        body: Box::new(body),
    })
}

/// Case elimination: `μ̃case(x ⇒ c₁, y ⇒ c₂)`.
///
/// Builds a [`Coterm::Elim`] wrapping a [`WithElim`]. The two bodies
/// correspond to the left and right injections of the dual `Plus` type.
///
/// This is a μ̃-form that pattern-matches on plus injections:
/// `μ̃case(x ⇒ c₁, y ⇒ c₂)` dispatches to the appropriate arm based on
/// whether the term was `inl` or `inr` `Grokking §4.1].
/// Reduction: `cut(inl(v), μ̃case(x ⇒ c₁, y ⇒ c₂))` reduces to `c₁[v/x]`
/// [Spiwack, `iota1`/`iota2`].
pub fn mu_case<'s, A: Positive, B: Positive>(
    body_left: impl FnOnce(Term<'s, A>) -> Command<'s> + 's,
    body_right: impl FnOnce(Term<'s, B>) -> Command<'s> + 's,
) -> Coterm<'s, With<A::Dual, B::Dual>>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    Coterm::Elim(WithElim {
        left: Box::new(body_left),
        right: Box::new(body_right),
    })
}

/// Exponential elimination: `μ̃!x.c`.
///
/// Builds a [`Coterm::Elim`] wrapping a [`WhynotElim`]. The body
/// receives a [`BangIntro`] — a duplicable producer of terms.
///
/// The body may clone the producer any number of times (zero, one,
/// many), or drop it without use. This is the exponential destructor
/// `μ̃!x.c` [Spiwack, `exponential`].
pub fn mu_bang<'s, A: Positive>(
    body: impl FnOnce(BangIntro<'s, A>) -> Command<'s> + 's,
) -> Coterm<'s, Whynot<A::Dual>>
where
    A::Dual: Negative,
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
/// Takes a producer closure `Fn() -> Term<'s, A>` that can be invoked
/// any number of times, each time yielding a fresh linear term. The
/// producer must be closed (no free linear variables) — this is
/// enforced by Rust's move semantics on the closure.
///
/// This is the promotion rule `!` of linear logic: a closed term
/// becomes duplicable. The resulting [`BangIntro`] can be cloned to
/// produce fresh linear terms on demand.
///
/// # Example
/// ```
/// use sequents::*;
///
/// struct X;
///
/// let bang = promote::<'static, AtomP<X>>(|| Term::Axiom(Resource::new()));
/// let v1 = derelict(bang.clone());
/// let v2 = derelict(bang); // fresh term
/// ```
pub fn promote<'s, A: Positive>(producer: impl Fn() -> Term<'s, A> + 's) -> BangIntro<'s, A>
where
    A::Dual: Negative,
{
    BangIntro {
        producer: std::rc::Rc::new(producer),
        _marker: PhantomData,
    }
}

/// Derelict: extract a linear term from a classical producer.
///
/// Invokes the producer once, yielding a fresh `Term<'s, A>`.
/// This is the dereliction rule `?` of linear logic: from a
/// duplicable term, obtain a single linear use.
///
/// Each call to `derelict` on the same [`BangIntro`] produces a
/// fresh term — the producer is not consumed.
pub fn derelict<'s, A: Positive>(v: BangIntro<'s, A>) -> Term<'s, A>
where
    A::Dual: Negative,
{
    (v.producer)()
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Resource;

    struct X;
    struct Y;

    #[test]
    fn unit_constructor() {
        let _u = unit();
    }

    #[test]
    fn tensor_constructor() {
        let x: Resource<'static, AtomP<X>> = Resource::new();
        let y: Resource<'static, AtomP<Y>> = Resource::new();
        let _pair = tensor(x, y);
    }

    #[test]
    fn inl_constructor() {
        let x: Resource<'static, AtomP<X>> = Resource::new();
        let _v = inl::<AtomP<X>, AtomP<Y>>(x);
    }

    #[test]
    fn inr_constructor() {
        let y: Resource<'static, AtomP<Y>> = Resource::new();
        let _v = inr::<AtomP<X>, AtomP<Y>>(y);
    }

    #[test]
    fn promote_and_derelict() {
        let bang = promote::<'static, AtomP<X>>(|| Term::Axiom(Resource::new()));
        let _v1 = derelict(bang.clone());
        let _v2 = derelict(bang);
    }
}
