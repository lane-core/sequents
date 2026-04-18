/// The result of invoking a command's reduction step.
pub enum Outcome<'s> {
    /// Reduction has reached a normal form (no further steps possible).
    Done,
    /// One reduction step was performed; the resulting command is ready
    /// for the next step.
    Step(Command<'s>),
    /// Reduction is stuck — the current configuration has no applicable
    /// reduction rule.  This should not occur in well-typed programs.
    Stuck(StuckReason),
}

impl<'s> std::fmt::Debug for Outcome<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Done => write!(f, "Done"),
            Outcome::Step(_) => write!(f, "Step(Command {{ .. }})"),
            Outcome::Stuck(r) => write!(f, "Stuck({r:?})"),
        }
    }
}

/// Why a command is stuck.
#[derive(Debug, PartialEq, Clone)]
pub enum StuckReason {
    /// The generic `cut` was used with a configuration that has no
    /// operational reduction rule (static-only well-formedness check).
    StaticOnly,
    /// Catch-all for unexpected configurations during development.
    Unexpected(String),
}

/// A command `c : (⊢ Γ)` at scope `'s`.
///
/// A command is a **continuation**: a closure that, when
/// invoked, performs one operational step and returns an `Outcome`.
pub struct Command<'s> {
    pub(crate) step: Box<dyn FnOnce() -> Outcome<'s> + 's>,
}

impl<'s> std::fmt::Debug for Command<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Command {{ .. }}")
    }
}

impl<'s> Command<'s> {
    /// Invoke the command's reduction step.
    pub fn run_once(self) -> Outcome<'s> {
        (self.step)()
    }

    /// Construct a stuck command.
    pub fn stuck(reason: StuckReason) -> Self {
        Command {
            step: Box::new(move || Outcome::Stuck(reason)),
        }
    }
}

/// Drive a command to a terminal state (`Done` or `Stuck`).
pub fn run<'s>(mut cmd: Command<'s>) -> Outcome<'s> {
    loop {
        match cmd.run_once() {
            Outcome::Done => return Outcome::Done,
            Outcome::Step(next) => cmd = next,
            o @ Outcome::Stuck(_) => return o,
        }
    }
}
