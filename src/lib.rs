//! # sequents
//!
//! A research prototype embedding the multiplicative fragment of linear System L
//! into Rust's type system.  Scope structure is carried by lifetimes; linearity
//! is enforced by move semantics.
//!
//! This is **Option B** from the design memo: `Command<'s>` is a continuation
//! (a closure that performs a reduction step when invoked), and `Outcome<'s>`
//! describes the result of that step.  Reduction is invocation-based (Krivine-
//! machine style), not AST inspection.

use std::marker::PhantomData;

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

impl Neg for Bot {
    type Dual = One;
    /// `Bot` has no user-constructible co-value form.
    type CoValue<'s> = std::convert::Infallible;
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

impl<A: Neg, B: Neg> Neg for Par<A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type Dual = Tensor<A::Dual, B::Dual>;
    /// `Par` has no user-constructible co-value form; introduction is only via
    /// the `μ(x ⅋ y)` binder.
    type CoValue<'s> = std::convert::Infallible;
}

// =============================================================================
// 5.  Variables — linear tokens
// =============================================================================

/// A variable (or covariable) token.
///
/// * Non-`Copy`, non-`Clone` — using it twice is a compile error.
/// * The lifetime `'x` is the scope in which this variable is valid.
/// * `PhantomData<&'x ()>` makes `Var` **covariant** in `'x`.
pub struct Var<'x, A> {
    _marker: PhantomData<&'x ()>,
    _type: PhantomData<A>,
}

impl<'x, A> Var<'x, A> {
    /// Create a fresh variable token.  In a real term, variables are
    /// introduced by binders; this constructor is useful for building
    /// open terms (e.g., the axiom rule) and for testing.
    pub fn new() -> Self {
        Var {
            _marker: PhantomData,
            _type: PhantomData,
        }
    }
}

impl<'x, A> Default for Var<'x, A> {
    fn default() -> Self {
        Self::new()
    }
}

// Explicitly NOT implementing Clone or Copy.
// Move semantics enforce linearity.

// =============================================================================
// 6.  Expression and co-expression traits (tagless-final style)
// =============================================================================

/// A positive expression `⊢ t : A | Γ` at scope `'s`.
///
/// `Expr` is a marker trait: any type implementing it is a well-formed
/// positive expression of type `A` at scope `'s`.
pub trait Expr<'s, A: Pos> {}

/// A negative co-expression (value of negative type) at scope `'s`.
///
/// `CoExpr` is a marker trait: any type implementing it is a well-formed
/// negative co-expression of type `N` at scope `'s`.
pub trait CoExpr<'s, N: Neg> {}

// -- Variables are expressions (axiom rule) ----------------------------------

impl<'s, A: Pos> Expr<'s, A> for Var<'s, A> {}
impl<'s, N: Neg> CoExpr<'s, N> for Var<'s, N> {}

// -- Unit value is an expression ---------------------------------------------

impl<'s> Expr<'s, One> for () {}

// -- Tensor value (pair) is an expression ------------------------------------

impl<'s, A: Pos, B: Pos> Expr<'s, Tensor<A, B>> for (A::Value<'s>, B::Value<'s>)
where
    A::Dual: Neg,
    B::Dual: Neg,
{
}

// =============================================================================
// 7.  Binder types
// =============================================================================

/// Positive μ-binder `μx⁺.c` at scope `'s`.
///
/// The body receives `A::Value<'s>` — the introduction form for `A` at
/// scope `'s`.  For atoms this is a `Var`; for tensors it is a pair.
pub struct MuPos<'s, A: Pos, F> {
    pub body: F,
    _marker: PhantomData<&'s ()>,
    _type: PhantomData<A>,
}

/// Negative μ-binder `μx⁻.c` at scope `'s`.
///
/// The body receives `N::Dual::Value<'s>` — the introduction form for the
/// dual of `N`.  For atomic negative types this is a positive variable.
pub struct MuNeg<'s, N: Neg, F> {
    pub body: F,
    _marker: PhantomData<&'s ()>,
    _type: PhantomData<N>,
}

/// Bottom destructor `μ().c` at scope `'s`.
pub struct MuUnit<'s, F> {
    pub body: F,
    _marker: PhantomData<&'s ()>,
}

/// Par destructor `μ(x ⅋ y).c` at scope `'s`.
///
/// The body receives `A::Value<'s>` and `B::Value<'s>` — the components
/// of the tensor value that triggered this reduction.  For atoms these are
/// variables; for composite types they are nested pairs.
pub struct MuPar<'s, A: Pos, B: Pos, F> {
    pub body: F,
    _marker: PhantomData<&'s ()>,
    _types: PhantomData<(A, B)>,
}

