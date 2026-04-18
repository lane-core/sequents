//! # sequents
//!
//! A research prototype embedding the multiplicative fragment of linear System L
//! into Rust's type system.  Scope structure is carried by lifetimes; linearity
//! is enforced by move semantics; binders use higher-rank lifetime
//! quantification (`for<'x>`).

use std::any::Any;
use std::marker::PhantomData;

// =============================================================================
// 1.  Polarity traits
// =============================================================================

pub trait Pos: Sized + 'static {
    type Dual: Neg<Dual = Self>;
    type Value<'s>;
}

pub trait Neg: Sized + 'static {
    type Dual: Pos<Dual = Self>;
    type CoValue<'s>;
}

// =============================================================================
// 2.  Atoms
// =============================================================================

pub struct AtomP<X>(PhantomData<X>);
pub struct AtomN<X>(PhantomData<X>);

impl<X: 'static> Pos for AtomP<X> {
    type Dual = AtomN<X>;
    type Value<'s> = Var<'s, AtomP<X>>;
}

impl<X: 'static> Neg for AtomN<X> {
    type Dual = AtomP<X>;
    type CoValue<'s> = Var<'s, AtomN<X>>;
}

// =============================================================================
// 3.  Multiplicative units
// =============================================================================

pub struct One;
pub struct Bot;

impl Pos for One {
    type Dual = Bot;
    type Value<'s> = ();
}

impl Neg for Bot {
    type Dual = One;
    type CoValue<'s> = std::convert::Infallible;
}

// =============================================================================
// 4.  Multiplicative connectives
// =============================================================================

pub struct Tensor<A: Pos, B: Pos>(PhantomData<(A, B)>);
pub struct Par<A: Neg, B: Neg>(PhantomData<(A, B)>);

impl<A: Pos, B: Pos> Pos for Tensor<A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    type Dual = Par<A::Dual, B::Dual>;
    type Value<'s> = (A::Value<'s>, B::Value<'s>);
}

impl<A: Neg, B: Neg> Neg for Par<A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type Dual = Tensor<A::Dual, B::Dual>;
    type CoValue<'s> = std::convert::Infallible;
}

// =============================================================================
// 5.  Variables — linear tokens
// =============================================================================

pub struct Var<'x, A> {
    _marker: PhantomData<&'x ()>,
    _type: PhantomData<A>,
}

impl<'x, A> Var<'x, A> {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Var {
            _marker: PhantomData,
            _type: PhantomData,
        }
    }
}

// =============================================================================
// 6.  Binder-body traits (stable-Rust workaround for FnOnce trait objects)
// =============================================================================

pub trait PosBinder<A: Pos> {
    fn call<'x>(self: Box<Self>, x: Var<'x, A>) -> Command<'x>;
}

impl<A: Pos, F> PosBinder<A> for F
where
    F: for<'x> FnOnce(Var<'x, A>) -> Command<'x> + 'static,
{
    fn call<'x>(self: Box<Self>, x: Var<'x, A>) -> Command<'x> {
        (*self)(x)
    }
}

pub trait NegBinder<N: Neg> {
    fn call<'x>(self: Box<Self>, x: Var<'x, N::Dual>) -> Command<'x>;
}

impl<N: Neg, F> NegBinder<N> for F
where
    F: for<'x> FnOnce(Var<'x, N::Dual>) -> Command<'x> + 'static,
{
    fn call<'x>(self: Box<Self>, x: Var<'x, N::Dual>) -> Command<'x> {
        (*self)(x)
    }
}

pub trait UnitBinder {
    fn call(self: Box<Self>) -> Command<'static>;
}

impl<F> UnitBinder for F
where
    F: FnOnce() -> Command<'static> + 'static,
{
    fn call(self: Box<Self>) -> Command<'static> {
        (*self)()
    }
}

pub trait ParBinder<A: Pos, B: Pos> {
    fn call<'x>(self: Box<Self>, x: Var<'x, A>, y: Var<'x, B>) -> Command<'x>;
}

impl<A: Pos, B: Pos, F> ParBinder<A, B> for F
where
    F: for<'x> FnOnce(Var<'x, A>, Var<'x, B>) -> Command<'x> + 'static,
{
    fn call<'x>(self: Box<Self>, x: Var<'x, A>, y: Var<'x, B>) -> Command<'x> {
        (*self)(x, y)
    }
}

// =============================================================================
// 7.  Expressions, co-expressions, and commands
// =============================================================================

