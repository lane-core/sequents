use crate::types::{Bang, BangIntro, Neg, One, Plus, PlusIntro, Pos, Tensor, Term};

/// A positive expression `⊢ t : A | Γ` at scope `'s`.
///
/// `Expr` is a marker trait: any type implementing it is a well-formed
/// positive expression of type `A` at scope `'s`.
pub trait Expr<'s, A: Pos> {}

/// A negative co-expression (coterm of negative type) at scope `'s`.
///
/// `CoExpr` is a marker trait: any type implementing it is a well-formed
/// negative co-expression of type `N` at scope `'s`.
pub trait CoExpr<'s, N: Neg> {}

// -- Variables are expressions (axiom rule) ----------------------------------

impl<'s, A: Pos> Expr<'s, A> for crate::var::Var<'s, A> {}
impl<'s, N: Neg> CoExpr<'s, N> for crate::var::Var<'s, N> {}

// -- Terms and coterms are expressions ---------------------------------------

impl<'s, A: Pos> Expr<'s, A> for Term<'s, A> {}

// -- Unit value is an expression ---------------------------------------------

impl<'s> Expr<'s, One> for () {}

// -- Tensor value (pair) is an expression ------------------------------------

impl<'s, A: Pos, B: Pos> Expr<'s, Tensor<A, B>> for (Term<'s, A>, Term<'s, B>)
where
    A::Dual: Neg,
    B::Dual: Neg,
{
}

// -- PlusIntro implements Expr ------------------------------------------------

impl<'s, A: Pos, B: Pos> Expr<'s, Plus<A, B>> for PlusIntro<'s, A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
}

// -- BangIntro implements Expr -----------------------------------------------

impl<'s, A: Pos> Expr<'s, Bang<A>> for BangIntro<'s, A> where A::Dual: Neg {}