// =============================================================================
// 8.  Binder trait implementations
// =============================================================================

impl<'s, A: Pos, F> Expr<'s, A> for MuPos<'s, A, F> where F: FnOnce(A::Value<'s>) -> Command<'s> {}

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

// =============================================================================
// 9.  Outcome and Command
// =============================================================================

/// The result of invoking a command's reduction step.
pub enum Outcome<'s> {
    /// Reduction has reached a normal form (no further steps possible).
    Done,
    /// One reduction step was performed; the resulting command is ready
    /// for the next step.
    Step(Command<'s>),
    /// Reduction is stuck — the current configuration has no applicable
    /// reduction rule.  This should not occur in well-typed programs.
    Stuck(StuckReason),
}

impl<'s> std::fmt::Debug for Outcome<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Done => write!(f, "Done"),
            Outcome::Step(_) => write!(f, "Step(Command {{ .. }})"),
            Outcome::Stuck(r) => write!(f, "Stuck({r:?})"),
        }
    }
}

/// Why a command is stuck.
#[derive(Debug, PartialEq, Clone)]
pub enum StuckReason {
    /// The generic `cut` was used with a configuration that has no
    /// operational reduction rule (static-only well-formedness check).
    StaticOnly,
    /// Catch-all for unexpected configurations during development.
    Unexpected(String),
}

/// A command `c : (⊢ Γ)` at scope `'s`.
///
/// In Option B, a command is a **continuation**: a closure that, when
/// invoked, performs one operational step and returns an `Outcome`.
pub struct Command<'s> {
    step: Box<dyn FnOnce() -> Outcome<'s> + 's>,
}

impl<'s> std::fmt::Debug for Command<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Command {{ .. }}")
    }
}

impl<'s> Command<'s> {
    /// Invoke the command's reduction step.
    pub fn run_once(self) -> Outcome<'s> {
        (self.step)()
    }

    /// Construct a stuck command.
    pub fn stuck(reason: StuckReason) -> Self {
        Command {
            step: Box::new(move || Outcome::Stuck(reason)),
        }
    }
}

/// Drive a command to a terminal state (`Done` or `Stuck`).
pub fn run<'s>(mut cmd: Command<'s>) -> Outcome<'s> {
    loop {
        match cmd.run_once() {
            Outcome::Done => return Outcome::Done,
            Outcome::Step(next) => cmd = next,
            o @ Outcome::Stuck(_) => return o,
        }
    }
}

// =============================================================================
// 10. Constructors
// =============================================================================

/// Positive μ-binder: `μx⁺.c`.
pub fn mu_pos<'s, A: Pos, F>(body: F) -> MuPos<'s, A, F>
where
    F: FnOnce(A::Value<'s>) -> Command<'s>,
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

// =============================================================================
// 11. Reduction step functions (specific cuts)
// =============================================================================

/// Generic cut: `⟨t | V⟩`.
///
/// The type system ensures the two sides have dual types, but without
/// knowing the specific introduction forms, no operational reduction can
/// be performed.  The resulting command is stuck.
pub fn cut<'s, A: Pos, T, E>(_t: T, _e: E) -> Command<'s>
where
    T: Expr<'s, A>,
    E: CoExpr<'s, A::Dual>,
    A::Dual: Neg,
{
    Command::stuck(StuckReason::StaticOnly)
}

/// Atomic cut: `⟨x | μy⁻.c⟩` where `x` is a variable.
///
/// Reduction rule: invoke the binder body with `x`.
///
/// For atomic types, `AtomP<X>::Value<'s> = Var<'s, AtomP<X>>`, so the
/// binder body receives the variable directly.
pub fn cut_atom<'s, X: 'static, F>(
    v: Var<'s, AtomP<X>>,
    binder: MuNeg<'s, AtomN<X>, F>,
) -> Command<'s>
where
    F: FnOnce(Var<'s, AtomP<X>>) -> Command<'s> + 's,
{
    let body = binder.body;
    Command {
        step: Box::new(move || Outcome::Step(body(v))),
    }
}

/// Unit cut: `⟨() | μ().c⟩`.
///
/// Reduction rule: invoke the binder body with no argument.
pub fn cut_unit<'s, F>(_v: (), binder: MuUnit<'s, F>) -> Command<'s>
where
    F: FnOnce() -> Command<'s> + 's,
{
    let body = binder.body;
    Command {
        step: Box::new(move || Outcome::Step(body())),
    }
}

/// Tensor-par cut: `⟨V ⊗ W | μ(x ⅋ y).c⟩`.
///
/// Reduction rule: destructure the pair and invoke the binder body with
/// the components.  Because binder bodies now receive `A::Value<'s>` and
/// `B::Value<'s>` (not `Var`s), this works for both atomic and composite
/// types without conversion.
///
/// For composite components, the binder body receives nested pairs and
/// can use further `cut_par` calls to destructure them (Milestone 4).
pub fn cut_par<'s, A: Pos, B: Pos, F>(
    v: (A::Value<'s>, B::Value<'s>),
    binder: MuPar<'s, A, B, F>,
) -> Command<'s>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F: FnOnce(A::Value<'s>, B::Value<'s>) -> Command<'s> + 's,
{
    let body = binder.body;
    let (a, b) = v;
    Command {
        step: Box::new(move || Outcome::Step(body(a, b))),
    }
}

