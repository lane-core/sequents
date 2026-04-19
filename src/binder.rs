use std::sync::Arc;

use crate::machine::Command;
use crate::types::{
    Bot, Coterm, Negative, One, Par, Plus, PlusIntro, Positive, Tensor, Term, Whynot, With,
};

// =============================================================================
// μ and μ̃ binders (general continuations)
// =============================================================================

/// μ-binder constructor: `μα.c` on the positive side.
///
/// Builds a [`Term::Mu`] — a general continuation that receives a coterm
/// of the dual type. In the λμ̃μ-calculus, `μα.c` binds a covariable
/// `α` and captures the current evaluation context as a first-class
/// object `Grokking §3.2]`.
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
/// struct XTag;
/// pub struct XElim<'s> { pub body: Box<dyn FnOnce(X) -> Command<'s> + 's> }
///
/// impl Positive for X {
///     type Dual = XTag;
///     type Intro<'s> = std::convert::Infallible;
///     type Witness<'x> = X;
///     fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> { match i {} }
/// }
/// impl Negative for XTag {
///     type Dual = X;
///     type Elim<'s> = XElim<'s>;
///     type Witness<'x> = XElim<'x>;
///     fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
///         (e.body)(r.into_witness())
///     }
/// }
///
/// let x: Resource<'static, X> = Resource::new(X);
/// let pos = mu::<'static, X>(|a: Coterm<'_, XTag>| {
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
/// `Grokking §3.2]`.
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
/// struct XTag;
/// pub struct XElim<'s> { pub body: Box<dyn FnOnce(X) -> Command<'s> + 's> }
///
/// impl Positive for X {
///     type Dual = XTag;
///     type Intro<'s> = std::convert::Infallible;
///     type Witness<'x> = X;
///     fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> { match i {} }
/// }
/// impl Negative for XTag {
///     type Dual = X;
///     type Elim<'s> = XElim<'s>;
///     type Witness<'x> = XElim<'x>;
///     fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
///         (e.body)(r.into_witness())
///     }
/// }
///
/// let z: CoResource<'static, XTag> = CoResource::new(XElim { body: Box::new(|_| Command::Normal) });
/// let coterm = mu_tilde::<'static, XTag>(|t: Term<'_, X>| cut(t, z.into()));
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
/// `MMM §7, rule (R1)]`.
#[allow(clippy::unused_unit)]
pub fn unit<'s>() -> Term<'s, One> {
    Term::Intro(())
}

/// Tensor introduction: `V ⊗ W`.
///
/// Takes anything convertible to `Term` (resources auto-wrap via
/// [`From<Resource>`]) and returns a `Term<'s, Tensor<A, B>>`.
/// The introduction rule for tensor pairs `MMM §7, rule (R⊗)]`.
///
/// # Example
/// ```
/// use sequents::*;
///
/// struct X;
/// struct XTag;
/// pub struct XElim<'s> { pub body: Box<dyn FnOnce(X) -> Command<'s> + 's> }
/// impl Positive for X {
///     type Dual = XTag;
///     type Intro<'s> = std::convert::Infallible;
///     type Witness<'x> = X;
///     fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> { match i {} }
/// }
/// impl Negative for XTag {
///     type Dual = X;
///     type Elim<'s> = XElim<'s>;
///     type Witness<'x> = XElim<'x>;
///     fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
///         (e.body)(r.into_witness())
///     }
/// }
///
/// struct Y;
/// struct YTag;
/// pub struct YElim<'s> { pub body: Box<dyn FnOnce(Y) -> Command<'s> + 's> }
/// impl Positive for Y {
///     type Dual = YTag;
///     type Intro<'s> = std::convert::Infallible;
///     type Witness<'x> = Y;
///     fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> { match i {} }
/// }
/// impl Negative for YTag {
///     type Dual = Y;
///     type Elim<'s> = YElim<'s>;
///     type Witness<'x> = YElim<'x>;
///     fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
///         (e.body)(r.into_witness())
///     }
/// }
///
/// let x: Resource<'static, X> = Resource::new(X);
/// let y: Resource<'static, Y> = Resource::new(Y);
/// let pair = tensor(x, y); // Term<Tensor<X, Y>>
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

/// Bottom elimination: `μ̃().c`.
///
/// Builds a [`Coterm::Elim`] wrapping a zero-argument closure.
/// The body receives no arguments — the destructor for the unit type `⊥`.
///
/// Reduction: `cut((), μ̃().c)` reduces to `c` `MMM §7, rule (R1)]`.
pub fn mu_unit<'s>(body: impl FnOnce() -> Command<'s> + 's) -> Coterm<'s, Bot> {
    Coterm::Elim(Box::new(body) as _)
}

/// Par elimination: `μ̃(x ⅋ y).c`.
///
/// Builds a [`Coterm::Elim`] wrapping a two-argument closure.
/// The body receives two terms — the components of the tensor pair that
/// triggered this reduction.
///
/// This is a μ̃-form specialized to pattern-matching on tensor pairs:
/// `μ̃(x ⅋ y).c` destructures the pair and binds its components
/// `Grokking §4.1]`. Reduction: `cut((v, w), μ̃(x ⅋ y).c)` reduces to
/// `c[v/x, w/y]` `MMM §7, rule (R⊗)]`.
pub fn mu_par<'s, A: Positive, B: Positive>(
    body: impl FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s> + 's,
) -> Coterm<'s, Par<A::Dual, B::Dual>>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    Coterm::Elim(Box::new(body) as _)
}

