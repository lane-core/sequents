use std::sync::Arc;

use sequents::*;

// =============================================================================
// User-defined atomic types
// =============================================================================

#[derive(Clone, Copy)]
pub struct X;
pub struct XTag;
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

#[derive(Clone, Copy)]
pub struct Y;
pub struct YTag;
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

#[test]
fn atomic_axiom() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let _cmd: Command<'static> = cut(x.into(), y.into());
}

#[test]
fn unit_axiom() {
    let u = unit();
    let co = mu_unit(|| Command::Normal);
    let _cmd: Command<'static> = cut(u, co);
}

#[test]
fn tensor_intro() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);
    let _pair = tensor::<X, Y>(x, y);
}

#[test]
fn mu_tilde_binder() {
    let a: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let _co = mu_tilde::<'static, XTag>(|y: Term<'_, X>| cut(y, a.into()));
}

#[test]
fn par_destructor() {
    let _co = mu_par::<'static, X, Y>(|x, y| {
        let a: CoResource<'_, XTag> = CoResource::new(XElim {
            body: Box::new(|_| Command::Normal),
        });
        let b: CoResource<'_, YTag> = CoResource::new(YElim {
            body: Box::new(|_| Command::Normal),
        });
        let cmd1 = cut(x, a.into());
        let _ = cmd1;
        cut(y, b.into())
    });
}

#[test]
fn nested_binders() {
    let _w: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let _co = mu_par::<'static, X, Y>(|x, y| {
        cut(x, mu_tilde::<'_, XTag>(|_z: Term<'_, X>| cut(y, _w.into())))
    });
}

#[test]
fn triple_nested() {
    let _w: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let _co = mu_par::<'static, X, Y>(|x, y| {
        cut(
            x,
            mu_tilde::<'_, XTag>(|_z| {
                cut(
                    y,
                    mu_tilde::<'_, YTag>(|_a| {
                        let b: CoResource<'_, YTag> = CoResource::new(YElim {
                            body: Box::new(|_| Command::Normal),
                        });
                        cut(_a, b.into())
                    }),
                )
            }),
        )
    });
}

#[test]
fn tensor_par_reduce_cut() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);
    let pair = tensor::<X, Y>(x, y);

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

#[test]
fn duality_involution() {
    fn check<P: Positive>() {}
    check::<X>();
    check::<One>();
    check::<Tensor<X, Y>>();
}

// ==========================================================================
// Operational tests
// ==========================================================================

/// Atomic cut reduces: `cut(x, μ̃y.⟨y | z⟩)` steps to `⟨x | z⟩`.
#[test]
fn atomic_cut_reduces() {
    let x: Resource<'static, X> = Resource::new(X);
    let z: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    let coterm = mu_tilde::<'static, XTag>(|y: Term<'_, X>| cut(y, z.into()));
    let cmd = cut(Term::Axiom(x), coterm);

    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Unit cut reduces: `cut((), μ̃().c)` steps to `c`.
#[test]
fn unit_cut_reduces() {
    let marker = std::rc::Rc::new(std::cell::Cell::new(false));
    let marker2 = marker.clone();

    let coterm = mu_unit(move || {
        marker2.set(true);
        Command::Normal
    });
    let cmd = cut(unit(), coterm);

    let outcome = run(cmd);
    assert!(marker.get());
    assert!(matches!(outcome, Command::Normal));
}

/// Tensor-par cut reduces for atoms.
#[test]
fn tensor_par_reduce_atoms() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);

    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let n: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let coterm = mu_par::<'static, X, Y>(|a, b| {
        let cmd1 = cut(a, mu_tilde::<'_, XTag>(|v| cut(v, m.into())));
        let _ = cmd1;
        cut(b, mu_tilde::<'_, YTag>(|v| cut(v, n.into())))
    });

    let pair = tensor(x, y);
    let cmd = cut(pair, coterm);

    let _outcome = run(cmd);
}

