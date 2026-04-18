//! Concrete examples exercising the sequents library.
//!
//! This file is NOT compiled as part of the library crate.  It exists as
//! documentation-by-example.  All code shown here has been verified to
//! compile when pasted into a suitable context.

use sequents::*;

struct X;
struct Y;

fn example_atomic_axiom() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let y: Var<'static, AtomN<X>> = Var::new();
    let _cmd: Command<'static> = cut(Expr::var(x), CoExpr::co_val(y));
}

fn example_mu_neg() {
    // μy⁻.⟨y | a⟩ where a : X⊥ is free.
    // Covariance allows the 'static variable a to be used at the
    // shorter binder lifetime '_.
    let a: Var<'static, AtomN<X>> = Var::new();
    let _co: CoExpr<'static, AtomN<X>> =
        mu_neg(|y: Var<'_, AtomP<X>>| cut(Expr::var(y), CoExpr::co_val(a)));
}

fn example_tensor() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let y: Var<'static, AtomP<Y>> = Var::new();
    let p = tensor::<'static, AtomP<X>, AtomP<Y>>(x, y);
    let _expr: Expr<'static, Tensor<AtomP<X>, AtomP<Y>>> = Expr::val(p);
}

fn example_par() {
    // mu_par introduces two variables. Both must be used exactly once.
    //
    // **Friction**: nested binders that capture outer variables do NOT
    // compile.  The `for<'x>` higher-rank bound requires closures to be
    // valid for all lifetimes 'x, which forbids capturing non-'static
    // data.  So the body cannot nest `mu_neg` and capture `y` from the
    // outer `mu_par` scope.
    let _co: CoExpr<'static, Par<AtomN<X>, AtomN<Y>>> = mu_par::<AtomP<X>, AtomP<Y>, _>(|x, y| {
        let a: Var<'_, AtomN<X>> = Var::new();
        let b: Var<'_, AtomN<Y>> = Var::new();
        let cmd_x = cut(Expr::var(x), CoExpr::co_val(a));
        let _ = cmd_x;
        cut(Expr::var(y), CoExpr::co_val(b))
    });
}

fn example_unit_and_bottom() {
    let u = unit();
    let _expr: Expr<'static, One> = Expr::val(u);
    let _bot: CoExpr<'static, Bot> = mu_unit(|| Command::new());
}

fn example_reduce_mu_neg() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let co: CoExpr<'static, AtomN<X>> = mu_neg(|y: Var<'_, AtomP<X>>| {
        let a: Var<'_, AtomN<X>> = Var::new();
        cut(Expr::var(y), CoExpr::co_val(a))
    });
    let body = match co {
        CoExpr::MuNeg(b) => b,
        _ => panic!("expected MuNeg"),
    };
    let _cmd: Command<'static> = reduce_mu_neg_atom(x, body);
}

fn example_reduce_tensor_par() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let y: Var<'static, AtomP<Y>> = Var::new();
    let co: CoExpr<'static, Par<AtomN<X>, AtomN<Y>>> = mu_par::<AtomP<X>, AtomP<Y>, _>(|a, b| {
        let m: Var<'_, AtomN<X>> = Var::new();
        let n: Var<'_, AtomN<Y>> = Var::new();
        let cmd_a = cut(Expr::var(a), CoExpr::co_val(m));
        let _ = cmd_a;
        cut(Expr::var(b), CoExpr::co_val(n))
    });
    let body = match co {
        CoExpr::MuPar(boxed) => match boxed.downcast::<MuPar<AtomP<X>, AtomP<Y>>>() {
            Ok(b) => *b,
            Err(_) => panic!("downcast failed"),
        },
        _ => panic!("expected MuPar"),
    };
    let _cmd: Command<'static> = reduce_tensor_par_atom(x, y, body);
}

fn main() {
    example_atomic_axiom();
    example_mu_neg();
    example_tensor();
    example_par();
    example_unit_and_bottom();
    example_reduce_mu_neg();
    example_reduce_tensor_par();
}
