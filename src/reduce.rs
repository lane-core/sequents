use crate::machine::Command;
use crate::types::{Coterm, Neg, Reduction, Substitution, Term};

/// The single cut function: `⟨t | e⟩`.
///
/// Dispatches over the 3×3 pairing of `Term` and `Coterm` variants.
/// Per-connective logic for intro-vs-elim and axiom-vs-elim is delegated
/// to the `Reduction` trait.
pub fn cut<'s, A: Reduction>(term: Term<'s, A>, coterm: Coterm<'s, A::Dual>) -> Command<'s>
where
    A::Dual: Neg,
{
    match (term, coterm) {
        // Axiom vs Axiom: canonical form.
        (Term::Axiom(_), Coterm::Axiom(_)) => Command::Normal,

        // Axiom vs Elim: variable substitution (non-trivial only for atoms).
        (Term::Axiom(v), Coterm::Elim(e)) => Command::Step(Box::new(move || e.substitute(v))),

        // Axiom vs MuTilde: commuting conversion — wrap axiom as term.
        (Term::Axiom(v), Coterm::MuTilde(f)) => Command::Step(Box::new(move || f(Term::Axiom(v)))),

        // Intro vs Axiom: blocked, awaiting outer substitution.
        (Term::Intro(_), Coterm::Axiom(_)) => Command::Normal,

        // Intro vs Elim: reduce cut (per-connective reduction).
        (Term::Intro(i), Coterm::Elim(e)) => Command::Step(Box::new(move || A::reduce(i, e))),

        // Intro vs MuTilde: commuting conversion — wrap intro as term.
        (Term::Intro(i), Coterm::MuTilde(f)) => Command::Step(Box::new(move || f(Term::Intro(i)))),

        // Mu vs anything: the μ-binder consumes the coterm.
        (Term::Mu(f), c) => Command::Step(Box::new(move || f(c))),
    }
}
