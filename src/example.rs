//! Concrete examples exercising the sequents library (Option A).
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
    let _cmd: Command<'static> = cut(x, y);
}

fn example_mu_neg() {
    let a: Var<'static, AtomN<X>> = Var::new();
    let _co = mu_neg::<'static, AtomN<X>, _>(|y: Var<'_, AtomP<X>>| cut(y, a));
}

fn example_tensor() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let y: Var<'static, AtomP<Y>> = Var::new();
    let _pair = tensor::<AtomP<X>, AtomP<Y>>(x, y);
}

fn example_par() {
    let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
        let a: Var<'_, AtomN<X>> = Var::new();
        let b: Var<'_, AtomN<Y>> = Var::new();
        let cmd_x = cut(x, a);
        let _ = cmd_x;
        cut(y, b)
    });
}

fn example_nested_binders() {
    // μ(x ⅋ y).⟨x | μz⁻.⟨y | w⟩⟩
    let _w: Var<'static, AtomN<Y>> = Var::new();

    let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
        cut(
            x,
            mu_neg::<'_, AtomN<X>, _>(|_z: Var<'_, AtomP<X>>| cut(y, _w)),
        )
    });
}

fn example_unit_and_bottom() {
    let u = unit();
    let _expr = u; // () implements Expr<'s, One>
    let _bot = mu_unit(|| Command::new());
}

fn example_tensor_par_cut() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let y: Var<'static, AtomP<Y>> = Var::new();
    let pair = tensor::<AtomP<X>, AtomP<Y>>(x, y);

    let co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
        let m: Var<'_, AtomN<X>> = Var::new();
        let n: Var<'_, AtomN<Y>> = Var::new();
        let cmd1 = cut(a, m);
        let _ = cmd1;
        cut(b, n)
    });

    let _cmd: Command<'static> = cut(pair, co);
}

fn main() {
    example_atomic_axiom();
    example_mu_neg();
    example_tensor();
    example_par();
    example_nested_binders();
    example_unit_and_bottom();
    example_tensor_par_cut();
}