// =============================================================================
// 12. Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct X;
    struct Y;

    #[test]
    fn atomic_axiom() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomN<X>> = Var::new();
        let _cmd: Command<'static> = cut(x, y);
    }

    #[test]
    fn unit_cut() {
        let u = unit();
        let co = mu_unit(|| Command::stuck(StuckReason::Unexpected("test".into())));
        let _cmd: Command<'static> = cut(u, co);
    }

    #[test]
    fn tensor_intro() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let _pair = tensor::<'static, AtomP<X>, AtomP<Y>>(x, y);
    }

    #[test]
    fn mu_neg_binder() {
        let a: Var<'static, AtomN<X>> = Var::new();
        let _co = mu_neg::<'static, AtomN<X>, _>(|y: Var<'_, AtomP<X>>| cut(y, a));
    }

    #[test]
    fn par_destructor() {
        let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
            let a: Var<'_, AtomN<X>> = Var::new();
            let b: Var<'_, AtomN<Y>> = Var::new();
            let cmd1 = cut(x, a);
            let _ = cmd1;
            cut(y, b)
        });
    }

    #[test]
    fn nested_binders() {
        let _w: Var<'static, AtomN<Y>> = Var::new();

        let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
            cut(
                x,
                mu_neg::<'_, AtomN<X>, _>(|_z: Var<'_, AtomP<X>>| cut(y, _w)),
            )
        });
    }

    #[test]
    fn triple_nested() {
        let _w: Var<'static, AtomN<Y>> = Var::new();

        let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
            cut(
                x,
                mu_neg::<'_, AtomN<X>, _>(|_z| {
                    cut(
                        y,
                        mu_neg::<'_, AtomN<Y>, _>(|_a| {
                            let b: Var<'_, AtomN<Y>> = Var::new();
                            cut(_a, b)
                        }),
                    )
                }),
            )
        });
    }

    #[test]
    fn tensor_par_cut() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let pair = tensor::<'static, AtomP<X>, AtomP<Y>>(x, y);

        let co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
            let m: Var<'_, AtomN<X>> = Var::new();
            let n: Var<'_, AtomN<Y>> = Var::new();
            let cmd1 = cut(a, m);
            let _ = cmd1;
            cut(b, n)
        });

        let _cmd: Command<'static> = cut(pair, co);
    }

    #[test]
    fn duality_involution() {
        fn check<P: Pos>() {}
        check::<AtomP<X>>();
        check::<One>();
        check::<Tensor<AtomP<X>, AtomP<Y>>>();
    }

    // =============================================================================
    // Option B operational tests
    // =============================================================================

    /// **Milestone 1**: Atomic cut reduces.
    /// `cut_atom(x, μy⁻.⟨y | z⟩)` should step to `⟨x | z⟩`.
    #[test]
    fn milestone_1_atomic_cut_reduces() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let z: Var<'static, AtomN<X>> = Var::new();

        let binder = mu_neg::<'static, AtomN<X>, _>(|y: Var<'_, AtomP<X>>| cut(y, z));
        let cmd = cut_atom(x, binder);

        let outcome = run(cmd);
        // After one step, we get `cut(x, z)` which is stuck (static only)
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// **Milestone 2**: Unit cut reduces.
    /// `cut_unit((), μ().c)` should step to `c`.
    #[test]
    fn milestone_2_unit_cut_reduces() {
        let marker = std::rc::Rc::new(std::cell::Cell::new(false));
        let marker2 = marker.clone();

        let binder = mu_unit(move || {
            marker2.set(true);
            Command::stuck(StuckReason::Unexpected("reached".into()))
        });
        let cmd = cut_unit((), binder);

        let outcome = run(cmd);
        assert!(marker.get());
        assert!(matches!(
            outcome,
            Outcome::Stuck(StuckReason::Unexpected(_))
        ));
    }

    /// **Milestone 3**: Tensor-par cut reduces for atoms.
    /// `cut_par((x, y), μ(a ⅋ b).c)` where `x`, `y` are atomic vars.
    #[test]
    fn milestone_3_tensor_par_atoms() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();

        let m: Var<'static, AtomN<X>> = Var::new();
        let n: Var<'static, AtomN<Y>> = Var::new();

        let binder = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
            let cmd1 = cut_atom(a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m)));
            let _ = cmd1;
            cut_atom(b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n)))
        });

        let pair = (x, y);
        let cmd = cut_par(pair, binder);

        // Just verify it runs without panicking; the exact outcome depends
        // on the body, which uses generic cut (stuck).
        let _outcome = run(cmd);
    }

    /// **Milestone 4**: Tensor-par cut reduces for composites.
    /// `cut_par(((a, b), (c, d)), μ(x ⅋ y).c)` where the pair is a nested
    /// tensor.  The binder body receives the composite components and can
    /// use nested `cut_par` calls to destructure them recursively.
    #[test]
    fn milestone_4_tensor_par_composites() {
        struct A;
        struct B;
        struct C;
        struct D;

        let a: Var<'static, AtomP<A>> = Var::new();
        let b: Var<'static, AtomP<B>> = Var::new();
        let c: Var<'static, AtomP<C>> = Var::new();
        let d: Var<'static, AtomP<D>> = Var::new();

        let m: Var<'static, AtomN<A>> = Var::new();
        let n: Var<'static, AtomN<B>> = Var::new();
        let p: Var<'static, AtomN<C>> = Var::new();
        let q: Var<'static, AtomN<D>> = Var::new();

        // Value: ((a, b), (c, d)) : Tensor<Tensor<AtomP<A>, AtomP<B>>, Tensor<AtomP<C>, AtomP<D>>>
        let pair = ((a, b), (c, d));

        // Binder: μ(x ⅋ y). ...  where x : Tensor<AtomP<A>, AtomP<B>> and y : Tensor<AtomP<C>, AtomP<D>>
        let binder =
            mu_par::<'static, Tensor<AtomP<A>, AtomP<B>>, Tensor<AtomP<C>, AtomP<D>>, _>(|x, y| {
                // x: (Var<AtomP<A>>, Var<AtomP<B>>)
                // y: (Var<AtomP<C>>, Var<AtomP<D>>)
                // Destructure x with an inner cut_par
                cut_par::<'_, AtomP<A>, AtomP<B>, _>(
                    x,
                    mu_par::<'_, AtomP<A>, AtomP<B>, _>(|a1, b1| {
                        // Destructure y with another inner cut_par
                        cut_par::<'_, AtomP<C>, AtomP<D>, _>(
                            y,
                            mu_par::<'_, AtomP<C>, AtomP<D>, _>(|c1, d1| {
                                let _ = cut_atom(a1, mu_neg::<'_, AtomN<A>, _>(|v| cut(v, m)));
                                let _ = cut_atom(b1, mu_neg::<'_, AtomN<B>, _>(|v| cut(v, n)));
                                let _ = cut_atom(c1, mu_neg::<'_, AtomN<C>, _>(|v| cut(v, p)));
                                cut_atom(d1, mu_neg::<'_, AtomN<D>, _>(|v| cut(v, q)))
                            }),
                        )
                    }),
                )
            });

        let cmd = cut_par(pair, binder);
        let _outcome = run(cmd);
    }

    /// **Milestone 5**: Multi-step reduction.
    /// `cut_atom(x, μy⁻.cut_atom(y, μz⁻.cut(z, w)))` reduces in two
    /// operational steps to `cut(x, w)` (stuck).
    #[test]
    fn milestone_5_multi_step() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let w: Var<'static, AtomN<X>> = Var::new();

        let cmd = cut_atom(
            x,
            mu_neg::<'static, AtomN<X>, _>(|y| {
                cut_atom(y, mu_neg::<'_, AtomN<X>, _>(|z| cut(z, w)))
            }),
        );

        let outcome = run(cmd);
        // Two reduction steps lead to cut(x, w), which is static-only stuck.
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// **Milestone 6**: Nested binders reduce correctly.
    /// Three levels of binders with outer capture, translated from the
    /// Option A `triple_nested` test into operational form.
    #[test]
    fn milestone_6_nested_binders() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let free: Var<'static, AtomN<Y>> = Var::new();

        let cmd = cut_par(
            (x, y),
            mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
                cut_atom(
                    a,
                    mu_neg::<'_, AtomN<X>, _>(|_z| {
                        cut_atom(
                            b,
                            mu_neg::<'_, AtomN<Y>, _>(|_a| {
                                cut(_a, free) // generic cut — stuck
                            }),
                        )
                    }),
                )
            }),
        );

        let outcome = run(cmd);
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }
}
