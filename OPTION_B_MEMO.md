# Option B Memo: Continuation-Based Commands

## Summary

All 6 milestones succeeded. Option B delivers operational semantics for linear
System L via continuation-based closures. The critical design decision was
changing binder bodies to receive `A::Value<'s>` (introduction forms) instead
of `Var<'s, A>` (tokens). This makes composite-type reduction work recursively
without a runtime environment.

## Milestone Results

| Milestone | Status | Notes |
|---|---|---|
| 1. Atomic cut reduces | ✅ Pass | `cut_atom` invokes binder body with the variable value directly. |
| 2. Unit cut reduces | ✅ Pass | `cut_unit` invokes binder body with no arguments. |
| 3. Tensor-par for atoms | ✅ Pass | `cut_par` destructures pair and passes components. |
| 4. Tensor-par for composites | ✅ Pass | Nested `cut_par` calls destructure recursively. |
| 5. Multi-step reduction | ✅ Pass | `run()` trampolines through `Outcome::Step` correctly. |
| 6. Nested binders | ✅ Pass | Three-level nesting with outer capture reduces in 3 steps. |

All 15 library tests pass; `cargo clippy` is clean.

## What Rust did and didn't do

### `Box<dyn FnOnce()>` on stable Rust

**It works.** `(self.step)()` on a `Box<dyn FnOnce() -> Outcome<'s> + 's>` compiles
and runs without custom traits or `self: Box<Self>` workarounds. This was the
first thing tested, and it removed the single biggest workaround from Option A's
first pass.

### Closure captures at concrete lifetimes

No friction. Every closure captures values and binder bodies at lifetime `'s`.
The `+ 's` bound on `Box<dyn FnOnce() + 's>` is satisfied because all captures
are either at `'s` or longer. No mysterious lifetime errors appeared at any
milestone.

### Recursive command types

No issue. A closure returns `Outcome::Step(Command<'s>)`, and the next command
has its own `Box<dyn FnOnce()>`. Rust's type inference handles this without
explicit annotations.

### Type inference and turbofish

**This was the main friction point.** In Milestone 4, nested `cut_par` and
`mu_par` calls inside composite binder bodies failed with "type annotations
needed." The compiler couldn't work backward from `(Var, Var)` through
associated type projections to infer `A` and `B` for the inner `mu_par`.

Fix: explicit turbofish on all inner constructors:

```rust
cut_par::<'_, AtomP<A>, AtomP<B>, _>(
    x,
    mu_par::<'_, AtomP<A>, AtomP<B>, _>(|a1, b1| { ... })
)
```

This is verbose but localized. It only affects deeply nested composite
reduction; atomic and unit cases infer cleanly.

### `impl Trait` vs concrete return types

Option A's constructors (`mu_neg`, `mu_par`, etc.) returned `impl CoExpr<'s, N>`.
This hid implementation details but prevented specific cut functions from
accessing binder bodies — `impl Trait` is opaque.

Option B changed constructors to return the concrete struct types (`MuNeg`,
`MuPar`, etc.). This is necessary because `cut_atom`, `cut_par`, etc. need to
move `binder.body` out of the struct. The structs still implement `Expr`/`CoExpr`,
so static well-formedness is preserved.

Trade-off: users see concrete types in error messages, which are slightly noisier
than opaque `impl Trait` errors.

## How composite reduction was solved (Milestone 4)

### The problem

In Option A, `Tensor<A, B>::Value<'s> = (A::Value<'s>, B::Value<'s>)`, but
binder bodies expected `Var<'s, A>` and `Var<'s, B>`. For atoms, `Value<'s> =
Var<'s, Self>`, so the types coincided. For composites, they diverged: a
nested pair `(Var, Var)` is not a `Var<Tensor<...,>>`.

### The solution

Change binder bodies to receive `A::Value<'s>` and `B::Value<'s>` instead of
`Var<'s, A>` and `Var<'s, B>`.

Before:
```rust
pub struct MuPar<'s, A: Pos, B: Pos, F> {
    body: F, // F: FnOnce(Var<'s, A>, Var<'s, B>) -> Command<'s>
}
```

After:
```rust
pub struct MuPar<'s, A: Pos, B: Pos, F> {
    body: F, // F: FnOnce(A::Value<'s>, B::Value<'s>) -> Command<'s>
}
```

For atomic types, `A::Value<'s> = Var<'s, A>`, so existing code is unchanged.
For composites, the body receives the actual composite value (e.g., a nested
pair) and can use further `cut_par` calls to destructure it:

```rust
cut_par(pair, mu_par(|x, y| {
    // x: (Var<AtomP<A>>, Var<AtomP<B>>)
    // y: (Var<AtomP<C>>, Var<AtomP<D>>)
    cut_par(x, mu_par(|a1, b1| {
        cut_par(y, mu_par(|c1, d1| {
            // a1, b1, c1, d1 are atomic vars now
            ...
        }))
    }))
}))
```

This is **substitution-by-destructuring**: the reduction step passes values
directly to the binder body, which recursively decomposes them. No environment,
no variable lookup, no `Box<dyn Any>`.

### Why this is correct