/// Case elimination: `μ̃case(x ⇒ c₁, y ⇒ c₂)`.
///
/// Builds a [`Coterm::Elim`] wrapping a pair of closures.
/// The two bodies correspond to the left and right injections of the
/// dual `Plus` type.
///
/// This is a μ̃-form that pattern-matches on plus injections:
/// `μ̃case(x ⇒ c₁, y ⇒ c₂)` dispatches to the appropriate arm based on
/// whether the term was `inl` or `inr` `Grokking §4.1]`.
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
    Coterm::Elim((Box::new(body_left) as _, Box::new(body_right) as _))
}

/// Exponential elimination: `μ̃!x.c`.
///
/// Builds a [`Coterm::Elim`] wrapping a closure consuming an `Arc`.
/// The body receives an `Arc<A::Witness<'s>>` — a duplicable witness.
///
/// The body may clone the `Arc` any number of times (zero, one,
/// many), or drop it without use. This is the exponential destructor
/// `μ̃!x.c` [Spiwack, `exponential`].
pub fn mu_bang<'s, A: Positive>(
    body: impl FnOnce(Arc<A::Witness<'s>>) -> Command<'s> + 's,
) -> Coterm<'s, Whynot<A::Dual>>
where
    A::Dual: Negative,
{
    Coterm::Elim(Box::new(body) as _)
}

// =============================================================================
// Exponential helpers
// =============================================================================

/// Promote a witness to a classical (duplicable) `Arc`.
///
/// Takes a witness of type `A::Witness<'s>` and wraps it in an `Arc`.
/// The resulting `Arc` can be cloned any number of times, each clone
/// yielding a fresh handle to the same witness.
///
/// This is the promotion rule `!` of linear logic: a closed witness
/// becomes duplicable.
///
/// # Example
/// ```
/// use sequents::*;
///
/// #[derive(Clone)]
/// struct X;
/// struct XTag;
/// pub struct XElim<'s> { pub body: Box<dyn FnOnce(X) -> Command<'s> + 's> }
/// impl Positive for X {
///     type Dual = XTag;
///     type Intro<'s> = std::convert::Infallible;
///     type Witness<'x> = X;
///     fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> { match i {} }
/// }
/// impl Negative for XTag {
///     type Dual = X;
///     type Elim<'s> = XElim<'s>;
///     type Witness<'x> = XElim<'x>;
///     fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
///         (e.body)(r.into_witness())
///     }
/// }
///
/// let bang = promote::<'static, X>(X);
/// let v1: Term<'static, X> = derelict(bang.clone());
/// let v2: Term<'static, X> = derelict(bang); // fresh term
/// ```
pub fn promote<'s, A: Positive>(witness: A::Witness<'s>) -> Arc<A::Witness<'s>> {
    Arc::new(witness)
}

/// Derelict: extract a linear term from a classical `Arc`.
///
/// Unwraps the `Arc`, yielding a fresh `Term::Axiom(Resource::new(witness))`.
///
/// This is the dereliction rule `?` of linear logic: from a
/// duplicable term, obtain a single linear use.
///
/// Each call to `derelict` on the same `Arc` produces a fresh term —
/// the `Arc` is not consumed.
pub fn derelict<'s, A: Positive>(v: Arc<A::Witness<'s>>) -> Term<'s, A>
where
    A::Witness<'s>: Clone,
{
    let witness = Arc::try_unwrap(v).unwrap_or_else(|arc| (*arc).clone());
    Term::Axiom(crate::types::Resource::new(witness))
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Resource;

    #[derive(Clone, Copy)]
    pub struct X;
    pub struct XTag;
    pub struct XElim<'s> {
        pub body: Box<dyn FnOnce(X) -> Command<'s> + 's>,
    }

    impl Positive for X {
        type Dual = XTag;
        type Intro<'s> = std::convert::Infallible;
        type Witness<'x> = X;
        fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
            match i {}
        }
    }

    impl Negative for XTag {
        type Dual = X;
        type Elim<'s> = XElim<'s>;
        type Witness<'x> = XElim<'x>;
        fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
            (e.body)(r.into_witness())
        }
    }

    #[derive(Clone, Copy)]
    pub struct Y;
    pub struct YTag;
    pub struct YElim<'s> {
        pub body: Box<dyn FnOnce(Y) -> Command<'s> + 's>,
    }

    impl Positive for Y {
        type Dual = YTag;
        type Intro<'s> = std::convert::Infallible;
        type Witness<'x> = Y;
        fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
            match i {}
        }
    }

    impl Negative for YTag {
        type Dual = Y;
        type Elim<'s> = YElim<'s>;
        type Witness<'x> = YElim<'x>;
        fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
            (e.body)(r.into_witness())
        }
    }

    #[test]
    fn unit_constructor() {
        let _u = unit();
    }

    #[test]
    fn tensor_constructor() {
        let x: Resource<'static, X> = Resource::new(X);
        let y: Resource<'static, Y> = Resource::new(Y);
        let _pair = tensor(x, y);
    }

    #[test]
    fn inl_constructor() {
        let x: Resource<'static, X> = Resource::new(X);
        let _v = inl::<X, Y>(x);
    }

    #[test]
    fn inr_constructor() {
        let y: Resource<'static, Y> = Resource::new(Y);
        let _v = inr::<X, Y>(y);
    }

    #[test]
    fn promote_and_derelict() {
        let bang = promote::<'static, X>(X);
        let _v1: Term<'static, X> = derelict(bang.clone());
        let _v2: Term<'static, X> = derelict(bang);
    }
}