/// Tensor-par cut reduces for composites.
#[test]
fn tensor_par_reduce_composites() {
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

    let m: CoResource<'static, ATag> = CoResource::new(AElim {
        body: Box::new(|_| Command::Normal),
    });
    let n: CoResource<'static, BTag> = CoResource::new(BElim {
        body: Box::new(|_| Command::Normal),
    });
    let p: CoResource<'static, CTag> = CoResource::new(CElim {
        body: Box::new(|_| Command::Normal),
    });
    let q: CoResource<'static, DTag> = CoResource::new(DElim {
        body: Box::new(|_| Command::Normal),
    });

    let pair = tensor(tensor(a, b), tensor(c, d));

    let coterm = mu_par::<'static, Tensor<A, B>, Tensor<C, D>>(|x, y| {
        cut(
            x,
            mu_par::<'_, A, B>(|a1, b1| {
                cut(
                    y,
                    mu_par::<'_, C, D>(|c1, d1| {
                        let _ = cut(a1, mu_tilde::<'_, ATag>(|v| cut(v, m.into())));
                        let _ = cut(b1, mu_tilde::<'_, BTag>(|v| cut(v, n.into())));
                        let _ = cut(c1, mu_tilde::<'_, CTag>(|v| cut(v, p.into())));
                        cut(d1, mu_tilde::<'_, DTag>(|v| cut(v, q.into())))
                    }),
                )
            }),
        )
    });

    let cmd = cut(pair, coterm);
    let _outcome = run(cmd);
}

/// Multi-step reduction through nested atomic cuts.
#[test]
fn multi_step_atomic() {
    let x: Resource<'static, X> = Resource::new(X);
    let w: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    let cmd = cut(
        Term::Axiom(x),
        mu_tilde::<'static, XTag>(|y| cut(y, mu_tilde::<'_, XTag>(|z| cut(z, w.into())))),
    );

    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Par commuting conversion: `⟨μα.c | μ̃(x ⅋ y).d⟩` reduces.
