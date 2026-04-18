# sequents

Linear classical L in Rust's type system.

A research library embedding the multiplicative fragment of linear classical
System L (Curien and Munch-Maccagnoni) into Rust. Scope structure is carried
by lifetimes; linearity is enforced by move semantics. Terms are both statically
well-formed by construction and operationally executable via continuation-based
reduction.

## What it does

```rust
use sequents::*;

struct X;

// Axiom: ⟨x | x⊥⟩
let x: Var<'static, AtomP<X>> = Var::new();
let y: Var<'static, AtomN<X>> = Var::new();
let cmd = cut(x, y);  // static well-formedness only

// Operational reduction: ⟨x | μz⁻.⟨z | y⟩⟩ steps to ⟨x | y⟩
let binder = mu_neg::<'static, AtomN<X>, _>(|z: Var<'_, AtomP<X>>| cut(z, y));
let cmd = cut_atom(x, binder);
let outcome = run(cmd);  // Stuck(StaticOnly) — normal form
```

## Architecture

- **Trait-based tagless-final:** `Expr<'s, A>` and `CoExpr<'s, N>` are marker
traits, not enums. Each introduction form is its own type.
- **Value-based binders:** Binder bodies receive `A::Value<'s>` (introduction
forms), not `Var<'s, A>` (tokens). This makes composite-type reduction work
recursively without a runtime environment.
- **Continuation-based commands:** `Command<'s>` wraps `Box<dyn FnOnce()>`.
Reduction is invocation. `run()` trampolines to a terminal state.
- **Specific lifetimes:** Binder closures take concrete lifetimes, enabling
capture of outer variables. Freshness is guaranteed by value identity
(move semantics on non-Copy `Var` tokens).

## Connectives

| Type | Polarity | Value |
|---|---|---|
| `AtomP<X>` | Positive | `Var<'s, AtomP<X>>` |
| `AtomN<X>` | Negative | `Var<'s, AtomN<X>>` |
| `One` | Positive | `()` |
| `Bot` | Negative | (no constructor) |
| `Tensor<A, B>` | Positive | `(A::Value, B::Value)` |
| `Par<A, B>` | Negative | (no constructor) |

## Reduction functions

- `cut(v, co)` — generic cut, static well-formedness only (stuck at runtime)
- `cut_atom(v, binder)` — atomic cut reduction
- `cut_unit(v, binder)` — unit cut reduction
- `cut_par(v, binder)` — tensor-par cut reduction (handles composites)
- `run(cmd)` — drive a command to `Done` or `Stuck`

## Running tests

```bash
cargo test
```

15 tests, all passing on stable Rust (2024 edition).

## Further reading

See `NOTES.md` for the full architectural account, including the HRTB-vs-
capture finding, the value-based binder design rationale, and documented
ergonomic costs.
