# Notes: Linear System L in Rust via Lifetimes

## What this library is

A trait-based embedding of the multiplicative fragment of linear classical
System L, with static well-formedness enforced by Rust's type system and
operational semantics implemented as continuation-based commands (Krivine-
machine style).

Scope structure is carried by lifetimes. Linearity is enforced by non-Copy
move semantics. Polarity is a trait-level predicate with operational content.
NNF is enforced by the grammar (no type constructor for negation).

## Key architectural commitments

These are load-bearing and should not be revisited without good reason:

1. **Scope structure carried by lifetimes.** Each binder introduces variables
   at a specific scope `'s`. The lifetime parameter propagates through
   expressions, values, and commands.

2. **Linearity enforced by non-Copy move semantics.** `Var<'s, A>` is not
   `Copy` or `Clone`. Using a variable twice is a compile error. This is the
   correct level of abstraction for linear logic in Rust.

3. **Polarity as trait-level predicates.** `Pos` and `Neg` traits carry
   `Dual`, `Value<'s>`, and `CoValue<'s>` as associated types. NNF is enforced
   because negation is not a user-constructible type constructor — it is
   computed structurally by `Dual`.

4. **Trait-based `Expr`/`CoExpr` (tagless-final).** Each introduction form is
   its own type implementing the appropriate trait. Enums were tried and
   abandoned because Rust lacks GADT-style per-variant type refinement.

5. **Value-based binder bodies.** Binders take `A::Value<'s>` (introduction
   forms) rather than `Var<'s, A>` (tokens). For atoms this is the same
   (`Value<'s> = Var<'s, Self>`), but for composites it means the body receives
   the actual structured value (e.g., a nested pair) and can destructure it
   recursively. Substitution becomes argument-passing to the closure.

6. **Continuation-based commands.** `Command<'s>` is a struct wrapping
   `Box<dyn FnOnce() -> Outcome<'s> + 's>`. Reduction is invocation. Each
   specific cut (`cut_atom`, `cut_par`, `cut_unit`) builds a closure that
   performs its reduction step when invoked. A trampolined `run()` driver
   loops until `Done` or `Stuck`.

7. **Specific-lifetime closures, not HRTB.** Binder bodies take concrete
   lifetimes (`FnOnce(Var<'s, A>) -> Command<'s>`), not higher-rank
   quantification (`for<'x> FnOnce(Var<'x, A>) -> Command<'x>`). This enables
   capture of outer variables. Freshness is guaranteed by value identity
   (distinct `Var` values), not type-level universal quantification.

## Documented constraint: HRTB and capture are incompatible

A real finding about Rust's type system, not a workaround gap:

In Rust's affine region logic, universal quantification over lifetimes
(`for<'x>`) is incompatible with capture of non-`'static` lifetime-carrying
data. The universal quantifier ranges over all lifetimes, including `'static`.
When a closure captures `Var<'a, ()>` and uses it where `Var<'x, ()>` is
expected, Rust needs `'a: 'x` for all `'x`. Since `'x` includes `'static`, this
requires `'a: 'static`. But `'a` is a generic lifetime parameter — it does not
necessarily outlive `'static`.

This constrains binder closures to take concrete lifetimes. Value-level
freshness (distinct `Var` values + move semantics) fills the role that type-
level freshness would have played. Two binders at the same scope produce
variables with the same lifetime type but distinct values — this is
α-equivalence, and it is semantically correct.

## The value-based binder insight

What makes composite reduction work: substitution becomes argument-passing to
the closure, shape-matching happens at the type level, no environment or term
rewriting is needed.

In System L, `⟨V ⊗ W | μ(x ⅋ y).c⟩ ▷ c[V/x, W/y]`. In this library, the
reduction step passes `V` and `W` directly to the binder body closure as its
arguments. The closure receives them as `A::Value<'s>` and `B::Value<'s>` and
can use further `cut_par` calls to destructure recursively.

This is one of several legitimate choices for classical sequent calculus. It
is appropriate for a Krivine-machine implementation because it mirrors the
machine's stack discipline: values are popped from the stack and bound to the
binder's parameters.

## What the library does

### Static well-formedness
- Every well-typed term is a well-typed Rust expression
- NNF, polarity, linearity, and scope safety are enforced at compile time
- No ill-formed System L term can be constructed

### Operational semantics
- `cut_atom` reduces an atomic cut by invoking the binder body with the value
- `cut_unit` reduces a unit cut by invoking the body with no arguments
- `cut_par` reduces a tensor-par cut by destructuring the pair and invoking
  the body with the components; composite values are handled by nested
  `cut_par` calls
- `run()` drives a command to terminal state

### Connectives
- Atoms: `AtomP<X>`, `AtomN<X>`
- Units: `One`, `Bot`
- Multiplicatives: `Tensor<A, B>`, `Par<A, B>`

## What is not in the library yet

- **Positive cuts (`cut_pos`):** `MuPos` and `mu_pos` are defined but not
  exercised operationally. A positive cut `⟨μx⁺.c | V⟩` would reduce by
  invoking the binder body with `V`. The atomic case is straightforward.
  Composite positive cuts against `Par` co-values go through `MuPar` (already
  implemented).

- **Additives (`A ⊕ B` and `A & B`):** Sum types with left/right injections
  and case destructors. This would exercise runtime dispatch (the value selects
  which branch of the case to run).

- **Exponentials (`!A` and `?A`):** Controlled weakening and contraction.
  Requires `Clone` semantics for values, which conflicts with the current
  linearity-by-move design. This is a substantial extension.

## Ergonomic costs

### Turbofish on nested composite constructors

Composite reduction requires explicit type annotations on nested `cut_par`
and `mu_par` calls:

```rust
cut_par::<'_, AtomP<A>, AtomP<B>, _>(
    x,
    mu_par::<'_, AtomP<A>, AtomP<B>, _>(|a1, b1| { ... })
)
```

Rust's inference cannot work backward from `(Var, Var)` through associated
type projections to infer `A` and `B` for the inner `mu_par`. This is a real
ergonomic cost. Macro-based sugar is possible future work but not the current
priority.

### `impl Trait` in variable bindings

You cannot write `let e: impl Expr<'s, A> = ...`. You must rely on inference
or return `impl Trait` from a function. This is a Rust syntax limitation.

### `Par<A::Dual, B::Dual>: Neg` inference

Explicit `where A::Dual: Neg, B::Dual: Neg` bounds are still needed at use
sites, even though they're theoretically implied by `A: Pos, B: Pos`. Rust's
trait solver does not propagate this implication automatically.

## Test suite

The test suite has 15 tests:

- 9 static well-formedness tests (carried from earlier iterations)
- 6 operational tests:
  - `milestone_1_atomic_cut_reduces`
  - `milestone_2_unit_cut_reduces`
  - `milestone_3_tensor_par_atoms`
  - `milestone_4_tensor_par_composites`
  - `milestone_5_multi_step`
  - `milestone_6_nested_binders`

All tests pass on stable Rust (2024 edition).
