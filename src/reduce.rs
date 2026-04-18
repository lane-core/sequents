use crate::binder::{MuBang, MuCase, MuNeg, MuPar, MuPos, MuUnit};
use crate::machine::{Command, Outcome, StuckReason};
use crate::types::{
    Bang, BangValue, BotCoValue, One, ParCoValue, Plus, PlusValue, Pos, Tensor, WhynotCoValue,
    WithCoValue,
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
///
/// For atomic types, `AtomP<X>::Value<'s> = Var<'s, AtomP<X>>`, so the
/// binder body receives the variable directly.
pub fn cut_atom<'s, X: 'static, F>(
    v: Var<'s, crate::types::AtomP<X>>,
    binder: MuNeg<'s, crate::types::AtomN<X>, F>,
) -> Command<'s>
where
    F: FnOnce(Var<'s, crate::types::AtomP<X>>) -> Command<'s> + 's,
{
    let body = binder.body;
    Command {
        step: Box::new(move || Outcome::Step(body(v))),
    }
}

/// Unit cut: `⟨() | μ().c⟩`.
///
/// Reduction rule: invoke the binder body with no argument.
pub fn cut_unit<'s, F>(_v: (), binder: MuUnit<'s, F>) -> Command<'s>
where
    F: FnOnce() -> Command<'s> + 's,
{
    let body = binder.body;
    Command {
        step: Box::new(move || Outcome::Step(body())),
    }
}

/// Unit commuting conversion: `⟨μ⁺().c | μ().d⟩`.
///
/// Reduction rule: wrap the `MuUnit` body as a `BotCoValue::Cont` and pass
/// it to the `MuPos` body.  When the body reaches a use of the covariable
/// bound by `μ⁺`, it invokes the continuation (the `MuUnit` body).
pub fn cut_pos_unit<'s, FPos, FNeg>(
    pos_binder: MuPos<'s, One, FPos>,
    neg_binder: MuUnit<'s, FNeg>,
) -> Command<'s>
where
    FPos: FnOnce(BotCoValue<'s>) -> Command<'s> + 's,
    FNeg: FnOnce() -> Command<'s> + 's,
{
    let pos_body = pos_binder.body;
    let neg_body = neg_binder.body;
    Command {
        step: Box::new(move || Outcome::Step(pos_body(BotCoValue::Cont(Box::new(neg_body))))),
    }
}

/// Positive atomic cut: `⟨μ⁺α.c | x⊥⟩` where `x⊥` is a covariable.
///
/// Reduction rule: invoke the binder body with the covariable.
///
/// For atomic types, `AtomN<X>::CoValue<'s> = Var<'s, AtomN<X>>`, so the
/// binder body receives the covariable directly.
pub fn cut_pos_atom<'s, X: 'static, F>(
    binder: MuPos<'s, crate::types::AtomP<X>, F>,
    v: Var<'s, crate::types::AtomN<X>>,
) -> Command<'s>
where
    F: FnOnce(Var<'s, crate::types::AtomN<X>>) -> Command<'s> + 's,
{
    let body = binder.body;
    Command {
        step: Box::new(move || Outcome::Step(body(v))),
    }
}

/// Par commuting conversion: `⟨μ⁺α.c | μ(x ⅋ y).d⟩`.
///
/// Reduction rule: wrap the `MuPar` body as a `ParCoValue::Cont` and pass
/// it to the `MuPos` body.  When the body reaches a use of the covariable
/// bound by `μ⁺`, it invokes the continuation with the tensor components.
pub fn cut_pos_par<'s, A: Pos, B: Pos, FPos, FNeg>(
    pos_binder: MuPos<'s, Tensor<A, B>, FPos>,
    neg_binder: MuPar<'s, A, B, FNeg>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    B::Dual: crate::types::Neg,
    FPos: FnOnce(ParCoValue<'s, A::Dual, B::Dual>) -> Command<'s> + 's,
    FNeg: FnOnce(A::Value<'s>, B::Value<'s>) -> Command<'s> + 's,
{
    let pos_body = pos_binder.body;
    let neg_body = neg_binder.body;
    Command {
        step: Box::new(move || Outcome::Step(pos_body(ParCoValue::Cont(Box::new(neg_body))))),
    }
}

/// Plus/with commuting conversion: `⟨μ⁺α.c | μcase(x ⇒ d₁, y ⇒ d₂)⟩`.
///
/// Reduction rule: wrap both `MuCase` bodies as a `WithCoValue::Cont` and
/// pass it to the `MuPos` body.  The body invokes whichever continuation
/// matches the injection it constructs.
pub fn cut_pos_plus<'s, A: Pos, B: Pos, FPos, FNeg1, FNeg2>(
    pos_binder: MuPos<'s, Plus<A, B>, FPos>,
    neg_binder: MuCase<'s, A, B, FNeg1, FNeg2>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    B::Dual: crate::types::Neg,
    FPos: FnOnce(WithCoValue<'s, A::Dual, B::Dual>) -> Command<'s> + 's,
    FNeg1: FnOnce(A::Value<'s>) -> Command<'s> + 's,
    FNeg2: FnOnce(B::Value<'s>) -> Command<'s> + 's,
{
    let pos_body = pos_binder.body;
    let neg_left = neg_binder.body_left;
    let neg_right = neg_binder.body_right;
    Command {
        step: Box::new(move || {
            Outcome::Step(pos_body(WithCoValue::Cont {
                left: Box::new(neg_left),
                right: Box::new(neg_right),
            }))
        }),
    }
}

/// Bang/whynot commuting conversion: `⟨μ⁺α.c | μ!x.d⟩`.
///
/// Reduction rule: wrap the `MuBang` body as a `WhynotCoValue::Cont` and
/// pass it to the `MuPos` body.  When the body reaches a use of the
/// covariable bound by `μ⁺`, it invokes the continuation with a `BangValue`.
pub fn cut_pos_bang<'s, A: Pos, FPos, FNeg>(
    pos_binder: MuPos<'s, Bang<A>, FPos>,
    neg_binder: MuBang<'s, A, FNeg>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    FPos: FnOnce(WhynotCoValue<'s, A::Dual>) -> Command<'s> + 's,
    FNeg: FnOnce(BangValue<'s, A>) -> Command<'s> + 's,
{
    let pos_body = pos_binder.body;
    let neg_body = neg_binder.body;
    Command {
        step: Box::new(move || Outcome::Step(pos_body(WhynotCoValue::Cont(Box::new(neg_body))))),
    }
}

/// Tensor-par cut: `⟨V ⊗ W | μ(x ⅋ y).c⟩`.
///
/// Reduction rule: destructure the pair and invoke the binder body with
/// the components.  Because binder bodies now receive `A::Value<'s>` and
/// `B::Value<'s>` (not `Var`s), this works for both atomic and composite
/// types without conversion.
///
/// For composite components, the binder body receives nested pairs and
/// can use further `cut_par` calls to destructure them recursively.
pub fn cut_par<'s, A: Pos, B: Pos, F>(
    v: (A::Value<'s>, B::Value<'s>),
    binder: MuPar<'s, A, B, F>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    B::Dual: crate::types::Neg,
    F: FnOnce(A::Value<'s>, B::Value<'s>) -> Command<'s> + 's,
{
    let body = binder.body;
    let (a, b) = v;
    Command {
        step: Box::new(move || Outcome::Step(body(a, b))),
    }
}

/// Additive cut: `⟨V | μcase(x ⇒ c₁, y ⇒ c₂)⟩`.
///
/// Reduction rule: inspect the injection and invoke the corresponding
/// body with the injected value.  This is the first reduction in the
/// library that performs runtime dispatch (the value's shape determines
/// which branch runs).
pub fn cut_plus<'s, A: Pos, B: Pos, F1, F2>(
    v: PlusValue<'s, A, B>,
    binder: MuCase<'s, A, B, F1, F2>,
) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    B::Dual: crate::types::Neg,
    F1: FnOnce(A::Value<'s>) -> Command<'s> + 's,
    F2: FnOnce(B::Value<'s>) -> Command<'s> + 's,
{
    Command {
        step: Box::new(move || match v {
            PlusValue::Inl(a) => Outcome::Step((binder.body_left)(a)),
            PlusValue::Inr(b) => Outcome::Step((binder.body_right)(b)),
        }),
    }
}

/// Exponential cut: `⟨!V | μ!x.c⟩`.
///
/// Reduction rule: invoke the binder body with the `BangValue`.  The body
/// may clone the producer (duplication), drop it (weakening), or use it
/// exactly once — all are permitted for classical values.
pub fn cut_bang<'s, A: Pos, F>(v: BangValue<'s, A>, binder: MuBang<'s, A, F>) -> Command<'s>
where
    A::Dual: crate::types::Neg,
    F: FnOnce(BangValue<'s, A>) -> Command<'s> + 's,
{
    let body = binder.body;
    Command {
        step: Box::new(move || Outcome::Step(body(v))),
    }
}