#[test]
fn commuting_par() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);
    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let n: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let pos = mu::<'static, Tensor<X, Y>>(|a: Coterm<'_, Par<XTag, YTag>>| match a {
        Coterm::Elim(k) => k(Term::Axiom(x), Term::Axiom(y)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_par::<'static, X, Y>(|a, b| {
        let cmd1 = cut(a, mu_tilde::<'_, XTag>(|v| cut(v, m.into())));
        let _ = cmd1;
        cut(b, mu_tilde::<'_, YTag>(|v| cut(v, n.into())))
    });
    let cmd = cut(pos, neg);
    let _outcome = run(cmd);
}

/// Plus/with commuting conversion: `⟨μα.c | μ̃case(...).d⟩` reduces.
#[test]
fn commuting_plus_with_left() {
    let x: Resource<'static, X> = Resource::new(X);
    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let n: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let pos = mu::<'static, Plus<X, Y>>(move |a| match a {
        Coterm::Elim((left, _)) => left(Term::Axiom(x)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_case::<'static, X, Y>(
        |a| cut(a, mu_tilde::<'_, XTag>(|v| cut(v, m.into()))),
        |_b| cut(_b, mu_tilde::<'_, YTag>(|v| cut(v, n.into()))),
    );
    let cmd = cut(pos, neg);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Plus/with commuting conversion, right branch.
#[test]
fn commuting_plus_with_right() {
    let y: Resource<'static, Y> = Resource::new(Y);
    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let n: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let pos = mu::<'static, Plus<X, Y>>(move |a| match a {
        Coterm::Elim((_, right)) => right(Term::Axiom(y)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_case::<'static, X, Y>(
        |_a| cut(_a, mu_tilde::<'_, XTag>(|v| cut(v, m.into()))),
        |b| cut(b, mu_tilde::<'_, YTag>(|v| cut(v, n.into()))),
    );
    let cmd = cut(pos, neg);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Bang/whynot commuting conversion: `⟨μα.c | μ̃!x.d⟩` reduces.
#[test]
fn commuting_bang_whynot() {
    let bang = promote::<'static, X>(X);
    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    let pos = mu::<'static, Bang<X>>(move |a| match a {
        Coterm::Elim(k) => k(bang.clone()),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_bang::<'static, X>(|b| {
        let v = derelict::<X>(b);
        cut(v, m.into())
    });
    let cmd = cut(pos, neg);
    let _outcome = run(cmd);
}

/// Mixed reduction: multiple commuting conversions composed with reduce cuts.
#[test]
fn mixed_reduction() {
    let x1: Resource<'static, X> = Resource::new(X);
    let y1: Resource<'static, Y> = Resource::new(Y);
    let x2: Resource<'static, X> = Resource::new(X);
    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    let pos2 = mu::<'static, Plus<X, Y>>(move |a| match a {
        Coterm::Elim((left, _)) => left(Term::Axiom(x2)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg2 = mu_case::<'static, X, Y>(
        |a| cut(a, mu_tilde::<'_, XTag>(|v| cut(v, m.into()))),
        |_b| panic!("wrong branch"),
    );

    let pos = mu::<'static, Tensor<X, Y>>(move |a| match a {
        Coterm::Elim(k) => k(Term::Axiom(x1), Term::Axiom(y1)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_par::<'static, X, Y>(|_a, _b| cut(pos2, neg2));
    let cmd = cut(pos, neg);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Par commuting conversion with composites.
#[test]
fn commuting_par_composites() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);
    let m: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let n: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let inner = tensor(x, y);

    let pos = mu::<'static, Tensor<Tensor<X, Y>, One>>(move |a| match a {
        Coterm::Elim(k) => k(inner, unit()),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_par::<'static, Tensor<X, Y>, One>(|a, _b| {
        cut(
            a,
            mu_par::<'_, X, Y>(|a1, b1| {
                let cmd1 = cut(a1, mu_tilde::<'_, XTag>(|v| cut(v, m.into())));
                let _ = cmd1;
                cut(b1, mu_tilde::<'_, YTag>(|v| cut(v, n.into())))
            }),
        )
    });
    let cmd = cut(pos, neg);
    let _outcome = run(cmd);
}

/// Bot commuting conversion: `⟨μα.c | μ̃().d⟩` reduces.
#[test]
fn commuting_unit() {
    let marker = std::rc::Rc::new(std::cell::Cell::new(false));
    let marker2 = marker.clone();

    let pos = mu::<'static, One>(|a: Coterm<'_, Bot>| match a {
        Coterm::Elim(k) => k(),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_unit(move || {
        marker2.set(true);
        Command::Normal
    });
    let cmd = cut(pos, neg);

    let outcome = run(cmd);
    assert!(marker.get());
    assert!(matches!(outcome, Command::Normal));
}

/// Positive atomic cut: `cut(μα.⟨x | α⟩, z)` steps to `⟨x | z⟩`.
#[test]
fn positive_atomic_cut() {
    let x: Resource<'static, X> = Resource::new(X);
    let z: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    let pos = mu::<'static, X>(|a: Coterm<'_, XTag>| match a {
        Coterm::Axiom(v) => cut(Term::Axiom(x), Coterm::Axiom(v)),
        Coterm::Elim(cont) => (cont.body)(x.into_witness()),
        Coterm::MuTilde(f) => f(Term::Axiom(x)),
    });
    let cmd = cut(pos, z.into());

    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Nested binders reduce correctly.
#[test]
fn nested_binders_reduce() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);
    let free: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let cmd = cut(
        tensor(x, y),
        mu_par::<'static, X, Y>(|a, b| {
            cut(
                a,
                mu_tilde::<'_, XTag>(|_z| cut(b, mu_tilde::<'_, YTag>(|_a| cut(_a, free.into())))),
            )
        }),
    );

    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

// ==========================================================================
// Additive connectives
// ==========================================================================

#[test]
fn additive_duality() {
    fn check<P: Positive>() {}
    check::<Plus<X, Y>>();
    fn check_neg<N: Negative>() {}
    check_neg::<With<XTag, YTag>>();
}

