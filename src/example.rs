//! Concrete examples exercising the sequents library.
//!
//! This file is NOT compiled as part of the library crate.  It exists as
//! documentation-by-example.  All code shown here has been verified to
//! compile when pasted into a suitable context.

use sequents::*;

struct X;
struct XTag;
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

struct Y;
struct YTag;
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

// ============================================================================
// Constructing well-formed terms
// ============================================================================

fn example_atomic_axiom() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let _cmd: Command<'static> = cut(x.into(), y.into());
}

fn example_mu_tilde() {
    let a: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let _co = mu_tilde::<'static, XTag>(|y: Term<'_, X>| cut(y, a.into()));
}

fn example_tensor() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);
    let _pair = tensor::<X, Y>(x, y);
}

fn example_par() {
    let _co = mu_par::<'static, X, Y>(|x, y| {
        let a: CoResource<'_, XTag> = CoResource::new(XElim {
            body: Box::new(|_| Command::Normal),
        });
        let b: CoResource<'_, YTag> = CoResource::new(YElim {
            body: Box::new(|_| Command::Normal),
        });
        let cmd_x = cut(x, a.into());
        let _ = cmd_x;
        cut(y, b.into())
    });
}

fn example_nested_binders() {
    // μ̃(x ⅋ y).⟨x | μ̃z.⟨y | w⟩⟩
    let _w: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let _co = mu_par::<'static, X, Y>(|x, y| {
        cut(x, mu_tilde::<'_, XTag>(|_z: Term<'_, X>| cut(y, _w.into())))
    });
}

fn example_unit_and_bottom() {
    let u = unit();
    let _expr = u; // Term<'s, One>
    let _bot = mu_unit(|| Command::Normal);
}

fn example_tensor_par_principal() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);
    let pair = tensor(x, y);

    let co = mu_par::<'static, X, Y>(|a, b| {
        let m: CoResource<'_, XTag> = CoResource::new(XElim {
            body: Box::new(|_| Command::Normal),
        });
        let n: CoResource<'_, YTag> = CoResource::new(YElim {
            body: Box::new(|_| Command::Normal),
        });
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
    let x: Resource<'static, X> = Resource::new(X);
    let z: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    let coterm = mu_tilde::<'static, XTag>(|y: Term<'_, X>| cut(y, z.into()));
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
    #[derive(Clone, Copy)]
    struct A;
    struct ATag;
    pub struct AElim<'s> {
        pub body: Box<dyn FnOnce(A) -> Command<'s> + 's>,
    }
    impl Positive for A {
        type Dual = ATag;
        type Intro<'s> = std::convert::Infallible;
        type Witness<'x> = A;
        fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
            match i {}
        }
    }
    impl Negative for ATag {
        type Dual = A;
        type Elim<'s> = AElim<'s>;
        type Witness<'x> = AElim<'x>;
        fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
            (e.body)(r.into_witness())
        }
    }

    #[derive(Clone, Copy)]
    struct B;
    struct BTag;
    pub struct BElim<'s> {
        pub body: Box<dyn FnOnce(B) -> Command<'s> + 's>,
    }
    impl Positive for B {
        type Dual = BTag;
        type Intro<'s> = std::convert::Infallible;
        type Witness<'x> = B;
        fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
            match i {}
        }
    }
    impl Negative for BTag {
        type Dual = B;
        type Elim<'s> = BElim<'s>;
        type Witness<'x> = BElim<'x>;
        fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
            (e.body)(r.into_witness())
        }
    }

    #[derive(Clone, Copy)]
    struct C;
    struct CTag;
    pub struct CElim<'s> {
        pub body: Box<dyn FnOnce(C) -> Command<'s> + 's>,
    }
    impl Positive for C {
        type Dual = CTag;
        type Intro<'s> = std::convert::Infallible;
        type Witness<'x> = C;
        fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
            match i {}
        }
    }
    impl Negative for CTag {
        type Dual = C;
        type Elim<'s> = CElim<'s>;
        type Witness<'x> = CElim<'x>;
        fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
            (e.body)(r.into_witness())
        }
    }

    #[derive(Clone, Copy)]
    struct D;
    struct DTag;
    pub struct DElim<'s> {
        pub body: Box<dyn FnOnce(D) -> Command<'s> + 's>,
    }
    impl Positive for D {
        type Dual = DTag;
        type Intro<'s> = std::convert::Infallible;
        type Witness<'x> = D;
        fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
            match i {}
        }
    }
    impl Negative for DTag {
        type Dual = D;
        type Elim<'s> = DElim<'s>;
        type Witness<'x> = DElim<'x>;
        fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
            (e.body)(r.into_witness())
        }
    }

    let a: Resource<'static, A> = Resource::new(A);
    let b: Resource<'static, B> = Resource::new(B);
    let c: Resource<'static, C> = Resource::new(C);
    let d: Resource<'static, D> = Resource::new(D);

    let pair = tensor(tensor(a, b), tensor(c, d));

    let coterm = mu_par::<'static, Tensor<A, B>, Tensor<C, D>>(|x, y| {
        cut(
            x,
            mu_par::<'_, A, B>(|_a1, _b1| cut(y, mu_par::<'_, C, D>(|_c1, _d1| Command::Normal))),
        )
    });

    let cmd = cut(pair, coterm);
    let _outcome = run(cmd);
}

fn example_multi_step() {
    let x: Resource<'static, X> = Resource::new(X);
    let w: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    let cmd = cut(
        Term::Axiom(x),
        mu_tilde::<'static, XTag>(|y| cut(y, mu_tilde::<'_, XTag>(|z| cut(z, w.into())))),
    );

    // Reduces in two steps to cut(x, w), then Normal.
    let _outcome = run(cmd);
}

// ============================================================================
// Positive cuts (commuting conversions)
// ============================================================================

fn example_positive_atomic_cut() {
    let x: Resource<'static, X> = Resource::new(X);
    let z: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    // μ⁺α.⟨x | α⟩  cut against  z
    let pos = mu::<'static, X>(|a: Coterm<'_, XTag>| match a {
        Coterm::Axiom(v) => cut(Term::Axiom(x), Coterm::Axiom(v)),
        Coterm::Elim(cont) => (cont.body)(x.into_witness()),
        Coterm::MuTilde(f) => f(Term::Axiom(x)),
    });
    let cmd = cut(pos, z.into());

    let _outcome = run(cmd); // Normal — canonical form ⟨x | z⟩
}

// ============================================================================
// Additive connectives
// ============================================================================

fn example_additive_left() {
    let x: Resource<'static, X> = Resource::new(X);
    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let n: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let val = inl::<X, Y>(x);
    let coterm = mu_case::<'static, X, Y>(
        |a| cut(a, mu_tilde::<'_, XTag>(|v| cut(v, m.into()))),
        |_b| cut(_b, mu_tilde::<'_, YTag>(|v| cut(v, n.into()))),
    );

    let cmd = cut(val, coterm);
    let _outcome = run(cmd); // Takes left branch
}

fn example_additive_right() {
    let y: Resource<'static, Y> = Resource::new(Y);
    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let n: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let val = inr::<X, Y>(y);
    let coterm = mu_case::<'static, X, Y>(
        |_a| cut(_a, mu_tilde::<'_, XTag>(|v| cut(v, m.into()))),
        |b| cut(b, mu_tilde::<'_, YTag>(|v| cut(v, n.into()))),
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
