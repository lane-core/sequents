/// A command `c : (⊢ Γ)` at scope `'s`.
///
/// In the λμμ̃-calculus, a command is the result of cutting a term
/// against a coterm. Commands are not values — they are computations
/// that reduce. This enum represents the two possible states of a
/// command:
///
/// * [`Command::Normal`] — a canonical form. Either an axiom cut
///   (variable vs variable) or a blocked weak head normal form
///   (intro vs axiom, or axiom vs elim for a composite type).
///   No further reduction is possible without external substitution.
///
/// * [`Command::Step`] — a thunk with work remaining. Invoking the
///   closure performs one reduction step and returns the resulting
///   command. This is the Krivine-machine style: reduction is
///   invocation-based, not AST inspection `Spiwack`.
///
/// Every non-terminal reduction produces exactly one `Step`. The
/// `Box` allocation is the physical trace of the reduction event.
pub enum Command<'s> {
    /// Canonical form — no further reduction possible.
    Normal,
    /// One reduction step remaining. Invoke the closure to perform it.
    Step(Box<dyn FnOnce() -> Command<'s> + 's>),
}

impl<'s> std::fmt::Debug for Command<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Normal => write!(f, "Normal"),
            Command::Step(_) => write!(f, "Step(..)"),
        }
    }
}

/// Drive a command to its normal form.
///
/// Repeatedly invokes [`Command::Step`] continuations until
/// [`Command::Normal`] is reached. This is the standard
/// normalization loop for Krivine-machine-style operational
/// semantics.
///
/// Note: this function does not detect infinite loops. A command
/// with an infinite reduction sequence will loop forever.
pub fn run<'s>(mut cmd: Command<'s>) -> Command<'s> {
    loop {
        match cmd {
            Command::Normal => return Command::Normal,
            Command::Step(next) => cmd = next(),
        }
    }
}
