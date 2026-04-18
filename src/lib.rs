//! # sequents
//!
//! A research prototype embedding the multiplicative fragment of linear System L
//! into Rust's type system.  Scope structure is carried by lifetimes; linearity
//! is enforced by move semantics.
//!
//! `Command<'s>` is a continuation
//! (a closure that performs a reduction step when invoked), and `Outcome<'s>`
//! describes the result of that step.  Reduction is invocation-based (Krivine-
//! machine style), not AST inspection.

pub mod binder;
pub mod expr;
pub mod machine;
pub mod reduce;
pub mod types;
pub mod var;

pub use binder::*;
pub use expr::{CoExpr, Expr};
pub use machine::{run, Command, Outcome, StuckReason};
pub use reduce::*;
pub use types::*;
pub use var::Var;

#[cfg(test)]
mod tests {
    use super::*;

    struct X;
    struct Y;

    #[test]
    fn atomic_axiom() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomN<X>> = Var::new();
        let _cmd: Command<'static> = cut(x, y);
    }

    #[test]
    fn unit_cut() {
        let u = unit();
        let co = mu_unit(|| Command::stuck(StuckReason::Unexpected("test".into())));
        let _cmd: Command<'static> = cut(u, co);
    }

    #[test]
    fn tensor_intro() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let _pair = tensor::<'static, AtomP<X>, AtomP<Y>>(x, y);
    }

    #[test]
    fn mu_neg_binder() {
        let a: Var<'static, AtomN<X>> = Var::new();
        let _co = mu_neg::<'static, AtomN<X>, _>(|y: Var<'_, AtomP<X>>| cut(y, a));
    }

    #[test]
    fn par_destructor() {
        let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
            let a: Var<'_, AtomN<X>> = Var::new();
            let b: Var<'_, AtomN<Y>> = Var::new();
            let cmd1 = cut(x, a);
            let _ = cmd1;
            cut(y, b)
        });
    }

    #[test]
    fn nested_binders() {
        let _w: Var<'static, AtomN<Y>> = Var::new();

        let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
            cut(
                x,
                mu_neg::<'_, AtomN<X>, _>(|_z: Var<'_, AtomP<X>>| cut(y, _w)),
            )
        });
    }

    #[test]
    fn triple_nested() {
        let _w: Var<'static, AtomN<Y>> = Var::new();

        let _co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
            cut(
                x,
                mu_neg::<'_, AtomN<X>, _>(|_z| {
                    cut(
                        y,
                        mu_neg::<'_, AtomN<Y>, _>(|_a| {
                            let b: Var<'_, AtomN<Y>> = Var::new();
                            cut(_a, b)
                        }),
                    )
                }),
            )
        });
    }

    #[test]
    fn tensor_par_cut() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let pair = tensor::<'static, AtomP<X>, AtomP<Y>>(x, y);

        let co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
            let m: Var<'_, AtomN<X>> = Var::new();
            let n: Var<'_, AtomN<Y>> = Var::new();
            let cmd1 = cut(a, m);
            let _ = cmd1;
            cut(b, n)
        });

        let _cmd: Command<'static> = cut(pair, co);
    }

    #[test]
    fn duality_involution() {
        fn check<P: Pos>() {}
        check::<AtomP<X>>();
        check::<One>();
        check::<Tensor<AtomP<X>, AtomP<Y>>>();
    }

    // ==========================================================================
    // Operational tests
    // ==========================================================================

    /// **Milestone 1**: Atomic cut reduces.
    /// `cut_atom(x, μy⁻.⟨y | z⟩)` should step to `⟨x | z⟩`.
    #[test]
    fn milestone_1_atomic_cut_reduces() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let z: Var<'static, AtomN<X>> = Var::new();

        let binder = mu_neg::<'static, AtomN<X>, _>(|y: Var<'_, AtomP<X>>| cut(y, z));
        let cmd = cut_atom(x, binder);

        let outcome = run(cmd);
        // After one step, we get `cut(x, z)` which is stuck (static only)
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// **Milestone 2**: Unit cut reduces.
    /// `cut_unit((), μ().c)` should step to `c`.
    #[test]
    fn milestone_2_unit_cut_reduces() {
        let marker = std::rc::Rc::new(std::cell::Cell::new(false));
        let marker2 = marker.clone();

        let binder = mu_unit(move || {
            marker2.set(true);
            Command::stuck(StuckReason::Unexpected("reached".into()))
        });
        let cmd = cut_unit((), binder);

        let outcome = run(cmd);
        assert!(marker.get());
        assert!(matches!(
            outcome,
            Outcome::Stuck(StuckReason::Unexpected(_))
        ));
    }

    /// **Milestone 3**: Tensor-par cut reduces for atoms.
    /// `cut_par((x, y), μ(a ⅋ b).c)` where `x`, `y` are atomic vars.
    #[test]
    fn milestone_3_tensor_par_atoms() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();

        let m: Var<'static, AtomN<X>> = Var::new();
        let n: Var<'static, AtomN<Y>> = Var::new();

        let binder = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
            let cmd1 = cut_atom(a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m)));
            let _ = cmd1;
            cut_atom(b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n)))
        });

        let pair = (x, y);
        let cmd = cut_par(pair, binder);

        // Just verify it runs without panicking; the exact outcome depends
        // on the body, which uses generic cut (stuck).
        let _outcome = run(cmd);
    }

    /// **Milestone 4**: Tensor-par cut reduces for composites.
    /// `cut_par(((a, b), (c, d)), μ(x ⅋ y).c)` where the pair is a nested
    /// tensor.  The binder body receives the composite components and can
    /// use nested `cut_par` calls to destructure them recursively.
    #[test]
    fn milestone_4_tensor_par_composites() {
        struct A;
        struct B;
        struct C;
        struct D;

        let a: Var<'static, AtomP<A>> = Var::new();
        let b: Var<'static, AtomP<B>> = Var::new();
        let c: Var<'static, AtomP<C>> = Var::new();
        let d: Var<'static, AtomP<D>> = Var::new();

        let m: Var<'static, AtomN<A>> = Var::new();
        let n: Var<'static, AtomN<B>> = Var::new();
        let p: Var<'static, AtomN<C>> = Var::new();
        let q: Var<'static, AtomN<D>> = Var::new();

        // Value: ((a, b), (c, d)) : Tensor<Tensor<AtomP<A>, AtomP<B>>, Tensor<AtomP<C>, AtomP<D>>>
        let pair = ((a, b), (c, d));

        // Binder: μ(x ⅋ y). ...  where x : Tensor<AtomP<A>, AtomP<B>> and y : Tensor<AtomP<C>, AtomP<D>>
        let binder =
            mu_par::<'static, Tensor<AtomP<A>, AtomP<B>>, Tensor<AtomP<C>, AtomP<D>>, _>(|x, y| {
                // x: (Var<AtomP<A>>, Var<AtomP<B>>)
                // y: (Var<AtomP<C>>, Var<AtomP<D>>)
                // Destructure x with an inner cut_par
                cut_par::<'_, AtomP<A>, AtomP<B>, _>(
                    x,
                    mu_par::<'_, AtomP<A>, AtomP<B>, _>(|a1, b1| {
                        // Destructure y with another inner cut_par
                        cut_par::<'_, AtomP<C>, AtomP<D>, _>(
                            y,
                            mu_par::<'_, AtomP<C>, AtomP<D>, _>(|c1, d1| {
                                let _ = cut_atom(a1, mu_neg::<'_, AtomN<A>, _>(|v| cut(v, m)));
                                let _ = cut_atom(b1, mu_neg::<'_, AtomN<B>, _>(|v| cut(v, n)));
                                let _ = cut_atom(c1, mu_neg::<'_, AtomN<C>, _>(|v| cut(v, p)));
                                cut_atom(d1, mu_neg::<'_, AtomN<D>, _>(|v| cut(v, q)))
                            }),
                        )
                    }),
                )
            });

        let cmd = cut_par(pair, binder);
        let _outcome = run(cmd);
    }

    /// **Milestone 5**: Multi-step reduction.
    /// `cut_atom(x, μy⁻.cut_atom(y, μz⁻.cut(z, w)))` reduces in two
    /// operational steps to `cut(x, w)` (stuck).
    #[test]
    fn milestone_5_multi_step() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let w: Var<'static, AtomN<X>> = Var::new();

        let cmd = cut_atom(
            x,
            mu_neg::<'static, AtomN<X>, _>(|y| {
                cut_atom(y, mu_neg::<'_, AtomN<X>, _>(|z| cut(z, w)))
            }),
        );

        let outcome = run(cmd);
        // Two reduction steps lead to cut(x, w), which is static-only stuck.
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// **Milestone 2 (Par commuting conversion)**: `⟨μ⁺α.c | μ(x ⅋ y).d⟩` reduces.
    /// The positive binder receives the par destructor as a `ParCoValue::Cont`
    /// and invokes it with the tensor components.
    #[test]
    fn milestone_2_commuting_par() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let m: Var<'static, AtomN<X>> = Var::new();
        let n: Var<'static, AtomN<Y>> = Var::new();

        let pos = mu_pos::<'static, Tensor<AtomP<X>, AtomP<Y>>, _>(
            |a: ParCoValue<'_, AtomN<X>, AtomN<Y>>| match a {
                ParCoValue::Cont(k) => k(x, y),
            },
        );
        let neg = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
            let cmd1 = cut_atom(a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m)));
            let _ = cmd1;
            cut_atom(b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n)))
        });
        let cmd = cut_pos_par(pos, neg);
        let _outcome = run(cmd);
    }

    /// **Milestone 3 (Plus/With commuting conversion)**: `⟨μ⁺α.c | μcase(...).d⟩` reduces.
    /// The positive binder receives the case destructor as a `WithCoValue::Cont`
    /// and invokes the branch matching its injection.
    #[test]
    fn milestone_3_commuting_plus_with() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let m: Var<'static, AtomN<X>> = Var::new();
        let n: Var<'static, AtomN<Y>> = Var::new();

        let pos = mu_pos::<'static, Plus<AtomP<X>, AtomP<Y>>, _>(move |a| match a {
            WithCoValue::Cont { left, right: _ } => left(x),
        });
        let neg = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
            |a| cut_atom(a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m))),
            |_b| cut_atom(_b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n))),
        );
        let cmd = cut_pos_plus(pos, neg);
        let outcome = run(cmd);
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// **Milestone 3b (Plus/With commuting conversion, right branch)**.
    #[test]
    fn milestone_3_commuting_plus_with_right() {
        let y: Var<'static, AtomP<Y>> = Var::new();
        let m: Var<'static, AtomN<X>> = Var::new();
        let n: Var<'static, AtomN<Y>> = Var::new();

        let pos = mu_pos::<'static, Plus<AtomP<X>, AtomP<Y>>, _>(move |a| match a {
            WithCoValue::Cont { left: _, right } => right(y),
        });
        let neg = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
            |_a| cut_atom(_a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m))),
            |b| cut_atom(b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n))),
        );
        let cmd = cut_pos_plus(pos, neg);
        let outcome = run(cmd);
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// **Milestone 4 (Bang/Whynot commuting conversion)**: `⟨μ⁺α.c | μ!x.d⟩` reduces.
    /// The positive binder receives the exponential destructor as a
    /// `WhynotCoValue::Cont` and invokes it with a `BangValue`.
    #[test]
    fn milestone_4_commuting_bang_whynot() {
        let bang = promote::<'static, AtomP<X>, _>(|| Var::new());
        let m: Var<'static, AtomN<X>> = Var::new();

        let pos = mu_pos::<'static, Bang<AtomP<X>>, _>(move |a| match a {
            WhynotCoValue::Cont(k) => k(bang),
        });
        let neg = mu_bang::<'static, AtomP<X>, _>(|b| {
            let v = derelict(b);
            cut(v, m)
        });
        let cmd = cut_pos_bang(pos, neg);
        let _outcome = run(cmd);
    }

    /// **Milestone 5 (Mixed reduction)**: Multiple commuting conversions
    /// composed with principal cuts, reducing end-to-end.
    ///
    /// Trace:
    /// 1. `cut_pos_par` (par commuting conversion) steps to the par body
    /// 2. Par body returns `cut_pos_plus` (with commuting conversion)
    /// 3. With commuting steps to the plus body invoking `left`
    /// 4. `left` body does `cut_atom` (principal cut)
    /// 5. Principal cut steps to generic `cut` (stuck)
    #[test]
    fn milestone_5_mixed_reduction() {
        let x1: Var<'static, AtomP<X>> = Var::new();
        let y1: Var<'static, AtomP<Y>> = Var::new();
        let x2: Var<'static, AtomP<X>> = Var::new();
        let m: Var<'static, AtomN<X>> = Var::new();

        // Inner: a plus/with commuting conversion
        let pos2 = mu_pos::<'static, Plus<AtomP<X>, AtomP<Y>>, _>(move |a| match a {
            WithCoValue::Cont { left, right: _ } => left(x2),
        });
        let neg2 = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
            |a| cut_atom(a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m))),
            |_b| Command::stuck(StuckReason::Unexpected("wrong branch".into())),
        );

        // Outer: a par commuting conversion whose body returns the inner one
        let pos = mu_pos::<'static, Tensor<AtomP<X>, AtomP<Y>>, _>(move |a| match a {
            ParCoValue::Cont(k) => k(x1, y1),
        });
        let neg = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|_a, _b| cut_pos_plus(pos2, neg2));
        let cmd = cut_pos_par(pos, neg);
        let outcome = run(cmd);
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// **Milestone 2b (Par commuting conversion with composites)**:
    /// Nested tensor components passed through `ParCoValue::Cont`.
    #[test]
    fn milestone_2_commuting_par_composites() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let m: Var<'static, AtomN<X>> = Var::new();
        let n: Var<'static, AtomN<Y>> = Var::new();

        let inner = (x, y);

        let pos = mu_pos::<'static, Tensor<Tensor<AtomP<X>, AtomP<Y>>, One>, _>(move |a| match a {
            ParCoValue::Cont(k) => k(inner, ()),
        });
        let neg = mu_par::<'static, Tensor<AtomP<X>, AtomP<Y>>, One, _>(|a, _b| {
            cut_par(
                a,
                mu_par::<'_, AtomP<X>, AtomP<Y>, _>(|a1, b1| {
                    let cmd1 = cut_atom(a1, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m)));
                    let _ = cmd1;
                    cut_atom(b1, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n)))
                }),
            )
        });
        let cmd = cut_pos_par(pos, neg);
        let _outcome = run(cmd);
    }

    /// **Milestone 1 (Bot commuting conversion)**: `⟨μ⁺().c | μ().d⟩` reduces.
    /// The positive binder receives the negative binder as a `BotCoValue::Cont`
    /// and invokes it, stepping to the negative binder's body.
    #[test]
    fn milestone_1_commuting_unit() {
        let marker = std::rc::Rc::new(std::cell::Cell::new(false));
        let marker2 = marker.clone();

        let pos = mu_pos::<'static, One, _>(|a: BotCoValue<'_>| {
            // Invoke the continuation: this is what a commuting conversion does.
            match a {
                BotCoValue::Cont(k) => k(),
            }
        });
        let neg = mu_unit(move || {
            marker2.set(true);
            Command::stuck(StuckReason::Unexpected("reached".into()))
        });
        let cmd = cut_pos_unit(pos, neg);

        let outcome = run(cmd);
        assert!(marker.get());
        assert!(matches!(
            outcome,
            Outcome::Stuck(StuckReason::Unexpected(_))
        ));
    }

    /// **Extension 1**: Positive atomic cut reduces.
    /// `cut_pos_atom(μ⁺α.⟨x | α⟩, z)` should step to `⟨x | z⟩`.
    #[test]
    fn extension_1_positive_atomic_cut() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let z: Var<'static, AtomN<X>> = Var::new();

        let binder = mu_pos::<'static, AtomP<X>, _>(|a: Var<'_, AtomN<X>>| cut(x, a));
        let cmd = cut_pos_atom(binder, z);

        let outcome = run(cmd);
        // After one step, we get `cut(x, z)` which is stuck (static only)
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// **Milestone 6**: Nested binders reduce correctly.
    /// Three levels of binders with outer capture
    /// Three levels of binders with outer capture.
    #[test]
    fn milestone_6_nested_binders() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let y: Var<'static, AtomP<Y>> = Var::new();
        let free: Var<'static, AtomN<Y>> = Var::new();

        let cmd = cut_par(
            (x, y),
            mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| {
                cut_atom(
                    a,
                    mu_neg::<'_, AtomN<X>, _>(|_z| {
                        cut_atom(
                            b,
                            mu_neg::<'_, AtomN<Y>, _>(|_a| {
                                cut(_a, free) // generic cut — stuck
                            }),
                        )
                    }),
                )
            }),
        );

        let outcome = run(cmd);
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    // ==========================================================================
    // Extension 2: Additive connectives
    // ==========================================================================

    /// Duality of additive connectives.
    #[test]
    fn additive_duality() {
        fn check<P: Pos>() {}
        check::<Plus<AtomP<X>, AtomP<Y>>>();
        fn check_neg<N: Neg>() {}
        check_neg::<With<AtomN<X>, AtomN<Y>>>();
    }

    /// `cut_plus` with left injection: should take the left branch.
    #[test]
    fn extension_2_plus_left() {
        let x: Var<'static, AtomP<X>> = Var::new();
        let m: Var<'static, AtomN<X>> = Var::new();
        let n: Var<'static, AtomN<Y>> = Var::new();

        let val = PlusValue::<'static, AtomP<X>, AtomP<Y>>::Inl(x);
        let binder = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
            |a| cut_atom(a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m))),
            |_b| cut_atom(_b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n))),
        );

        let cmd = cut_plus(val, binder);
        let outcome = run(cmd);
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    /// `cut_plus` with right injection: should take the right branch.
    #[test]
    fn extension_2_plus_right() {
        let y: Var<'static, AtomP<Y>> = Var::new();
        let m: Var<'static, AtomN<X>> = Var::new();
        let n: Var<'static, AtomN<Y>> = Var::new();

        let val = PlusValue::<'static, AtomP<X>, AtomP<Y>>::Inr(y);
        let binder = mu_case::<'static, AtomP<X>, AtomP<Y>, _, _>(
            |_a| cut_atom(_a, mu_neg::<'_, AtomN<X>, _>(|v| cut(v, m))),
            |b| cut_atom(b, mu_neg::<'_, AtomN<Y>, _>(|v| cut(v, n))),
        );

        let cmd = cut_plus(val, binder);
        let outcome = run(cmd);
        assert!(matches!(outcome, Outcome::Stuck(StuckReason::StaticOnly)));
    }

    // ==========================================================================
    // Extension 3: Exponential connectives
    // ==========================================================================

    /// Milestone 1: `Bang` and `Whynot` types compile with correct duality.
    #[test]
    fn exponential_duality() {
        fn check<P: Pos>() {}
        check::<Bang<AtomP<X>>>();
        fn check_neg<N: Neg>() {}
        check_neg::<Whynot<AtomN<X>>>();
    }

    /// Milestone 2: `BangValue` is `Clone`.
    #[test]
    fn exponential_bang_value_clone() {
        let bang = promote::<'static, AtomP<X>, _>(|| Var::new());
        let _clone = bang.clone();
    }

    /// Milestone 3: Promotion produces a `BangValue`.
    #[test]
    fn exponential_promote() {
        let bang: BangValue<'static, AtomP<X>> = promote(|| Var::new());
        let _ = bang;
    }

    /// Milestone 4: Dereliction extracts a fresh linear value.
    #[test]
    fn exponential_derelict() {
        let bang: BangValue<'static, AtomP<X>> = promote(|| Var::new());
        let _v1: Var<'static, AtomP<X>> = derelict(bang.clone());
        let _v2: Var<'static, AtomP<X>> = derelict(bang);
        // v1 and v2 are distinct variables (different values)
    }

    /// Milestone 5: Exponential cut — body receives BangValue and can use
    /// it zero times (weakening), one time, or multiple times (contraction).
    #[test]
    fn exponential_cut_weakening() {
        let bang = promote::<'static, AtomP<X>, _>(|| Var::new());
        let binder = mu_bang::<'static, AtomP<X>, _>(|_b: BangValue<'_, AtomP<X>>| {
            Command::stuck(StuckReason::Unexpected("weakened".into()))
        });
        let cmd = cut_bang(bang, binder);
        let outcome = run(cmd);
        assert!(matches!(
            outcome,
            Outcome::Stuck(StuckReason::Unexpected(_))
        ));
    }

    #[test]
    fn exponential_cut_single_use() {
        let z: Var<'static, AtomN<X>> = Var::new();
        let bang = promote::<'static, AtomP<X>, _>(|| Var::new());
        let binder = mu_bang::<'static, AtomP<X>, _>(|b: BangValue<'_, AtomP<X>>| {
            let v = derelict(b);
            cut(v, z)
        });
        let cmd = cut_bang(bang, binder);
        let _outcome = run(cmd);
    }

    #[test]
    fn exponential_cut_contraction() {
        let z1: Var<'static, AtomN<X>> = Var::new();
        let z2: Var<'static, AtomN<X>> = Var::new();
        let bang = promote::<'static, AtomP<X>, _>(|| Var::new());
        let binder = mu_bang::<'static, AtomP<X>, _>(|b: BangValue<'_, AtomP<X>>| {
            let v1 = derelict(b.clone());
            let v2 = derelict(b);
            let cmd1 = cut(v1, z1);
            let _ = cmd1;
            cut(v2, z2)
        });
        let cmd = cut_bang(bang, binder);
        let _outcome = run(cmd);
    }

    /// Milestone 6: Composite classical types.
    /// `Bang<Tensor<AtomP<X>, AtomP<Y>>>` produces pairs on demand.
    #[test]
    fn exponential_composite() {
        let z: Var<'static, AtomN<X>> = Var::new();
        let w: Var<'static, AtomN<Y>> = Var::new();

        let bang = promote::<'static, Tensor<AtomP<X>, AtomP<Y>>, _>(|| (Var::new(), Var::new()));

        let binder = mu_bang::<'static, Tensor<AtomP<X>, AtomP<Y>>, _>(
            |b: BangValue<'_, Tensor<AtomP<X>, AtomP<Y>>>| {
                let pair = derelict(b);
                cut_par(
                    pair,
                    mu_par::<'_, AtomP<X>, AtomP<Y>, _>(|a, b_val| {
                        let cmd1 = cut(a, z);
                        let _ = cmd1;
                        cut(b_val, w)
                    }),
                )
            },
        );

        let cmd = cut_bang(bang, binder);
        let _outcome = run(cmd);
    }
}