/// A positive expression `⊢ t : A | Γ` at scope `'s`.
pub enum Expr<'s, A: Pos> {
    Var(Var<'s, A>),
    Val(A::Value<'s>),
    MuPos(MuPos<A>),
}

/// A negative co-expression (value of negative type) at scope `'s`.
pub enum CoExpr<'s, N: Neg> {
    CoVal(N::CoValue<'s>),
    MuNeg(MuNeg<N>),
    /// Only valid for `N = Bot`.
    MuUnit(MuUnit),
    /// Only valid for `N = Par<A, B>`. The `Any` boxing is needed because
    /// `MuPar<A, B>` does not fit into the enum's type parameter `N`.
    MuPar(Box<dyn Any>),
}

pub struct MuPos<A: Pos> {
    #[allow(dead_code)]
    body: Box<dyn PosBinder<A> + 'static>,
}

pub struct MuNeg<N: Neg> {
    body: Box<dyn NegBinder<N> + 'static>,
}

pub struct MuUnit {
    #[allow(dead_code)]
    body: Box<dyn UnitBinder + 'static>,
}

pub struct MuPar<A: Pos, B: Pos> {
    body: Box<dyn ParBinder<A, B> + 'static>,
}

/// A command `c : (⊢ Γ)` at scope `'s`.
pub struct Command<'s> {
    _marker: PhantomData<&'s ()>,
}

impl<'s> Command<'s> {
    fn new() -> Self {
        Command {
            _marker: PhantomData,
        }
    }
}

// =============================================================================
// 8.  Constructors
// =============================================================================

impl<'s, A: Pos> Expr<'s, A> {
    pub fn var(v: Var<'s, A>) -> Self {
        Expr::Var(v)
    }
    pub fn val(v: A::Value<'s>) -> Self {
        Expr::Val(v)
    }
}

impl<'s, N: Neg> CoExpr<'s, N> {
    pub fn co_val(v: N::CoValue<'s>) -> Self {
        CoExpr::CoVal(v)
    }
}

impl<'s> CoExpr<'s, Bot> {
    pub fn mu_unit(mu: MuUnit) -> Self {
        CoExpr::MuUnit(mu)
    }
}

impl<'s, A: Neg, B: Neg> CoExpr<'s, Par<A, B>>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    pub fn mu_par(mu: MuPar<A::Dual, B::Dual>) -> Self {
        CoExpr::MuPar(Box::new(mu))
    }
}

pub fn cut<'s, A: Pos>(_t: Expr<'s, A>, _v: CoExpr<'s, A::Dual>) -> Command<'s>
where
    A::Dual: Neg,
{
    Command::new()
}

pub fn mu_pos<A: Pos, F>(body: F) -> Expr<'static, A>
where
    F: for<'x> FnOnce(Var<'x, A>) -> Command<'x> + 'static,
{
    Expr::MuPos(MuPos {
        body: Box::new(body),
    })
}

pub fn mu_neg<N: Neg, F>(body: F) -> CoExpr<'static, N>
where
    F: for<'x> FnOnce(Var<'x, N::Dual>) -> Command<'x> + 'static,
{
    CoExpr::MuNeg(MuNeg {
        body: Box::new(body),
    })
}

#[allow(clippy::unused_unit)]
pub fn unit() -> <One as Pos>::Value<'static> {
    ()
}

pub fn tensor<'s, A: Pos, B: Pos>(
    v: A::Value<'s>,
    w: B::Value<'s>,
) -> (A::Value<'s>, B::Value<'s>) {
    (v, w)
}

pub fn mu_unit<F>(body: F) -> CoExpr<'static, Bot>
where
    F: FnOnce() -> Command<'static> + 'static,
{
    CoExpr::mu_unit(MuUnit {
        body: Box::new(body),
    })
}

pub fn mu_par<A: Pos, B: Pos, F>(body: F) -> CoExpr<'static, Par<A::Dual, B::Dual>>
where
    A::Dual: Neg,
    B::Dual: Neg,
    Par<A::Dual, B::Dual>: Neg,
    F: for<'x> FnOnce(Var<'x, A>, Var<'x, B>) -> Command<'x> + 'static,
{
    CoExpr::mu_par(MuPar {
        body: Box::new(body),
    })
}

// =============================================================================
// 9.  Reduction (atomic case)
// =============================================================================

