# sequents

Multiplicative-additive linear System L with exponentials, embedded in Rust's type system. Scope structure is carried by lifetimes; linearity is enforced by move semantics.

This library implements the λμμ̃-calculus — classical sequent calculus as a programming language. Terms and coterms are symmetric three-variant enums. One `cut` function dispatches over the 3×3 pairing. Per-connective reduction is handled by the `Reduction` and `Substitution` traits.

## What this is

A research prototype, not a production library. It demonstrates that linear classical L can be expressed directly in a modern type system, with the operational semantics (Krivine-machine style) executable as Rust code.

The key idea: formulas are types, proofs are programs, and cut-elimination is evaluation. The library makes this literal — `cut(term, coterm)` produces a `Command` that reduces step by step.

## Quick example

```rust
use sequents::*;

struct X; // atomic type

let x: Resource<'static, AtomP<X>> = Resource::new();
let y: Resource<'static, AtomN<X>> = Resource::new();

// Axiom cut: ⟨x | y⟩ — already in normal form
let cmd = cut(x.into(), y.into());
assert!(matches!(run(cmd), Command::Normal));
```

A reduction:

```rust
let x: Resource<'static, AtomP<X>> = Resource::new();
let z: Resource<'static, AtomN<X>> = Resource::new();

// μ̃y.⟨y | z⟩ — a coterm that substitutes its argument into a cut with z
let coterm = mu_tilde::<'static, AtomN<X>>(|y| cut(y, z.into()));

// ⟨x | μ̃y.⟨y | z⟩⟩ reduces in one step to ⟨x | z⟩
let cmd = cut(Term::Axiom(x), coterm);
let outcome = run(cmd);
assert!(matches!(outcome, Command::Normal));
```

Tensor and par:

```rust
struct Y;

let x: Resource<'static, AtomP<X>> = Resource::new();
let y: Resource<'static, AtomP<Y>> = Resource::new();
let pair = tensor(x, y);

let coterm = mu_par::<'static, AtomP<X>, AtomP<Y>>(|a, b| {
    let m: Resource<'_, AtomN<X>> = Resource::new();
    let n: Resource<'_, AtomN<Y>> = Resource::new();
    let _ = cut(a, m.into());
    cut(b, n.into())
});

let _cmd = cut(pair, coterm);
```

## Core concepts

- **`Term<'s, A>`** — positive terms: axiom (variable), introduction (constructors), or μ-binder (control operator)
- **`Coterm<'s, N>`** — negative coterms: axiom (covariable), elimination (destructors), or μ̃-binder (control operator)
- **`cut(term, coterm)`** — pair a term with a coterm of dual type, producing a `Command`
- **`Command<'s>`** — either `Normal` (canonical form) or `Step` (one reduction remaining)
- **`run(cmd)`** — drive a command to normal form
- **`Resource<'x, A>`** — a linear-use token. Non-`Copy`, non-`Clone`. Move semantics enforce single use.

## Connectives

| Positive | Negative | Description |
|----------|----------|-------------|
| `AtomP<X>` | `AtomN<X>` | Atomic types |
| `One` | `Bot` | Multiplicative unit (⊥) |
| `Tensor<A, B>` | `Par<A, B>` | Multiplicative conjunction/disjunction (⊗/⅋) |
| `Plus<A, B>` | `With<A, B>` | Additive disjunction/conjunction (⊕/&) |
| `Bang<A>` | `Whynot<N>` | Exponential modality (!/?) |

## Documentation

Run `cargo doc --open` for full API documentation. Every public item has a doc comment explaining its theoretical basis.

Citations: `docs/CITATIONS.md`.

## References

- **MMM** — Mangel, Melliès, Munch-Maccagnoni. "Classical Notions of Computation and the Hasegawa–Thielecke Theorem." POPL 2026.
- **Spiwack** — Spiwack, Arnaud. "A Dissection of L." 2014.
- **Grokking** — Binder et al. "Grokking the Sequent Calculus." ICFP 2024.

See `docs/CITATIONS.md` for full bibliographic details.

## Tests

```
cargo test
```

46 tests covering structural typing, operational reduction, commuting conversions, and canonical form production.

## License

MIT OR Apache-2.0