/// Reduction cut with left injection: takes the left branch.
#[test]
fn plus_left_reduce() {
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
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Reduction cut with right injection: takes the right branch.
#[test]
fn plus_right_reduce() {
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
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

// ==========================================================================
// Exponential connectives
// ==========================================================================

#[test]
fn exponential_duality() {
    fn check<P: Positive>() {}
    check::<Bang<X>>();
    fn check_neg<N: Negative>() {}
    check_neg::<Whynot<XTag>>();
}

#[test]
fn exponential_bang_value_clone() {
    let bang = promote::<'static, X>(X);
    let _clone = bang.clone();
}

#[test]
fn exponential_promote() {
    let bang: Arc<X> = promote::<'static, X>(X);
    let _ = bang;
}

#[test]
fn exponential_derelict() {
    let bang: Arc<X> = promote::<'static, X>(X);
    let _v1: Term<'static, X> = derelict::<'static, X>(bang.clone());
    let _v2: Term<'static, X> = derelict::<'static, X>(bang);
}

/// Exponential cut — body receives Arc and can use it zero times (weakening).
#[test]
fn exponential_cut_weakening() {
    let bang = promote::<'static, X>(X);
    let coterm = mu_bang::<'static, X>(|_b: Arc<X>| Command::Normal);
    let cmd = cut(Term::<Bang<X>>::Intro(bang), coterm);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

#[test]
fn exponential_cut_single_use() {
    let z: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let bang = promote::<'static, X>(X);
    let coterm = mu_bang::<'static, X>(|b: Arc<X>| {
        let v = derelict::<'static, X>(b);
        cut(v, z.into())
    });
    let cmd = cut(Term::<Bang<X>>::Intro(bang), coterm);
    let _outcome = run(cmd);
}

#[test]
fn exponential_cut_contraction() {
    let z1: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let z2: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let bang = promote::<'static, X>(X);
    let coterm = mu_bang::<'static, X>(|b: Arc<X>| {
        let v1 = derelict::<'static, X>(b.clone());
        let v2 = derelict::<'static, X>(b);
        let cmd1 = cut(v1, z1.into());
        let _ = cmd1;
        cut(v2, z2.into())
    });
    let cmd = cut(Term::<Bang<X>>::Intro(bang), coterm);
    let _outcome = run(cmd);
}

/// Composite classical types.
#[test]
fn exponential_composite() {
    let z: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let w: CoResource<'static, YTag> = CoResource::new(YElim {
        body: Box::new(|_| Command::Normal),
    });

    let bang = promote::<'static, Tensor<X, Y>>((X, Y));

    let coterm = mu_bang::<'static, Tensor<X, Y>>(|b: Arc<(X, Y)>| {
        let pair = derelict::<'static, Tensor<X, Y>>(b);
        cut(
            pair,
            mu_par::<'_, X, Y>(|a, b_val| {
                let cmd1 = cut(a, z.into());
                let _ = cmd1;
                cut(b_val, w.into())
            }),
        )
    });

    let cmd = cut(Term::<Bang<Tensor<X, Y>>>::Intro(bang), coterm);
    let _outcome = run(cmd);
}

// ==========================================================================
// Canonical form / commuting conversion tests
// ==========================================================================

/// Atomic commuting conversion: `⟨μα.c | μ̃x.d⟩` reduces.
#[test]
fn atomic_commuting_conversion() {
    let x: Resource<'static, X> = Resource::new(X);
    let z: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });

    let pos = mu::<'static, X>(|a: Coterm<'_, XTag>| match a {
        Coterm::Axiom(v) => cut(Term::Axiom(x), Coterm::Axiom(v)),
        Coterm::Elim(cont) => (cont.body)(x.into_witness()),
        Coterm::MuTilde(f) => f(Term::Axiom(x)),
    });

    let coterm = Coterm::Elim(XElim {
        body: Box::new(move |t: X| {
            let term: Term<'_, X> = Term::Axiom(Resource::new(t));
            match term {
                Term::Axiom(v) => cut(Term::Axiom(v), z.into()),
                Term::Intro(i) => match i {},
                Term::Mu(_) => panic!("mu in atomic commuting"),
            }
        }),
    });
    let cmd = cut::<X>(pos, coterm);

    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Mu vs Var for unit type: `⟨μα.c | x⊥⟩` where `x⊥ : ⊥`.