/// Reduce `⟨x | μy⁻.c⟩` for atomic `X`.
///
///   `⟨x : X | μy⁻.c⟩ ▷ c[x/y]`
pub fn reduce_mu_neg_atom<'s, X: 'static>(
    x: Var<'s, AtomP<X>>,
    body: MuNeg<AtomN<X>>,
) -> Command<'s> {
    body.body.call(x)
}

/// Reduce `⟨V ⊗ W | μ(x ⅋ y).c⟩` for atomic `X`, `Y`.
///
///   `⟨x ⊗ y | μ(a ⅋ b).c⟩ ▷ c[x/a, y/b]`
pub fn reduce_tensor_par_atom<'s, X: 'static, Y: 'static>(
    x: Var<'s, AtomP<X>>,
    y: Var<'s, AtomP<Y>>,
    body: MuPar<AtomP<X>, AtomP<Y>>,
) -> Command<'s> {
    body.body.call(x, y)
}

// =============================================================================
// 10. Tests
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
        let _cmd = cut(Expr::var(x), CoExpr::co_val(y));
    }

    #[test]
    fn mu_neg_binder() {
        let a: Var<'static, AtomN<X>> = Var::new();
        let _co: CoExpr<'static, AtomN<X>> =
            mu_neg(|y: Var<'_, AtomP<X>>| cut(Expr::var(y), CoExpr::co_val(a)));
    }

    #[test]
    fn tensor_intro() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let p = tensor::<'static, AtomP<X>, AtomP<Y>>(x, y);
        let _expr: Expr<'static, Tensor<AtomP<X>, AtomP<Y>>> = Expr::val(p);
    }

    #[test]
    fn par_destructor() {
        // mu_par body: use both introduced variables in separate cuts.
        // We return one command; the other is dropped (test only).
        let _co: CoExpr<'static, Par<AtomN<X>, AtomN<Y>>> =
            mu_par::<AtomP<X>, AtomP<Y>, _>(|x, y| {
                let a: Var<'_, AtomN<X>> = Var::new();
                let b: Var<'_, AtomN<Y>> = Var::new();
                let cmd_x = cut(Expr::var(x), CoExpr::co_val(a));
                let cmd_y = cut(Expr::var(y), CoExpr::co_val(b));
                let _ = cmd_x; // in a real term, both would be composed
                cmd_y
            });
    }

    // Nested binders that capture outer variables do NOT compile because
    // the `for<'x>` higher-rank bound requires the closure to be valid
    // for all lifetimes 'x, which forbids capturing non-'static data.
    // See NOTES.md for discussion.

    #[test]
    fn unit_and_bottom() {
        let u = unit();
        let _expr: Expr<'static, One> = Expr::val(u);
        let _bot: CoExpr<'static, Bot> = mu_unit(|| Command::new());
    }

    #[test]
    fn reduce_mu_neg_atom_works() {
        let x: Var<'static, AtomP<X>> = Var::new();
        // Build the body using the public constructor, then extract the inner MuNeg.
        let co: CoExpr<'static, AtomN<X>> = mu_neg(|y: Var<'_, AtomP<X>>| {
            let a: Var<'_, AtomN<X>> = Var::new();
            cut(Expr::var(y), CoExpr::co_val(a))
        });
        let body = match co {
            CoExpr::MuNeg(body) => body,
            _ => panic!("expected MuNeg"),
        };
        let _cmd = super::reduce_mu_neg_atom(x, body);
    }

    #[test]
    fn reduce_tensor_par_atom_works() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let co: CoExpr<'static, Par<AtomN<X>, AtomN<Y>>> =
            mu_par::<AtomP<X>, AtomP<Y>, _>(|a, b| {
                let m: Var<'_, AtomN<X>> = Var::new();
                let n: Var<'_, AtomN<Y>> = Var::new();
                let cmd1 = cut(Expr::var(a), CoExpr::co_val(m));
                let _ = (cmd1, b, n);
                Command::new()
            });
        let body = match co {
            CoExpr::MuPar(boxed) => match boxed.downcast::<MuPar<AtomP<X>, AtomP<Y>>>() {
                Ok(b) => *b,
                Err(_) => panic!("downcast failed"),
            },
            _ => panic!("expected MuPar"),
        };
        let _cmd = super::reduce_tensor_par_atom(x, y, body);
    }

    /// This test demonstrates that the involutive duality bound compiles.
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
    //       let _ = Expr::var(x);
    //       let _ = Expr::var(x); // ERROR: use of moved value
    //   }
}
