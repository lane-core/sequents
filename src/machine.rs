/// A command `c : (⊢ Γ)` at scope `'s`.
///
/// Two variants:
/// - `Normal`: a canonical form (axiom cut or blocked weak head normal form).
/// - `Step`: a continuation with work remaining — invoke the closure to
///   perform one reduction step.
pub enum Command<'s> {
    Normal,
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
/// Repeatedly invokes `Step` continuations until `Normal` is reached.
pub fn run<'s>(mut cmd: Command<'s>) -> Command<'s> {
    loop {
        match cmd {
            Command::Normal => return Command::Normal,
            Command::Step(next) => cmd = next(),
        }
    }
}
