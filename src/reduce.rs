use crate::binder::{MuBang, MuCase, MuNeg, MuPar, MuPos, MuUnit};
use crate::machine::{Command, Outcome, StuckReason};
use crate::types::{
    Bang, BangIntro, Bot, Coterm, One, Par, Plus, PlusIntro, Pos, Tensor, Term, Whynot, With,
    WithBody,
};
use crate::var::Var;

/// Generic cut: `⟨t | V⟩`.
///
/// The type system ensures the two sides have dual types, but without
/// knowing the specific introduction forms, no operational reduction can
/// be performed.  The resulting command is stuck.
pub fn cut<'s, A: Pos, T, E>(_t: T, _e: E) -> Command<'s>
where
    T: crate::expr::Expr<'s, A>,
    E: crate::expr::CoExpr<'s, A::Dual>,
    A::Dual: crate::types::Neg,
{
    Command::stuck(StuckReason::StaticOnly)
}

/// Atomic cut: `⟨x | μy⁻.c⟩` where `x` is a variable.
///
/// Reduction rule: invoke the binder body with `x`.
pub fn cut_atom<'s, X: 'static, F>(
    term: Term<'s, crate::types::AtomP<X>>,
    binder: MuNeg<'s, crate::types::AtomN<X>, F>,
) -> Command<'s>
where
    F: FnOnce(Term<'s, crate::types::AtomP<X>>) -> Command<'s> + 's,
{
    match term {
        Term::Var(v) => {
            let body = binder.body;
            Command {
                step: Box::new(move || Outcome::Step(body(Term::Var(v)))),
            }
        }
        Term::Intro(i) => match i {},
    }
}

/// Unit cut: `⟨() | μ().c⟩`.
///
/// Reduction rule: invoke the binder body with no argument.
pub fn cut_unit<'s, F>(term: Term<'s, One>, binder: MuUnit<'s, F>) -> Command<'s>
where
    F: FnOnce() -> Command<'s> + 's,
{
    match term {
        Term::Intro(()) => {
            let body = binder.body;
            Command {
                step: Box::new(move || Outcome::Step(body())),
            }
        }
        Term::Var(_) => Command::stuck(StuckReason::StaticOnly),
    }
}

/// Unit commuting conversion: `⟨μ⁺().c | μ().d⟩`.
///
/// Reduction rule: wrap the `MuUnit` body as a `Coterm::Body` and pass
/// it to the `MuPos` body.  When the body reaches a use of the covariable
/// bound by `μ⁺`, it invokes the continuation (the `MuUnit` body).
pub fn cut_pos_unit<'s, FPos, FNeg>(
    pos_binder: MuPos<'s, One, FPos>,
    neg_binder: MuUnit<'s, FNeg>,
) -> Command<'s>
where
    FPos: FnOnce(Coterm<'s, Bot>) -> Command<'s> + 's,
    FNeg: FnOnce() -> Command<'s> + 's,
{
    let pos_body = pos_binder.body;
    let neg_body = neg_binder.body;
    Command {
        step: Box::new(move || {
            let cont: Box<dyn FnOnce() -> Command<'s> + 's> = Box::new(neg_body);
            Outcome::Step(pos_body(Coterm::Body(cont)))
        }),
    }
}

/// Positive atomic cut: `⟨μ⁺α.c | x⊥⟩` where `x⊥` is a covariable.
///
/// Reduction rule: invoke the binder body with the coterm.
pub fn cut_pos_atom<'s, X: 'static, F>(
    binder: MuPos<'s, crate::types::AtomP<X>, F>,
    coterm: Coterm<'s, crate::types::AtomN<X>>,
) -> Command<'s>
where
    F: FnOnce(Coterm<'s, crate::types::AtomN<X>>) -> Command<'s> + 's,
{
    let body = binder.body;
    Command {
        step: Box::new(move || Outcome::Step(body(coterm))),
    }
}

/// Positive binder vs covariable: `⟨μ⁺α.c | x⊥⟩` for any positive type.
///
/// Reduction rule: wrap the covariable as `Coterm::Var` and pass it to
/// the `MuPos` body.  This is the generic form of `cut_pos_atom` that
/// works for all positive types, not just atoms.
pub fn cut_pos_var<'s, A: Pos, F>(binder: MuPos<'s, A, F>, var: Var<'s, A::Dual>) -> Command<'s>
where
    F: FnOnce(Coterm<'s, A::Dual>) -> Command<'s> + 's,
    A::Dual: crate::types::Neg,
{
    let body = binder.body;
    Command {
        step: Box::new(move || Outcome::Step(body(Coterm::Var(var)))),
    }
}

/// Par commuting conversion: `⟨μ⁺α.c | μ(x ⅋ y).d⟩`.
///
/// Reduction rule: wrap the `MuPar` body as a `Coterm::Body` and pass
/// it to the `MuPos` body.
pub fn cut_pos_par<'s, A: Pos, B: Pos, FPos, FNeg>(
    pos_binder: MuPos<'s, Tensor<A, B>, FPos>,
    neg_binder: MuPar<'s, A, B, FNeg>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    B::Dual: crate::types::Neg,
    FPos: FnOnce(Coterm<'s, Par<A::Dual, B::Dual>>) -> Command<'s> + 's,
    FNeg: FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s> + 's,
{
    let pos_body = pos_binder.body;
    let neg_body = neg_binder.body;
    Command {
        step: Box::new(move || {
            let cont: Box<dyn FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s> + 's> =
                Box::new(neg_body);
            Outcome::Step(pos_body(Coterm::Body(cont)))
        }),
    }
}

/// Plus/with commuting conversion: `⟨μ⁺α.c | μcase(x ⇒ d₁, y ⇒ d₂)⟩`.
///
/// Reduction rule: wrap both `MuCase` bodies as a `WithBody` inside
/// `Coterm::Body` and pass it to the `MuPos` body.
pub fn cut_pos_plus<'s, A: Pos, B: Pos, FPos, FNeg1, FNeg2>(
    pos_binder: MuPos<'s, Plus<A, B>, FPos>,
    neg_binder: MuCase<'s, A, B, FNeg1, FNeg2>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    B::Dual: crate::types::Neg,
    FPos: FnOnce(Coterm<'s, With<A::Dual, B::Dual>>) -> Command<'s> + 's,
    FNeg1: FnOnce(Term<'s, A>) -> Command<'s> + 's,
    FNeg2: FnOnce(Term<'s, B>) -> Command<'s> + 's,
{
    let pos_body = pos_binder.body;
    let neg_left = neg_binder.body_left;
    let neg_right = neg_binder.body_right;
    Command {
        step: Box::new(move || {
            Outcome::Step(pos_body(Coterm::Body(WithBody {
                left: Box::new(neg_left),
                right: Box::new(neg_right),
            })))
        }),
    }
}

/// Bang/whynot commuting conversion: `⟨μ⁺α.c | μ!x.d⟩`.
///
/// Reduction rule: wrap the `MuBang` body as a `Coterm::Body` and
/// pass it to the `MuPos` body.
pub fn cut_pos_bang<'s, A: Pos, FPos, FNeg>(
    pos_binder: MuPos<'s, Bang<A>, FPos>,
    neg_binder: MuBang<'s, A, FNeg>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    FPos: FnOnce(Coterm<'s, Whynot<A::Dual>>) -> Command<'s> + 's,
    FNeg: FnOnce(BangIntro<'s, A>) -> Command<'s> + 's,
{
    let pos_body = pos_binder.body;
    let neg_body = neg_binder.body;
    Command {
        step: Box::new(move || {
            let cont: Box<dyn FnOnce(BangIntro<'s, A>) -> Command<'s> + 's> = Box::new(neg_body);
            Outcome::Step(pos_body(Coterm::Body(cont)))
        }),
    }
}

/// Tensor-par cut: `⟨V ⊗ W | μ(x ⅋ y).c⟩`.
///
/// Reduction rule: destructure the pair and invoke the binder body with
/// the components.
pub fn cut_par<'s, A: Pos, B: Pos, F>(
    term: Term<'s, Tensor<A, B>>,
    binder: MuPar<'s, A, B, F>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    B::Dual: crate::types::Neg,
    F: FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s> + 's,
{
    match term {
        Term::Intro((a, b)) => {
            let body = binder.body;
            Command {
                step: Box::new(move || Outcome::Step(body(a, b))),
            }
        }
        Term::Var(_) => Command::stuck(StuckReason::StaticOnly),
    }
}

/// Additive cut: `⟨V | μcase(x ⇒ c₁, y ⇒ c₂)⟩`.
///
/// Reduction rule: inspect the injection and invoke the corresponding
/// body with the injected term.
pub fn cut_plus<'s, A: Pos, B: Pos, F1, F2>(
    term: Term<'s, Plus<A, B>>,
    binder: MuCase<'s, A, B, F1, F2>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    B::Dual: crate::types::Neg,
    F1: FnOnce(Term<'s, A>) -> Command<'s> + 's,
    F2: FnOnce(Term<'s, B>) -> Command<'s> + 's,
{
    match term {
        Term::Intro(v) => match v {
            PlusIntro::Inl(a) => {
                let body = binder.body_left;
                Command {
                    step: Box::new(move || Outcome::Step(body(a))),
                }
            }
            PlusIntro::Inr(b) => {
                let body = binder.body_right;
                Command {
                    step: Box::new(move || Outcome::Step(body(b))),
                }
            }
        },
        Term::Var(_) => Command::stuck(StuckReason::StaticOnly),
    }
}

/// Exponential cut: `⟨!V | μ!x.c⟩`.
///
/// Reduction rule: invoke the binder body with the `BangIntro`.
pub fn cut_bang<'s, A: Pos, F>(term: Term<'s, Bang<A>>, binder: MuBang<'s, A, F>) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    F: FnOnce(BangIntro<'s, A>) -> Command<'s> + 's,
{
    match term {
        Term::Intro(v) => {
            let body = binder.body;
            Command {
                step: Box::new(move || Outcome::Step(body(v))),
            }
        }
        Term::Var(_) => Command::stuck(StuckReason::StaticOnly),
    }
}
