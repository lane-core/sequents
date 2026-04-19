use sequents::*;

struct X;
struct Y;

#[test]
fn atomic_axiom() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomN<X>> = Resource::new();
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
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let _pair = tensor::<AtomP<X>, AtomP<Y>>(x, y);
}

#[test]
fn mu_tilde_binder() {
    let a: Resource<'static, AtomN<X>> = Resource::new();
    let _co = mu_tilde::<'static, AtomN<X>>(|y: Term<'_, AtomP<X>>| cut(y, a.into()));
}

#[test]
fn par_destructor() {
    let _co = mu_par::<'static, AtomP<X>, AtomP<Y>>(|x, y| {
        let a: Resource<'_, AtomN<X>> = Resource::new();
        let b: Resource<'_, AtomN<Y>> = Resource::new();
        let cmd1 = cut(x, a.into());
        let _ = cmd1;
        cut(y, b.into())
    });
}

#[test]
fn nested_binders() {
    let _w: Resource<'static, AtomN<Y>> = Resource::new();

    let _co = mu_par::<'static, AtomP<X>, AtomP<Y>>(|x, y| {
        cut(
            x,
            mu_tilde::<'_, AtomN<X>>(|_z: Term<'_, AtomP<X>>| cut(y, _w.into())),
        )
    });
}

#[test]
fn triple_nested() {
    let _w: Resource<'static, AtomN<Y>> = Resource::new();

    let _co = mu_par::<'static, AtomP<X>, AtomP<Y>>(|x, y| {
        cut(
            x,
            mu_tilde::<'_, AtomN<X>>(|_z| {
                cut(
                    y,
                    mu_tilde::<'_, AtomN<Y>>(|_a| {
                        let b: Resource<'_, AtomN<Y>> = Resource::new();
                        cut(_a, b.into())
                    }),
                )
            }),
        )
    });
}

#[test]
fn tensor_par_reduce_cut() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let pair = tensor::<AtomP<X>, AtomP<Y>>(x, y);

    let co = mu_par::<'static, AtomP<X>, AtomP<Y>>(|a, b| {
        let m: Resource<'_, AtomN<X>> = Resource::new();
        let n: Resource<'_, AtomN<Y>> = Resource::new();
        let cmd1 = cut(a, m.into());
        let _ = cmd1;
        cut(b, n.into())
    });

    let _cmd: Command<'static> = cut(pair, co);
}

#[test]
fn duality_involution() {
    fn check<P: Positive>() {}
    check::<AtomP<X>>();
    check::<One>();
    check::<Tensor<AtomP<X>, AtomP<Y>>>();
}

// ==========================================================================
// Operational tests
// ==========================================================================

/// Atomic cut reduces: `cut(x, μ̃y.⟨y | z⟩)` steps to `⟨x | z⟩`.
#[test]
fn atomic_cut_reduces() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let z: Resource<'static, AtomN<X>> = Resource::new();

    let coterm = mu_tilde::<'static, AtomN<X>>(|y: Term<'_, AtomP<X>>| cut(y, z.into()));
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
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();

    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let coterm = mu_par::<'static, AtomP<X>, AtomP<Y>>(|a, b| {
        let cmd1 = cut(a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into())));
        let _ = cmd1;
        cut(b, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into())))
    });

    let pair = tensor(x, y);
    let cmd = cut(pair, coterm);

    let _outcome = run(cmd);
}

