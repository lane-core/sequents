//! # sequents
//!
//! A research prototype embedding multiplicative-additive linear System L
//! with exponentials into Rust's type system. Scope structure is carried
//! by lifetimes; linearity is enforced by move semantics.
//!
//! `Command<'s>` is a continuation (a closure that performs a reduction
//! step when invoked), and `Command::Normal` is the canonical terminal
//! state. Reduction is invocation-based (Krivine-machine style), not
//! AST inspection.

pub mod binder;
pub mod machine;
pub mod reduce;
pub mod types;

pub use binder::*;
pub use machine::{Command, run};
pub use reduce::*;
pub use types::*;

#[cfg(test)]
mod tests;
