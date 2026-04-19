use crate::machine::Command;
use crate::types::{Coterm, Negative, Positive, Term};

/// The single cut function: `⟨t | e⟩`.
///
/// In the sequent calculus, the cut rule pairs a term of type `A` with
/// a coterm of the dual type `A⊥` to form a command. This function
/// implements the operational semantics of cut: it dispatches over the
/// 3×3 pairing of [`Term`] and [`Coterm`] variants, producing either
/// a canonical form ([`Command::Normal`]) or a reduction step
/// ([`Command::Step`]).
///
/// # The 3×3 dispatch
///
/// | Term \ Coterm | `Axiom` | `Elim` | `MuTilde` |
/// |----------------|---------|--------|-----------|
/// | `Axiom` | `Normal` | `Step(e.resolve(v))` | `Step(f(Axiom(v)))` |
/// | `Intro` | `Normal` | `Step(A::interact(i, e))` | `Step(f(Intro(i)))` |
/// | `Mu` | `Step(f(c))` | `Step(f(c))` | `Step(f(c))` |
///
/// Two cases dispatch per-connective:
///
/// * **Axiom vs Elim** — [`Negative::resolve`]: the elim's body
///   consumes the resource. For atoms this resolves the variable
///   into the body; for composites this is blocked (`Normal`).
///
/// * **Intro vs Elim** — [`Positive::interact`]: the structural β-rule
///   for the connective fires. For tensor/par, the pair is destructured;
///   for plus/with, the injection is inspected; for unit/bot, the body
///   runs with no arguments.
///
/// The remaining cases are commuting conversions or μ/μ̃-reductions,
/// which re-enter `cut` after substituting the bound value.
///
/// Every non-terminal case wraps in [`Command::Step`] — each reduction
/// is one observable event. See `MMM §7]` and [Spiwack, module
/// [`Positive::interact`] for the underlying rules.
pub fn cut<'s, A: Positive>(term: Term<'s, A>, coterm: Coterm<'s, A::Dual>) -> Command<'s>
where
    A::Dual: Negative,
{
    match (term, coterm) {
        // Axiom vs Axiom: canonical form.
        (Term::Axiom(_), Coterm::Axiom(_)) => Command::Normal,

        // Axiom vs Elim: variable resolution (non-trivial only for atoms).
        (Term::Axiom(v), Coterm::Elim(e)) => {
            Command::Step(Box::new(move || <A::Dual as Negative>::resolve(e, v)))
        }

        // Axiom vs MuTilde: commuting conversion — wrap axiom as term.
        (Term::Axiom(v), Coterm::MuTilde(f)) => Command::Step(Box::new(move || f(Term::Axiom(v)))),

        // Intro vs Axiom: blocked, awaiting outer substitution.
        (Term::Intro(_), Coterm::Axiom(_)) => Command::Normal,

        // Intro vs Elim: interact cut (per-connective interaction).
        (Term::Intro(i), Coterm::Elim(e)) => Command::Step(Box::new(move || A::interact(i, e))),

        // Intro vs MuTilde: commuting conversion — wrap intro as term.
        (Term::Intro(i), Coterm::MuTilde(f)) => Command::Step(Box::new(move || f(Term::Intro(i)))),

        // Mu vs anything: the μ-binder consumes the coterm.
        (Term::Mu(f), c) => Command::Step(Box::new(move || f(c))),
    }
}