/// Tensor-par cut reduces for composites.
#[test]
fn tensor_par_reduce_composites() {
    struct A;
    struct B;
    struct C;
    struct D;

    let a: Resource<'static, AtomP<A>> = Resource::new();
    let b: Resource<'static, AtomP<B>> = Resource::new();
    let c: Resource<'static, AtomP<C>> = Resource::new();
    let d: Resource<'static, AtomP<D>> = Resource::new();

    let m: Resource<'static, AtomN<A>> = Resource::new();
    let n: Resource<'static, AtomN<B>> = Resource::new();
    let p: Resource<'static, AtomN<C>> = Resource::new();
    let q: Resource<'static, AtomN<D>> = Resource::new();

    let pair = tensor(tensor(a, b), tensor(c, d));

    let coterm =
        mu_par::<'static, Tensor<AtomP<A>, AtomP<B>>, Tensor<AtomP<C>, AtomP<D>>>(|x, y| {
            cut(
                x,
                mu_par::<'_, AtomP<A>, AtomP<B>>(|a1, b1| {
                    cut(
                        y,
                        mu_par::<'_, AtomP<C>, AtomP<D>>(|c1, d1| {
                            let _ = cut(a1, mu_tilde::<'_, AtomN<A>>(|v| cut(v, m.into())));
                            let _ = cut(b1, mu_tilde::<'_, AtomN<B>>(|v| cut(v, n.into())));
                            let _ = cut(c1, mu_tilde::<'_, AtomN<C>>(|v| cut(v, p.into())));
                            cut(d1, mu_tilde::<'_, AtomN<D>>(|v| cut(v, q.into())))
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
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let w: Resource<'static, AtomN<X>> = Resource::new();

    let cmd = cut(
        Term::Axiom(x),
        mu_tilde::<'static, AtomN<X>>(|y| cut(y, mu_tilde::<'_, AtomN<X>>(|z| cut(z, w.into())))),
    );

    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Par commuting conversion: `⟨μα.c | μ̃(x ⅋ y).d⟩` reduces.
#[test]
fn commuting_par() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let pos = mu::<'static, Tensor<AtomP<X>, AtomP<Y>>>(
        |a: Coterm<'_, Par<AtomN<X>, AtomN<Y>>>| match a {
            Coterm::Elim(k) => (k.body)(Term::Axiom(x), Term::Axiom(y)),
            Coterm::Axiom(_) => panic!("axiom in commuting"),
            Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
        },
    );
    let neg = mu_par::<'static, AtomP<X>, AtomP<Y>>(|a, b| {
        let cmd1 = cut(a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into())));
        let _ = cmd1;
        cut(b, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into())))
    });
    let cmd = cut(pos, neg);
    let _outcome = run(cmd);
}

/// Plus/with commuting conversion: `⟨μα.c | μ̃case(...).d⟩` reduces.
#[test]
fn commuting_plus_with_left() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let pos = mu::<'static, Plus<AtomP<X>, AtomP<Y>>>(move |a| match a {
        Coterm::Elim(WithElim { left, right: _ }) => left(Term::Axiom(x)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_case::<'static, AtomP<X>, AtomP<Y>>(
        |a| cut(a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into()))),
        |_b| cut(_b, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into()))),
    );
    let cmd = cut(pos, neg);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Plus/with commuting conversion, right branch.
#[test]
fn commuting_plus_with_right() {
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let pos = mu::<'static, Plus<AtomP<X>, AtomP<Y>>>(move |a| match a {
        Coterm::Elim(WithElim { left: _, right }) => right(Term::Axiom(y)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_case::<'static, AtomP<X>, AtomP<Y>>(
        |_a| cut(_a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into()))),
        |b| cut(b, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into()))),
    );
    let cmd = cut(pos, neg);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Bang/whynot commuting conversion: `⟨μα.c | μ̃!x.d⟩` reduces.
#[test]
fn commuting_bang_whynot() {
    let bang = promote::<'static, AtomP<X>>(|| Term::Axiom(Resource::new()));
    let m: Resource<'static, AtomN<X>> = Resource::new();

    let pos = mu::<'static, Bang<AtomP<X>>>(move |a| match a {
        Coterm::Elim(k) => (k.body)(bang.clone()),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_bang::<'static, AtomP<X>>(|b| {
        let v = derelict(b);
        cut(v, m.into())
    });
    let cmd = cut(pos, neg);
    let _outcome = run(cmd);
}