In System L, `⟨V ⊗ W | μ(x ⅋ y).c⟩ ▷ c[V/x, W/y]`. The substitution replaces
variables with values. By making the binder body closure take values directly,
we're effectively performing the substitution at the moment the closure is
invoked — the caller passes the values, the closure receives them as its
arguments. This is exactly the Krivine-machine semantics: the stack (the values)
is popped and bound to the binder's parameters.

### N::Dual::Value ambiguity

One Rust-specific issue: changing `MuNeg` to `FnOnce(N::Dual::Value<'s>) ->
Command<'s>` produced "ambiguous associated type" errors because `N::Dual` is
an associated type and chained projections (`N::Dual::Value`) aren't allowed in
`where` clauses without an explicit bound.

Fix:
```rust
impl<'s, N: Neg, F> CoExpr<'s, N> for MuNeg<'s, N, F>
where
    N::Dual: Pos,
    F: FnOnce(<N::Dual as Pos>::Value<'s>) -> Command<'s>,
```

The explicit `N::Dual: Pos` bound is implied by `N: Neg` but Rust needs it in
scope for the projection.

## The driver (`run`)

```rust
pub fn run<'s>(mut cmd: Command<'s>) -> Outcome<'s> {
    loop {
        match cmd.run_once() {
            Outcome::Done => return Outcome::Done,
            Outcome::Step(next) => cmd = next,
            Outcome::Stuck(r) => return Outcome::Stuck(r),
        }
    }
}
```

Composes cleanly. No lifetime issues. Each iteration consumes the current
command and replaces it with the next one. The loop terminates when the command
returns `Done` or `Stuck`.

`Outcome::Done` is currently unused — all well-typed multiplicative terms reduce
to `Stuck(StaticOnly)` (normal form) because the generic `cut` has no reduction
rule. `Done` is reserved for future connectives (e.g., additives or exponentials)
that might have terminal configurations.

## Error handling

`StuckReason` has two variants:

- `StaticOnly`: generic `cut` was used instead of a specific reduction function.
  This is the "normal form" for open terms.
- `Unexpected(String)`: catch-all for development.

No panics in the reduction logic. All error paths return `Outcome::Stuck`.

## Comparison with Option A

| Aspect | Option A | Option B |
|---|---|---|
| `Command<'s>` | ZST marker | Closure with operational step |
| `cut` | No-op | Specific reduction functions (`cut_atom`, `cut_par`, etc.) |
| Binder body args | `Var<'s, A>` | `A::Value<'s>` (introduction form) |
| Composite reduction | Unimplemented | Recursive `cut_par` nesting |
| Can run terms | No | Yes, via `run()` |
| Environment | None | None (substitution by destructuring) |
| `Box<dyn FnOnce>` | Not used | Core mechanism |
| Tests | 9 static | 15 (9 static + 6 operational) |

## Workarounds used

1. **Constructors return concrete types** instead of `impl Trait`, so specific
cut functions can access `binder.body`. This is a necessary API change, not a
language workaround.

2. **Turbofish on nested composite constructors** for type inference. This is a
Rust limitation with associated type projections in deep nesting.

3. **`N::Dual: Pos` explicit bound** in `MuNeg` impls due to ambiguous
associated type projections.

4. **Manual `Debug` impls** for `Command` and `Outcome` because `Box<dyn FnOnce>`
doesn't implement `Debug` or `PartialEq`.

## Architectural decision: value-based binders

The memo originally suggested keeping binder shapes unchanged and using an
environment for composite substitution. After experimenting with both approaches,
the value-based binder design (bodies receive `A::Value<'s>`) proved superior
because:

- No environment needed (simpler, no type erasure)
- Recursive destructuring is natural (nested `cut_par`)
- Atomic cases are transparent (`A::Value<'s> = Var<'s, A>` for atoms)
- Static types remain correct (all binder bodies still produce `Command<'s>`)

This is a departure from the memo's sketch but justified by the evidence: the
environment approach would have required `Box<dyn Any>` or similar type erasure,
reintroducing the exact workarounds Option A eliminated.

## Open questions

1. **Positive binder reduction**: `MuPos` and `cut_pos` are defined but not
exercised. Positive cuts (`⟨μx⁺.c | V⟩`) require co-value introduction forms,
which the multiplicative fragment doesn't have user-level constructors for
(except atomic covariables).

2. **`Done` vs `Stuck(StaticOnly)`**: In a complete implementation, `Done` should
represent genuine terminal states (e.g., additive choices). For the
multiplicative fragment, `Stuck(StaticOnly)` is the de facto normal form.

3. **Performance**: Each reduction step allocates a new `Box<dyn FnOnce>`.
This is acceptable for a research prototype but would need attention for
production use (e.g., a custom enum-based command representation with an
explicit stack).

4. **Exponentials and additives**: The current design should extend to `!A` and
`?A` (exponentials) and `A & B` / `A ⊕ B` (additives), but this is untested.

## Conclusion

Option B succeeds. The library can now construct, reduce, and drive linear
classical L terms. The trait-based static architecture from Option A is
preserved; the operational layer adds continuations without compromising the
type safety. The value-based binder design is the key insight that makes
composite reduction possible without an environment.