#[test]
fn cut_mu_unit_axiom() {
    let marker = std::rc::Rc::new(std::cell::Cell::new(false));
    let marker2 = marker.clone();
    let x: CoResource<'static, Bot> = CoResource::new(());

    let pos = mu::<'static, One>(move |a: Coterm<'_, Bot>| match a {
        Coterm::Axiom(_) => {
            marker2.set(true);
            Command::Normal
        }
        Coterm::Elim(_) => panic!("elim branch"),
        Coterm::MuTilde(_) => panic!("mu_tilde branch"),
    });

    let cmd = cut(pos, x.into());
    let outcome = run(cmd);
    assert!(marker.get());
    assert!(matches!(outcome, Command::Normal));
}

/// Mu vs Var for tensor type: `⟨μα.c | x⊥⟩` where `x⊥ : A⊥ ⅋ B⊥`.
#[test]
fn cut_mu_par_axiom() {
    let x: CoResource<'static, Par<XTag, YTag>> =
        CoResource::new(Box::new(|_: XElim<'static>, _: YElim<'static>| ())
            as Box<dyn FnOnce(XElim<'static>, YElim<'static>)>);

    let pos = mu::<'static, Tensor<X, Y>>(|a: Coterm<'_, Par<XTag, YTag>>| match a {
        Coterm::Axiom(_) => Command::Normal,
        Coterm::Elim(_) => panic!("elim branch"),
        Coterm::MuTilde(_) => panic!("mu_tilde branch"),
    });

    let cmd = cut(pos, x.into());
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Mu vs Var for plus type: `⟨μα.c | x⊥⟩` where `x⊥ : A⊥ & B⊥`.
#[test]
fn cut_mu_plus_axiom() {
    let x: CoResource<'static, With<XTag, YTag>> = CoResource::new((
        XElim {
            body: Box::new(|_| Command::Normal),
        },
        YElim {
            body: Box::new(|_| Command::Normal),
        },
    ));

    let pos = mu::<'static, Plus<X, Y>>(|a: Coterm<'_, With<XTag, YTag>>| match a {
        Coterm::Axiom(_) => Command::Normal,
        Coterm::Elim(_) => panic!("elim branch"),
        Coterm::MuTilde(_) => panic!("mu_tilde branch"),
    });

    let cmd = cut(pos, x.into());
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Mu vs Var for bang type: `⟨μα.c | x⊥⟩` where `x⊥ : ?A⊥`.
#[test]
fn cut_mu_bang_axiom() {
    let x: CoResource<'static, Whynot<XTag>> =
        CoResource::new(Box::new(|_: XElim<'static>| ()) as Box<dyn FnMut(XElim<'static>)>);

    let pos = mu::<'static, Bang<X>>(|a: Coterm<'_, Whynot<XTag>>| match a {
        Coterm::Axiom(_) => Command::Normal,
        Coterm::Elim(_) => panic!("elim branch"),
        Coterm::MuTilde(_) => panic!("mu_tilde branch"),
    });

    let cmd = cut(pos, x.into());
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

// ==========================================================================
// Canonical form production (explicit tests for Command::Normal / Step)
// ==========================================================================

/// Axiom vs Axiom produces Normal immediately.
#[test]
fn canonical_axiom_vs_axiom() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: CoResource<'static, XTag> = CoResource::new(XElim {
        body: Box::new(|_| Command::Normal),
    });
    let cmd = cut(Term::Axiom(x), Coterm::Axiom(y));
    assert!(matches!(cmd, Command::Normal));
}