/// Mixed reduction: multiple commuting conversions composed with reduce cuts.
#[test]
fn mixed_reduction() {
    let x1: Resource<'static, AtomP<X>> = Resource::new();
    let y1: Resource<'static, AtomP<Y>> = Resource::new();
    let x2: Resource<'static, AtomP<X>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();

    let pos2 = mu::<'static, Plus<AtomP<X>, AtomP<Y>>>(move |a| match a {
        Coterm::Elim(WithElim { left, right: _ }) => left(Term::Axiom(x2)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg2 = mu_case::<'static, AtomP<X>, AtomP<Y>>(
        |a| cut(a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into()))),
        |_b| panic!("wrong branch"),
    );

    let pos = mu::<'static, Tensor<AtomP<X>, AtomP<Y>>>(move |a| match a {
        Coterm::Elim(k) => (k.body)(Term::Axiom(x1), Term::Axiom(y1)),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_par::<'static, AtomP<X>, AtomP<Y>>(|_a, _b| cut(pos2, neg2));
    let cmd = cut(pos, neg);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Par commuting conversion with composites.
#[test]
fn commuting_par_composites() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let inner = tensor(x, y);

    let pos = mu::<'static, Tensor<Tensor<AtomP<X>, AtomP<Y>>, One>>(move |a| match a {
        Coterm::Elim(k) => (k.body)(inner, unit()),
        Coterm::Axiom(_) => panic!("axiom in commuting"),
        Coterm::MuTilde(_) => panic!("mu_tilde in commuting"),
    });
    let neg = mu_par::<'static, Tensor<AtomP<X>, AtomP<Y>>, One>(|a, _b| {
        cut(
            a,
            mu_par::<'_, AtomP<X>, AtomP<Y>>(|a1, b1| {
                let cmd1 = cut(a1, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into())));
                let _ = cmd1;
                cut(b1, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into())))
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
        Coterm::Elim(k) => (k.body)(),
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
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let z: Resource<'static, AtomN<X>> = Resource::new();

    let pos = mu::<'static, AtomP<X>>(|a: Coterm<'_, AtomN<X>>| match a {
        Coterm::Axiom(v) => cut(Term::Axiom(x), Coterm::Axiom(v)),
        Coterm::Elim(cont) => (cont.body)(Term::Axiom(x)),
        Coterm::MuTilde(f) => f(Term::Axiom(x)),
    });
    let cmd = cut(pos, z.into());

    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Nested binders reduce correctly.
#[test]
fn nested_binders_reduce() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let free: Resource<'static, AtomN<Y>> = Resource::new();

    let cmd = cut(
        tensor(x, y),
        mu_par::<'static, AtomP<X>, AtomP<Y>>(|a, b| {
            cut(
                a,
                mu_tilde::<'_, AtomN<X>>(|_z| {
                    cut(b, mu_tilde::<'_, AtomN<Y>>(|_a| cut(_a, free.into())))
                }),
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
    check::<Plus<AtomP<X>, AtomP<Y>>>();
    fn check_neg<N: Negative>() {}
    check_neg::<With<AtomN<X>, AtomN<Y>>>();
}

/// Reduction cut with left injection: takes the left branch.
#[test]
fn plus_left_reduce() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let val = inl::<AtomP<X>, AtomP<Y>>(x);
    let coterm = mu_case::<'static, AtomP<X>, AtomP<Y>>(
        |a| cut(a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into()))),
        |_b| cut(_b, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into()))),
    );

    let cmd = cut(val, coterm);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Reduction cut with right injection: takes the right branch.
