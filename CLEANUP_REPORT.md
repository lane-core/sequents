# Cleanup and Polishing Report

## Status

All milestones complete:
- M1 — CITATIONS.md and references.bib created, URLs verified
- M2 — types.rs doc comments (625 lines, every public item documented)
- M3 — binder.rs doc comments (266 lines, every constructor documented)
- M4 — reduce.rs and machine.rs doc comments
- M5 — lib.rs crate-level doc comment
- M6 — Fidelity sweep (see below)
- M7 — Ergonomic observations (see below)

Build: 46/46 tests pass, `cargo clippy --tests` clean, `cargo doc --no-deps` clean.

## Fidelity sweep (M6)

### Verified matches

**Three-variant Term/Coterm structure.** Matches the λμμ̃-calculus exactly:
- `Axiom` = axiom rule (variable/covariable)
- `Intro`/`Elim` = structural forms (data/codata constructors and destructors)
- `Mu`/`MuTilde` = μ/μ̃ binders (control operators)

This is the grammar of the linear classical L-calculus [MMM §7, figure/syntaxDialogue] and the polarized system L [Spiwack, module `Types`].

**Reduction rules.** Each `Reduction::reduce` implements the correct β-rule:
- `One`/`Bot`: `(R1)` — invoke body with no args [MMM §7]
- `Tensor`/`Par`: `(R⊗)` — destructure pair, pass components [MMM §7]
- `Plus`/`With`: branch on injection [Spiwack, `iota1`/`iota2`]
- `Bang`/`Whynot`: pass `BangIntro` to elim body [Spiwack, `exponential`]
- `AtomP`/`AtomN`: unreachable principal cut; `substitute` wraps resource as `Term::Axiom` [Spiwack, `mu`]

**Uniform Step wrapping.** Every non-terminal reduction produces `Step`. This matches the operational semantics where each reduction is one observable event.

**Naming.** `Resource`, `Axiom`, `Intro`, `Elim`, `Mu`, `MuTilde`, `Reduction`, `Substitution` all align with λμμ̃-calculus conventions [Grokking §3–4].

### Flagged observation (not a discrepancy)

**The library uses a two-sided presentation; MMM §7 uses one-sided.**

Our `Term`/`Coterm` split is a two-sided presentation: positive terms and negative coterms are separate types. MMM's linear classical L-calculus (§9, figure/syntaxDialogue) uses a one-sided sequent calculus where terms and coterms are both "expressions" distinguished by polarity within a unified grammar.

This is a notational choice, not a theoretical divergence. The two-sided presentation is standard in Curien-Herbelin λμμ̃ [Grokking §3] and in Spiwack's polarized system L. The duality is the same; only the type-system encoding differs. Our doc comments note this where relevant.

**No substantive discrepancies found.**

## Ergonomic observations (M7)

### Minor

1. **`Term::<Bang<AtomP<X>>>::Intro(bang)` requires explicit type annotation.**
   In tests, `cut(Term::Intro(bang), coterm)` fails inference for exponential cuts because `Term::Intro` doesn't carry enough type information. The turbofish `Term::<Bang<AtomP<X>>>::Intro(bang)` is required at ~4 call sites. A helper like `bang_term(bang)` that wraps with the correct type would eliminate this.

2. **`.into()` on `Resource` is slightly verbose.**
   `cut(x.into(), y.into())` for atomic axioms requires the `Into<Term>`/`Into<Coterm>` conversion. A helper `cut_axiom(x, y)` that takes two `Resource`s and wraps them would be cleaner for the common atomic-cut case. But this would be a convenience wrapper, not a structural change.

3. **`(k.body)(a, b)` callable field syntax is unusual.**
   Every elim call site needs parentheses around the field access. This is a Rust syntax quirk, not fixable without changing the struct design (e.g., making `body` a method). Worth noting but not changing — the current design is correct.

4. **`mu_tilde` name is three characters longer than `mu_neg` was.**
   Minor typing cost. The theoretical precision is worth it.

### Noted but not acted on

5. **No `Debug` impl for elim structs.**
   `ParElim`, `WithElim`, etc. don't implement `Debug`. This makes debugging test failures slightly harder. Adding `#[derive(Debug)]` where possible (not possible for `Box<dyn FnOnce>`) or custom `Debug` impls would help.

6. **`run` doesn't detect infinite loops.**
   As documented, `run` loops forever on non-terminating commands. This is correct for the core calculus but worth noting for any future extension that adds fuel-limited reduction.

### Summary

The API is clean and usable. The only recurring ergonomic friction is the `Term::Intro(bang)` type annotation for exponentials, which could be addressed with a small helper. Everything else is either a Rust syntax quirk (parenthesized field calls) or a deliberate design choice (explicit `.into()` conversions preserving type safety).
