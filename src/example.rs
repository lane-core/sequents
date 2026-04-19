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
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomN<X>> = Resource::new();
    let _cmd: Command<'static> = cut(x.into(), y.into());
}

fn example_mu_tilde() {
    let a: Resource<'static, AtomN<X>> = Resource::new();
    let _co = mu_tilde::<'static, AtomN<X>>(|y: Term<'_, AtomP<X>>| cut(y, a.into()));
}

fn example_tensor() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let _pair = tensor::<AtomP<X>, AtomP<Y>>(x, y);
}

fn example_par() {
    let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
        let a: Resource<'_, AtomN<X>> = Resource::new();
        let b: Resource<'_, AtomN<Y>> = Resource::new();
        let cmd_x = cut(x, a.into());
        let _ = cmd_x;
        cut(y, b.into())
    });
}

fn example_nested_binders() {
    // μ̃(x ⅋ y).⟨x | μ̃z.⟨y | w⟩⟩
    let _w: Resource<'static, AtomN<Y>> = Resource::new();

    let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
        cut(
            x,
            mu_tilde::<'_, AtomN<X>>(|_z: Term<'_, AtomP<X>>| cut(y, _w.into())),
        )
    });
}

fn example_unit_and_bottom() {
    let u = unit();
    let _expr = u; // Term<'s, One>
    let _bot = mu_unit(|| Command::Normal);
}

fn example_tensor_par_principal() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let pair = tensor(x, y);

    let co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
        let m: Resource<'_, AtomN<X>> = Resource::new();
        let n: Resource<'_, AtomN<Y>> = Resource::new();
        let cmd1 = cut(a, m.into());
        let _ = cmd1;
        cut(b, n.into())
    });

    let _cmd: Command<'static> = cut(pair, co);
}

// ============================================================================
// Reducing terms
// ============================================================================

fn example_atomic_reduction() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let z: Resource<'static, AtomN<X>> = Resource::new();

    let coterm = mu_tilde::<'static, AtomN<X>>(|y: Term<'_, AtomP<X>>| cut(y, z.into()));
    let cmd = cut(Term::Axiom(x), coterm);

    // After running, we get Normal — the canonical form ⟨x | z⟩.
    let _outcome = run(cmd);
}

fn example_unit_reduction() {
    let coterm = mu_unit(|| Command::Normal);
    let cmd = cut(unit(), coterm);

    let _outcome = run(cmd);
}

fn example_composite_reduction() {
    struct A;
    struct B;
    struct C;
    struct D;

    let a: Resource<'static, AtomP<A>> = Resource::new();
    let b: Resource<'static, AtomP<B>> = Resource::new();
    let c: Resource<'static, AtomP<C>> = Resource::new();
    let d: Resource<'static, AtomP<D>> = Resource::new();

    let pair = tensor(tensor(a, b), tensor(c, d));

    let coterm =
        mu_par::<'static, Tensor<AtomP<A>, AtomP<B>>, Tensor<AtomP<C>, AtomP<D>>, _>(|x, y| {
            cut(
                x,
                mu_par::<'_, AtomP<A>, AtomP<B>, _>(|_a1, _b1| {
                    cut(
                        y,
                        mu_par::<'_, AtomP<C>, AtomP<D>, _>(|_c1, _d1| Command::Normal),
                    )
                }),
            )
        });

    let cmd = cut(pair, coterm);
    let _outcome = run(cmd);
}

fn example_multi_step() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let w: Resource<'static, AtomN<X>> = Resource::new();

    let cmd = cut(
        Term::Axiom(x),
        mu_tilde::<'static, AtomN<X>>(|y| {
            cut(y, mu_tilde::<'_, AtomN<X>>(|z| cut(z.into(), w.into())))
        }),
    );

    // Reduces in two steps to cut(x, w), then Normal.
    let _outcome = run(cmd);
}

// ============================================================================
// Positive cuts (commuting conversions)
// ============================================================================

fn example_positive_atomic_cut() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let z: Resource<'static, AtomN<X>> = Resource::new();

    // μ⁺α.⟨x | α⟩  cut against  z
    let pos = mu::<'static, AtomP<X>>(|a: Coterm<'_, AtomN<X>>| match a {
        Coterm::Axiom(v) => cut(Term::Axiom(x), Coterm::Axiom(v)),
        Coterm::Elim(cont) => (cont.body)(Term::Axiom(x)),
        Coterm::MuTilde(f) => f(Term::Axiom(x)),
    });
    let cmd = cut(pos, z.into());

    let _outcome = run(cmd); // Normal — canonical form ⟨x | z⟩
}

// ============================================================================
// Additive connectives
// ============================================================================

fn example_additive_left() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let val = inl::<AtomP<X>, AtomP<Y>>(x);
    let coterm = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
        |a| cut(a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into()))),
        |_b| cut(_b, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into()))),
    );

    let cmd = cut(val, coterm);
    let _outcome = run(cmd); // Takes left branch
}

fn example_additive_right() {
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let val = inr::<AtomP<X>, AtomP<Y>>(y);
    let coterm = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
        |_a| cut(_a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into()))),
        |b| cut(b, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into()))),
    );

    let cmd = cut(val, coterm);
    let _outcome = run(cmd); // Takes right branch
}

fn main() {
    example_atomic_axiom();
    example_mu_tilde();
    example_tensor();
    example_par();
    example_nested_binders();
    example_unit_and_bottom();
    example_tensor_par_principal();
    example_atomic_reduction();
    example_unit_reduction();
    example_composite_reduction();
    example_multi_step();
    example_positive_atomic_cut();
    example_additive_left();
    example_additive_right();
}