#[test]
fn plus_right_reduce() {
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let m: Resource<'static, AtomN<X>> = Resource::new();
    let n: Resource<'static, AtomN<Y>> = Resource::new();

    let val = inr::<AtomP<X>, AtomP<Y>>(y);
    let coterm = mu_case::<'static, AtomP<X>, AtomP<Y>>(
        |_a| cut(_a, mu_tilde::<'_, AtomN<X>>(|v| cut(v, m.into()))),
        |b| cut(b, mu_tilde::<'_, AtomN<Y>>(|v| cut(v, n.into()))),
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
    check::<Bang<AtomP<X>>>();
    fn check_neg<N: Negative>() {}
    check_neg::<Whynot<AtomN<X>>>();
}

#[test]
fn exponential_bang_value_clone() {
    let bang = promote::<'static, AtomP<X>>(|| Term::Axiom(Resource::new()));
    let _clone = bang.clone();
}

#[test]
fn exponential_promote() {
    let bang: BangIntro<'static, AtomP<X>> = promote(|| Term::Axiom(Resource::new()));
    let _ = bang;
}

#[test]
fn exponential_derelict() {
    let bang: BangIntro<'static, AtomP<X>> = promote(|| Term::Axiom(Resource::new()));
    let _v1: Term<'static, AtomP<X>> = derelict(bang.clone());
    let _v2: Term<'static, AtomP<X>> = derelict(bang);
}

/// Exponential cut — body receives BangIntro and can use it zero times (weakening).
#[test]
fn exponential_cut_weakening() {
    let bang = promote::<'static, AtomP<X>>(|| Term::Axiom(Resource::new()));
    let coterm = mu_bang::<'static, AtomP<X>>(|_b: BangIntro<'_, AtomP<X>>| Command::Normal);
    let cmd = cut(Term::<Bang<AtomP<X>>>::Intro(bang), coterm);
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

#[test]
fn exponential_cut_single_use() {
    let z: Resource<'static, AtomN<X>> = Resource::new();
    let bang = promote::<'static, AtomP<X>>(|| Term::Axiom(Resource::new()));
    let coterm = mu_bang::<'static, AtomP<X>>(|b: BangIntro<'_, AtomP<X>>| {
        let v = derelict(b);
        cut(v, z.into())
    });
    let cmd = cut(Term::<Bang<AtomP<X>>>::Intro(bang), coterm);
    let _outcome = run(cmd);
}

#[test]
fn exponential_cut_contraction() {
    let z1: Resource<'static, AtomN<X>> = Resource::new();
    let z2: Resource<'static, AtomN<X>> = Resource::new();
    let bang = promote::<'static, AtomP<X>>(|| Term::Axiom(Resource::new()));
    let coterm = mu_bang::<'static, AtomP<X>>(|b: BangIntro<'_, AtomP<X>>| {
        let v1 = derelict(b.clone());
        let v2 = derelict(b);
        let cmd1 = cut(v1, z1.into());
        let _ = cmd1;
        cut(v2, z2.into())
    });
    let cmd = cut(Term::<Bang<AtomP<X>>>::Intro(bang), coterm);
    let _outcome = run(cmd);
}

/// Composite classical types.
#[test]
fn exponential_composite() {
    let z: Resource<'static, AtomN<X>> = Resource::new();
    let w: Resource<'static, AtomN<Y>> = Resource::new();

    let bang = promote::<'static, Tensor<AtomP<X>, AtomP<Y>>>(|| {
        Term::Intro((Term::Axiom(Resource::new()), Term::Axiom(Resource::new())))
    });

    let coterm = mu_bang::<'static, Tensor<AtomP<X>, AtomP<Y>>>(
        |b: BangIntro<'_, Tensor<AtomP<X>, AtomP<Y>>>| {
            let pair = derelict(b);
            cut(
                pair,
                mu_par::<'_, AtomP<X>, AtomP<Y>>(|a, b_val| {
                    let cmd1 = cut(a, z.into());
                    let _ = cmd1;
                    cut(b_val, w.into())
                }),
            )
        },
    );

    let cmd = cut(
        Term::<Bang<Tensor<AtomP<X>, AtomP<Y>>>>::Intro(bang),
        coterm,
    );
    let _outcome = run(cmd);
}

// ==========================================================================
// Canonical form / commuting conversion tests
// ==========================================================================