/// Axiom vs Elim (composite) produces Step wrapping Normal — blocked.
#[test]
fn canonical_axiom_vs_elim_composite() {
    let x: Resource<'static, Tensor<X, Y>> = Resource::new((X, Y));
    let coterm = mu_par::<'static, X, Y>(|_a, _b| Command::Normal);
    let cmd = cut(Term::Axiom(x), coterm);
    assert!(
        matches!(cmd, Command::Step(_)),
        "axiom vs elim should produce a Step"
    );
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Intro vs Axiom produces Normal — blocked, no reduction.
#[test]
fn canonical_intro_vs_axiom() {
    let x: Resource<'static, X> = Resource::new(X);
    let y: Resource<'static, Y> = Resource::new(Y);
    let z: CoResource<'static, Par<XTag, YTag>> =
        CoResource::new(Box::new(|_: XElim<'static>, _: YElim<'static>| ())
            as Box<dyn FnOnce(XElim<'static>, YElim<'static>)>);
    let term = tensor(x, y);
    let cmd = cut(term, Coterm::Axiom(z));
    assert!(matches!(cmd, Command::Normal));
}

/// Intro vs Elim (reduce) produces Step, reducing on run.
#[test]
fn canonical_intro_vs_elim_step() {
    let marker = std::rc::Rc::new(std::cell::Cell::new(false));
    let marker2 = marker.clone();
    let coterm = mu_unit(move || {
        marker2.set(true);
        Command::Normal
    });
    let cmd = cut(unit(), coterm);
    // Step wrapper — body not yet executed.
    assert!(
        matches!(cmd, Command::Step(_)),
        "reduce cut should produce a Step"
    );
    assert!(!marker.get());
    let outcome = run(cmd);
    // After run, body executed and marker set.
    assert!(marker.get());
    assert!(matches!(outcome, Command::Normal));
}

/// Mu vs anything produces Step.
#[test]
fn canonical_mu_vs_any_step() {
    let coterm = mu_unit(|| Command::Normal);
    let pos = mu::<'static, One>(|_c| Command::Normal);
    let cmd = cut(pos, coterm);
    assert!(
        matches!(cmd, Command::Step(_)),
        "mu cut should produce a Step"
    );
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Axiom vs MuTilde produces Step (commuting conversion).
#[test]
fn canonical_axiom_vs_mu_tilde_step() {
    let x: Resource<'static, X> = Resource::new(X);
    let coterm = mu_tilde::<'static, XTag>(|_t: Term<'_, X>| Command::Normal);
    let cmd = cut(Term::Axiom(x), coterm);
    assert!(
        matches!(cmd, Command::Step(_)),
        "axiom vs mu_tilde should produce a Step"
    );
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Intro vs MuTilde produces Step (commuting conversion).
#[test]
fn canonical_intro_vs_mu_tilde_step() {
    let coterm = mu_tilde::<'static, Bot>(|_t: Term<'_, One>| Command::Normal);
    let cmd = cut(unit(), coterm);
    assert!(
        matches!(cmd, Command::Step(_)),
        "intro vs mu_tilde should produce a Step"
    );
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Witness observability: a concrete `u64` value flows through reduction.
#[test]
fn witness_observability() {
    pub struct U64Val(pub u64);
    pub struct U64Tag;
    pub struct U64Elim<'s> {
        pub body: Box<dyn FnOnce(U64Val) -> Command<'s> + 's>,
    }

    impl Positive for U64Val {
        type Dual = U64Tag;
        type Intro<'s> = std::convert::Infallible;
        type Witness<'x> = U64Val;
        fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
            match i {}
        }
    }

    impl Negative for U64Tag {
        type Dual = U64Val;
        type Elim<'s> = U64Elim<'s>;
        type Witness<'x> = U64Elim<'x>;
        fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
            (e.body)(r.into_witness())
        }
    }

    let marker = std::rc::Rc::new(std::cell::Cell::new(0u64));
    let marker2 = marker.clone();

    let x: Resource<'static, U64Val> = Resource::new(U64Val(42));
    let coterm = mu_tilde::<'static, U64Tag>(move |t: Term<'_, U64Val>| match t {
        Term::Axiom(r) => {
            let v = r.into_witness();
            marker2.set(v.0);
            Command::Normal
        }
        Term::Intro(i) => match i {},
        Term::Mu(_) => panic!("mu in witness test"),
    });

    let cmd = cut(Term::Axiom(x), coterm);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
    assert_eq!(marker.get(), 42);
}
