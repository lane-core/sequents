//! # sequents
//!
//! A research prototype embedding the multiplicative fragment of linear System L
//! into Rust's type system.  Scope structure is carried by lifetimes; linearity
//! is enforced by move semantics.
//!
//! This is **Option A** from the design memo: `Expr` and `CoExpr` are traits,
//! not enums.  Each introduction form is its own type, implementing the
//! appropriate trait.  There is no `Box<dyn Any>`, no custom binder traits, and
//! no `+ 'static` bounds on closures.

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
pub struct MuPos<'s, A: Pos, F> {
    #[allow(dead_code)]
    body: F,
    _marker: PhantomData<&'s ()>,
    _type: PhantomData<A>,
}

/// Negative μ-binder `μx⁻.c` at scope `'s`.
pub struct MuNeg<'s, N: Neg, F> {
    #[allow(dead_code)]
    body: F,
    _marker: PhantomData<&'s ()>,
    _type: PhantomData<N>,
}

/// Bottom destructor `μ().c` at scope `'s`.
pub struct MuUnit<'s, F> {
    #[allow(dead_code)]
    body: F,
    _marker: PhantomData<&'s ()>,
}

/// Par destructor `μ(x ⅋ y).c` at scope `'s`.
pub struct MuPar<'s, A: Pos, B: Pos, F> {
    #[allow(dead_code)]
    body: F,
    _marker: PhantomData<&'s ()>,
    _types: PhantomData<(A, B)>,
}

// =============================================================================
// 8.  Binder trait implementations
// =============================================================================

impl<'s, A: Pos, F> Expr<'s, A> for MuPos<'s, A, F> where F: FnOnce(Var<'s, A>) -> Command<'s> {}

impl<'s, N: Neg, F> CoExpr<'s, N> for MuNeg<'s, N, F> where
    F: FnOnce(Var<'s, N::Dual>) -> Command<'s>
{
}

impl<'s, F> CoExpr<'s, Bot> for MuUnit<'s, F> where F: FnOnce() -> Command<'s> {}

impl<'s, A: Pos, B: Pos, F> CoExpr<'s, Par<A::Dual, B::Dual>> for MuPar<'s, A, B, F>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F: FnOnce(Var<'s, A>, Var<'s, B>) -> Command<'s>,
{
}

// =============================================================================
// 9.  Command
// =============================================================================

/// A command `c : (⊢ Γ)` at scope `'s`.
///
/// In Phase 1 this is a marker type (ZST).  Phase 2 may give it operational
/// content (continuation-based reduction).
pub struct Command<'s> {
    _marker: PhantomData<&'s ()>,
}

impl<'s> Command<'s> {
    /// Construct a command.  In Phase 1 this is a no-op marker.
    pub fn new() -> Self {
        Command {
            _marker: PhantomData,
        }
    }
}

impl<'s> Default for Command<'s> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// 10. Constructors
// =============================================================================

/// Positive μ-binder: `μx⁺.c`.
///
/// The binder is scoped at `'s` — the body receives a variable of type `A`
/// valid at `'s` and must produce a command at `'s`.  This allows the body
/// to capture variables from the ambient scope, enabling nested binders.
pub fn mu_pos<'s, A: Pos, F>(body: F) -> impl Expr<'s, A>
where
    F: FnOnce(Var<'s, A>) -> Command<'s>,
{
    MuPos {
        body,
        _marker: PhantomData,
        _type: PhantomData,
    }
}

/// Negative μ-binder: `μx⁻.c`.
pub fn mu_neg<'s, N: Neg, F>(body: F) -> impl CoExpr<'s, N>
where
    F: FnOnce(Var<'s, N::Dual>) -> Command<'s>,
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
pub fn mu_unit<'s, F>(body: F) -> impl CoExpr<'s, Bot>
where
    F: FnOnce() -> Command<'s>,
{
    MuUnit {
        body,
        _marker: PhantomData,
    }
}

/// Par destructor: `μ(x ⅋ y).c`.
pub fn mu_par<'s, A: Pos, B: Pos, F>(body: F) -> impl CoExpr<'s, Par<A::Dual, B::Dual>>
where
    A::Dual: Neg,
    B::Dual: Neg,
    F: FnOnce(Var<'s, A>, Var<'s, B>) -> Command<'s>,
{
    MuPar {
        body,
        _marker: PhantomData,
        _types: PhantomData,
    }
}

/// Cut: `⟨t | V⟩`.
///
/// The type system ensures the two sides have dual types.
pub fn cut<'s, A: Pos, T, E>(_t: T, _e: E) -> Command<'s>
where
    T: Expr<'s, A>,
    E: CoExpr<'s, A::Dual>,
    A::Dual: Neg,
{
    Command::new()
}

// =============================================================================
// 11. Tests
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
        let co = mu_unit(|| Command::new());
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
            let cmd_x = cut(x, a);
            let _ = cmd_x;
            cut(y, b)
        });
    }

    /// **The key milestone for Option A**: nested binders that capture outer
    /// variables.  In the first pass this failed because `for<'x>` combined
    /// with `'static` boxing prevented capture.  With traits and specific
    /// lifetimes, it compiles cleanly.
    #[test]
    fn nested_binders() {
        // μ(x ⅋ y).⟨x | μz⁻.⟨y | w⟩⟩
        // where w : Y⊥ is free.
        let _w: Var<'static, AtomN<Y>> = Var::new();

        let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
            cut(
                x,
                mu_neg::<'_, AtomN<X>, _>(|_z: Var<'_, AtomP<X>>| cut(y, _w)),
            )
        });
    }

    /// A more complex nesting: three levels of binders.
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
                            cut(_a, b) // use the innermost variable
                        }),
                    )
                }),
            )
        });
    }

    /// Tensor of two variables cut against a par destructor.
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

    /// Demonstrate that the involutive duality bound still compiles.
    #[test]
    fn duality_involution() {
        fn check<P: Pos>() {}
        check::<AtomP<X>>();
        check::<One>();
        check::<Tensor<AtomP<X>, AtomP<Y>>>();
    }

    // This test must NOT compile — it uses a variable twice.
    // Uncomment to verify that linearity is enforced:
    //   fn _linearity_violation() {
    //       let x: Var<'static, AtomP<X>> = Var::new();
    //       let _ = x;
    //       let _ = x; // ERROR: use of moved value
    //   }
}