/// Atomic commuting conversion: `⟨μα.c | μ̃x.d⟩` reduces.
#[test]
fn atomic_commuting_conversion() {
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let z: Resource<'static, AtomN<X>> = Resource::new();

    let pos = mu::<'static, AtomP<X>>(|a: Coterm<'_, AtomN<X>>| match a {
        Coterm::Axiom(v) => cut(Term::Axiom(x), Coterm::Axiom(v)),
        Coterm::Elim(cont) => (cont.body)(Term::Axiom(x)),
        Coterm::MuTilde(f) => f(Term::Axiom(x)),
    });

    let coterm = mu_atom::<'static, X>(move |t: Term<'_, AtomP<X>>| match t {
        Term::Axiom(v) => cut(Term::Axiom(v), z.into()),
        Term::Intro(i) => match i {},
        Term::Mu(_) => panic!("mu in atomic commuting"),
    });
    let cmd = cut(pos, coterm);

    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Mu vs Var for unit type: `⟨μα.c | x⊥⟩` where `x⊥ : ⊥`.
#[test]
fn cut_mu_unit_axiom() {
    let marker = std::rc::Rc::new(std::cell::Cell::new(false));
    let marker2 = marker.clone();
    let x: Resource<'static, Bot> = Resource::new();

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
    let x: Resource<'static, Par<AtomN<X>, AtomN<Y>>> = Resource::new();

    let pos = mu::<'static, Tensor<AtomP<X>, AtomP<Y>>>(
        |a: Coterm<'_, Par<AtomN<X>, AtomN<Y>>>| match a {
            Coterm::Axiom(_) => Command::Normal,
            Coterm::Elim(_) => panic!("elim branch"),
            Coterm::MuTilde(_) => panic!("mu_tilde branch"),
        },
    );

    let cmd = cut(pos, x.into());
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Mu vs Var for plus type: `⟨μα.c | x⊥⟩` where `x⊥ : A⊥ & B⊥`.
#[test]
fn cut_mu_plus_axiom() {
    let x: Resource<'static, With<AtomN<X>, AtomN<Y>>> = Resource::new();

    let pos =
        mu::<'static, Plus<AtomP<X>, AtomP<Y>>>(
            |a: Coterm<'_, With<AtomN<X>, AtomN<Y>>>| match a {
                Coterm::Axiom(_) => Command::Normal,
                Coterm::Elim(_) => panic!("elim branch"),
                Coterm::MuTilde(_) => panic!("mu_tilde branch"),
            },
        );

    let cmd = cut(pos, x.into());
    let outcome = run(cmd);
    assert!(matches!(outcome, Command::Normal));
}

/// Mu vs Var for bang type: `⟨μα.c | x⊥⟩` where `x⊥ : ?A⊥`.
#[test]
fn cut_mu_bang_axiom() {
    let x: Resource<'static, Whynot<AtomN<X>>> = Resource::new();

    let pos = mu::<'static, Bang<AtomP<X>>>(|a: Coterm<'_, Whynot<AtomN<X>>>| match a {
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
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomN<X>> = Resource::new();
    let cmd = cut(Term::Axiom(x), Coterm::Axiom(y));
    assert!(matches!(cmd, Command::Normal));
}

/// Axiom vs Elim (composite) produces Step wrapping Normal — blocked.
#[test]
fn canonical_axiom_vs_elim_composite() {
    let x: Resource<'static, Tensor<AtomP<X>, AtomP<Y>>> = Resource::new();
    let coterm = mu_par::<'static, AtomP<X>, AtomP<Y>>(|_a, _b| Command::Normal);
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
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let y: Resource<'static, AtomP<Y>> = Resource::new();
    let z: Resource<'static, Par<AtomN<X>, AtomN<Y>>> = Resource::new();
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
    let x: Resource<'static, AtomP<X>> = Resource::new();
    let coterm = mu_tilde::<'static, AtomN<X>>(|_t: Term<'_, AtomP<X>>| Command::Normal);
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
