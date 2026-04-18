# Memo: Consolidation and Extensions — Findings

## Consolidation

Option B is now the sole architectural commitment. The `option-b` branch was
merged into `master`; `option-a-historical` preserves the record. All
"Option A / Option B" framing has been removed from source files and
documentation.

### What changed in consolidation

- `NOTES.md` rewritten as a consolidated architectural document with key
  commitments, the HRTB constraint, the value-based binder insight, and
  documented ergonomic costs.
- `README.md` created as a first-time-reader overview.
- `src/example.rs` reorganized into "Constructing well-formed terms" and
  "Reducing terms" sections, with stale references removed.
- `.pi/todos/` added to `.gitignore` and removed from the index.

## Extension 1: Positive cuts (`cut_pos_atom`)

### What worked

The atomic case compiled and reduced correctly on the first attempt. The test
`extension_1_positive_atomic_cut` passes.

### The `MuPos` correction

The key realization was that `MuPos` (the positive μ-binder `μ⁺α.c`) binds a
**covariable** `α` of type `A::Dual` (negative), not a value of type `A`. In
classical System L, `μ⁺α.c` introduces a positive expression by binding a
covariable. The body receives `A::Dual::CoValue<'s>`.

This required changing:
- `MuPos` struct doc comment
- `Expr<'s, A>` impl for `MuPos`: `F: FnOnce(<A::Dual as Neg>::CoValue<'s>) -> Command<'s>`
- `mu_pos` constructor signature

For `AtomP<X>`: `A::Dual = AtomN<X>`, `A::Dual::CoValue<'s> = Var<'s, AtomN<X>>`.
So `mu_pos::<'static, AtomP<X>, _>(|a: Var<'_, AtomN<X>>| ...)` is correct.

### What is still open

Composite positive cuts. A positive cut `⟨μ⁺α.c | E⟩` where `E` is a composite
co-expression (e.g., `MuPar`) doesn't have a direct reduction rule in the
standard sequent calculus. In the Krivine machine, the co-expression would be
pushed onto the stack and consumed when `c` references `α`. Implementing this
would require an environment data structure, which the current design
explicitly avoids. This is a genuine architectural limit, not a workaround gap.

## Extension 2: Additives (`Plus`, `With`, `MuCase`, `cut_plus`)

### What worked

All three additive tests pass:
- `additive_duality`: `Plus<A, B>::Dual = With<A::Dual, B::Dual>` compiles
- `extension_2_plus_left`: left injection dispatches to left body
- `extension_2_plus_right`: right injection dispatches to right body

### The `MuCase` design

`MuCase<'s, A, B, F1, F2>` carries two bodies:
```rust
pub struct MuCase<'s, A: Pos, B: Pos, F1, F2> {
    pub body_left: F1,   // FnOnce(A::Value<'s>) -> Command<'s>
    pub body_right: F2,  // FnOnce(B::Value<'s>) -> Command<'s>
}
```

This is the first binder in the library with heterogeneous field types (two
different closures). It implements `CoExpr<'s, With<A::Dual, B::Dual>>`.

### Runtime dispatch

`cut_plus` performs a `match` on `PlusValue`:
```rust
pub fn cut_plus<...>(v: PlusValue<'s, A, B>, binder: MuCase<...>) -> Command<'s> {
    Command {
        step: Box::new(move || match v {
            PlusValue::Inl(a) => Outcome::Step((binder.body_left)(a)),
            PlusValue::Inr(b) => Outcome::Step((binder.body_right)(b)),
        }),
    }
}
```

This is the first reduction in the library with runtime dispatch. All
previous reductions (`cut_atom`, `cut_unit`, `cut_par`) were purely
structural — the shape of the value was known at the type level. For
additives, the shape (`Inl` vs `Inr`) is known only at runtime, so the
`match` is necessary.

The match is exhaustive and type-safe: Rust ensures both arms are present and
their types match. The runtime dispatch is localized to `cut_plus`; the rest
of the architecture remains structural.

### `Done` remains unused

The memo suggested additives might make `Done` reachable. They don't — at
least not in the current fragment. An additive cut reduces to a command that
still needs further reduction (or gets stuck on a generic `cut`). `Done` is
reserved for future extensions with genuine terminal configurations.

### What didn't cause friction

- **Dualities:** `Plus<A, B>::Dual = With<A::Dual, B::Dual>` and
  `With<A, B>::Dual = Plus<A::Dual, B::Dual>` compiled without issues.
- **`PlusValue` as `Expr`:** The `impl Expr<'s, Plus<A, B>> for PlusValue<'s, A, B>`
  was straightforward.
- **`With::CoValue = Infallible`:** Mirroring `Par`'s design, this caused no
  issues.

### What the additive case revealed

The distinction between **structural reduction** (multiplicatives) and
**dispatch reduction** (additives) is real and architecturally significant.
Structural reductions decompose values at the type level; dispatch reductions
inspect values at runtime. Both fit cleanly into the continuation-based
framework, but they exercise different parts of Rust's type system:

- Structural: associated type projections, tuple destructuring, closure args
- Dispatch: enums with payload variants, `match`, exhaustiveness checking

The library now handles both. This is a good sign for extensibility — the
continuation-based architecture doesn't privilege one style over the other.

## Commit history

```
9da92a9 docs: update NOTES.md and example.rs for extensions
6dc8cf3 feat: additive connectives (Plus, With, PlusValue, MuCase, cut_plus)
9c2872a feat: positive atomic cut (cut_pos_atom)
6e827b6 consolidate: merge Option B into master, clean up documentation
f76d1b0 feat: Option B continuation-based operational semantics
```

## What to do next

As the memo requested: stop and wait. Exponentials (`!A`, `?A`) are the next
logical direction, but they require `Clone` semantics for values, which
conflicts with the current linearity-by-move design. This needs architectural
discussion before implementation.
