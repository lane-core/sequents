//! Concrete examples exercising the sequents library.
//!
//! This file is NOT compiled as part of the library crate.  It exists as
//! documentation-by-example.  All code shown here has been verified to
//! compile when pasted into a suitable context.

use sequents::*;

struct X;
struct Y;

// ============================================================================
// Constructing well-formed terms
// ============================================================================

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
    let _bot = mu_unit(|| Command::stuck(StuckReason::StaticOnly));
}

fn example_tensor_par_cut() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let y: Var<'static, AtomP<Y>> = Var::new();
    let pair = (x, y);

    let co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
        let m: Var<'_, AtomN<X>> = Var::new();
        let n: Var<'_, AtomN<Y>> = Var::new();
        let cmd1 = cut(a, m);
        let _ = cmd1;
        cut(b, n)
    });

    let _cmd: Command<'static> = cut(pair, co);
}

// ============================================================================
// Reducing terms
// ============================================================================

fn example_atomic_reduction() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let z: Var<'static, AtomN<X>> = Var::new();

    let binder = mu_neg::<'static, AtomN<X>, _>(|y: Var<'_, AtomP<X>>| cut(y, z));
    let cmd = cut_atom(x, binder);

    // After running, we get Stuck(StaticOnly) — the normal form ⟨x | z⟩.
    let _outcome = run(cmd);
}

fn example_unit_reduction() {
    let binder = mu_unit(|| Command::stuck(StuckReason::Unexpected("done".into())));
    let cmd = cut_unit((), binder);

    let _outcome = run(cmd);
}

fn example_composite_reduction() {
    struct A;
    struct B;
    struct C;
    struct D;

    let a: Var<'static, AtomP<A>> = Var::new();
    let b: Var<'static, AtomP<B>> = Var::new();
    let c: Var<'static, AtomP<C>> = Var::new();
    let d: Var<'static, AtomP<D>> = Var::new();

    let pair = ((a, b), (c, d));

    let binder =
        mu_par::<'static, Tensor<AtomP<A>, AtomP<B>>, Tensor<AtomP<C>, AtomP<D>>, _>(|x, y| {
            cut_par::<'_, AtomP<A>, AtomP<B>, _>(
                x,
                mu_par::<'_, AtomP<A>, AtomP<B>, _>(|_a1, _b1| {
                    cut_par::<'_, AtomP<C>, AtomP<D>, _>(
                        y,
                        mu_par::<'_, AtomP<C>, AtomP<D>, _>(|_c1, _d1| {
                            Command::stuck(StuckReason::StaticOnly)
                        }),
                    )
                }),
            )
        });

    let cmd = cut_par(pair, binder);
    let _outcome = run(cmd);
}

fn example_multi_step() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let w: Var<'static, AtomN<X>> = Var::new();

    let cmd = cut_atom(
        x,
        mu_neg::<'static, AtomN<X>, _>(|y| cut_atom(y, mu_neg::<'_, AtomN<X>, _>(|z| cut(z, w)))),
    );

    // Reduces in two steps to cut(x, w), then Stuck(StaticOnly).
    let _outcome = run(cmd);
}

// ============================================================================
// Positive cuts
// ============================================================================

fn example_positive_atomic_cut() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let z: Var<'static, AtomN<X>> = Var::new();

    // μ⁺α.⟨x | α⟩  cut against  z
    let binder = mu_pos::<'static, AtomP<X>, _>(|a: Var<'_, AtomN<X>>| cut(x, a));
    let cmd = cut_pos_atom(binder, z);

    let _outcome = run(cmd); // Stuck(StaticOnly) — normal form ⟨x | z⟩
}

// ============================================================================
// Additive connectives
// ============================================================================

fn example_additive_left() {
    let x: Var<'static, AtomP<X>> = Var::new();
    let m: Var<'static, AtomN<X>> = Var::new();
    let n: Var<'static, AtomN<Y>> = Var::new();

    let val = PlusValue::<'static, AtomP<X>, AtomP<Y>>::Inl(x);
    let binder = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
        |a| cut_atom(a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m))),
        |_b| cut_atom(_b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n))),
    );

    let cmd = cut_plus(val, binder);
    let _outcome = run(cmd); // Takes left branch
}

fn example_additive_right() {
    let y: Var<'static, AtomP<Y>> = Var::new();
    let m: Var<'static, AtomN<X>> = Var::new();
    let n: Var<'static, AtomN<Y>> = Var::new();

    let val = PlusValue::<'static, AtomP<X>, AtomP<Y>>::Inr(y);
    let binder = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
        |_a| cut_atom(_a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m))),
        |b| cut_atom(b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n))),
    );

    let cmd = cut_plus(val, binder);
    let _outcome = run(cmd); // Takes right branch
}

fn main() {
    example_atomic_axiom();
    example_mu_neg();
    example_tensor();
    example_par();
    example_nested_binders();
    example_unit_and_bottom();
    example_tensor_par_cut();
    example_atomic_reduction();
    example_unit_reduction();
    example_composite_reduction();
    example_multi_step();
    example_positive_atomic_cut();
    example_additive_left();
    example_additive_right();
}
