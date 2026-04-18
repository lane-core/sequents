use crate::types::{Bang, BangValue, Neg, One, Plus, PlusValue, Pos, Tensor};

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

impl<'s, A: Pos> Expr<'s, A> for crate::var::Var<'s, A> {}
impl<'s, N: Neg> CoExpr<'s, N> for crate::var::Var<'s, N> {}

// -- Unit value is an expression ---------------------------------------------

impl<'s> Expr<'s, One> for () {}

// -- Tensor value (pair) is an expression ------------------------------------

impl<'s, A: Pos, B: Pos> Expr<'s, Tensor<A, B>> for (A::Value<'s>, B::Value<'s>)
where
    A::Dual: Neg,
    B::Dual: Neg,
{
}

// -- PlusValue implements Expr ------------------------------------------------

impl<'s, A: Pos, B: Pos> Expr<'s, Plus<A, B>> for PlusValue<'s, A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
}

// -- BangValue implements Expr -----------------------------------------------

impl<'s, A: Pos> Expr<'s, Bang<A>> for BangValue<'s, A> where A::Dual: Neg {}
