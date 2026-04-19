//! # sequents
//!
//! A research prototype embedding multiplicative-additive linear System L
//! with exponentials into Rust's type system.
//!
//! Scope structure is carried by lifetimes; linearity is enforced by move
//! semantics. The library implements the λμ̃μ-calculus (see `Grokking`) with
//! symmetric three-variant [`Term`] and [`Coterm`] enums, a single [`cut`]
//! function dispatching over the 3×3 pairing, and per-connective reduction
//! via the [`Interaction`] and [`Resolution`] traits.
//!
//! ## Core concepts
//!
//! * [`Term`] — positive terms: axiom, introduction, or μ-binder.
//! * [`Coterm`] — negative coterms: axiom, elimination, or μ̃-binder.
//! * [`cut`] — the single operation pairing term with coterm.
//! * [`Command`] — a computation: either normal form or a reduction step.
//! * [`run`] — drives a command to normal form.
//!
//! ## Connectives
//!
//! Multiplicative: [`One`] / [`Bot`], [`Tensor`] / [`Par`].
//! Additive: [`Plus`] / [`With`].
//! Exponential: [`Bang`] / [`Whynot`].
//! Atomic: [`AtomP`] / [`AtomN`].
//!
//! ## References
//!
//! See `docs/CITATIONS.md` for full bibliographic details.
//!
//! * `MMM` — Mangel, Melliès, Munch-Maccagnoni. "Classical Notions of
//!   Computation and the Hasegawa–Thielecke Theorem." POPL 2026.
//! * `Spiwack` — Spiwack, Arnaud. "A Dissection of L." 2014.
//! * `Grokking` — Binder et al. "Grokking the Sequent Calculus." ICFP 2024.

/// Constructor functions for terms and coterms (μ, μ̃, tensor, par, etc.).
pub mod binder;
/// The command type and normalization loop.
pub mod machine;
/// The single cut function and its 3×3 dispatch table.
pub mod reduce;
/// Core type definitions: Term/Coterm enums, polarity traits, connectives.
pub mod types;

pub use binder::*;
pub use machine::{run, Command};
pub use reduce::*;
pub use types::*;

// Unit tests are colocated in each module's #[cfg(test)] block.
// Integration tests live in tests/*.rs.
