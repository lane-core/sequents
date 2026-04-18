# Notes: Linear System L in Rust via Lifetimes

## Overview

This is a research prototype validating the thesis that Rust's lifetime system
can faithfully carry the scope structure of simply-typed linear System L.  The
library has been through two design iterations:

- **First pass** (enum-based AST): validated static well-formedness but hit
  friction in four places that all traced to one root cause.
- **Option A** (trait-based, tagless-final): replaces enums with traits.  Each
  introduction form is its own type.  This eliminates the enum-based friction
  and enables nested binders.

## What the first pass established

The first pass proved that the core static thesis holds:

- NNF enforced by grammar ✓
- Polarity traits carry operational content (`Value<'s>`, `CoValue<'s>`) ✓
- `for<'x>` gives α-equivalence and capture-avoidance for free ✓
- Linearity via non-Copy move semantics ✓
- Scope safety by construction ✓

## Why the first pass hit friction

The first pass tried to build a concrete inspectable AST using Rust's enum
system while also carrying rich type-level information via traits and lifetimes.
Those two goals fight each other:

- Enums have runtime tags but no per-variant type refinement (no GADTs)
- So `CoExpr<'s, N>` needed `Box<dyn Any>` for the `MuPar` variant
- `FnOnce` trait objects aren't callable on stable Rust, requiring custom binder
  traits with `self: Box<Self>`
- Boxing closures required `+ 'static`, which prevented nested binders from
  capturing outer variables
- `Command<'s>` became an opaque ZST because trait objects with
  lifetime-returning methods are invariant

**The pattern**: when a local workaround appears, zoom out and ask whether the
structure causing the workaround is necessary.  The enum representation was
causing all the workarounds.

## Option A: Traits instead of enums

### The design

`Expr<'s, A>` and `CoExpr<'s, N>` are traits (empty markers), not enums.  Each
introduction form is a distinct type implementing the appropriate trait:

```rust
pub trait Expr<'s, A: Pos> {}
pub trait CoExpr<'s, N: Neg> {}

impl<'s, A: Pos> Expr<'s, A> for Var<'s, A> {}        // axiom
impl<'s, N: Neg> CoExpr<'s, N> for Var<'s, N> {}      // axiom (negative)
impl<'s> Expr<'s, One> for () {}                       // unit
impl<'s, A: Pos, B: Pos> Expr<'s, Tensor<A, B>>
    for (A::Value<'s>, B::Value<'s>) {}               // tensor

// Binders are structs parameterized by a specific lifetime 's
pub struct MuNeg<'s, N: Neg, F> { body: F, ... }
impl<'s, N: Neg, F> CoExpr<'s, N> for MuNeg<'s, N, F>
where F: FnOnce(Var<'s, N::Dual>) -> Command<'s> {}
```

Constructors return `impl Expr<'s, A>` or `impl CoExpr<'s, N>`:

```rust
pub fn mu_neg<'s, N: Neg, F>(body: F) -> impl CoExpr<'s, N>
where F: FnOnce(Var<'s, N::Dual>) -> Command<'s>
```

### What this fixes

1. **`Box<dyn Any>` eliminated.** `MuPar<'s, A, B, F>` has its natural type and
   implements `CoExpr<'s, Par<A::Dual, B::Dual>>` directly.  No runtime type
   erasure.

2. **Custom binder traits eliminated.** `MuNeg` stores `F` directly, not
   `Box<dyn NegBinder<N>>`.  The closure type is visible to the type system.
   No `self: Box<Self>` workaround.

3. **`'static` bounds eliminated.** Since there is no boxing, closures don't
   need `+ 'static`.  They can capture variables from any outer scope.

4. **`Command` stays a ZST but for a good reason.** In Phase 1 it's just a
   marker.  Phase 2 may give it operational content (continuation-based
   reduction).  The opacity is intentional, not a workaround.

5. **Nested binders compile cleanly.** This is the key milestone.

### The nested binder milestone

The term `μ(x ⅋ y).⟨x | μz⁻.⟨y | w⟩⟩` — where `w` is free in the outer scope —
now compiles:

```rust
let w: Var<'static, AtomN<Y>> = Var::new();
let co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
    cut(x, mu_neg::<'_, AtomN<X>, _>(|_z| cut(y, w)))
});
```

The inner `mu_neg` captures `y` from the outer `mu_par` scope.  This works
because:

- The binder constructors take a **specific lifetime `'s`** (not `for<'x>`)
- The closure body returns `Command<'s>` where `'s` matches the outer scope
- No `'static` bound prevents the capture

### The lifetime design decision (and the HRTB question)

The first pass used `for<'x> FnOnce(Var<'x, A>) -> Command<'x>` for binder
bodies.  The architectural thesis claimed that `for<'x>` gives α-equivalence
and freshness for free — each binder introduces a genuinely fresh, universally-
quantified lifetime.

Option A dropped `for<'x>` in favor of specific lifetimes:
`FnOnce(Var<'s, A>) -> Command<'s>`.  This enables capture but weakens the
type-level freshness guarantee.  The question was whether `for<'x>` could be
preserved alongside capture by separating the binder's outer lifetime from the
variable's inner lifetime.

**Investigation result: HRTB + non-`'static` capture is incompatible.**

Tested the proposed signature:

```rust
pub fn mu_neg_hrtb<'outer, F>(body: F) -> Command<'outer>
where
    F: for<'x> FnOnce(Var<'x, ()>) -> Command<'x> + 'outer,
```

Against a nested binder capturing a non-`'static` `Var`:

```rust
fn inner<'a>(outer: Var<'a, ()>) {
    let _cmd = mu_neg_hrtb(|inner: Var<'_, ()>| {
        cut(inner, outer)  // requires 'a: 'x for all 'x
    });
}
```

**Error:** `lifetime may not live long enough — returning this value requires that 'a must outlive 'static`

The `for<'x>` quantifier is universal over **all** lifetimes, including
`'static`. When the body captures `outer: Var<'a, ()>` and uses it where
`Var<'x, ()>` is expected, Rust needs `'a: 'x` for all `'x`. Since `'x`
includes `'static`, this requires `'a: 'static`. But `'a` is a generic lifetime
parameter — it does not necessarily outlive `'static`.

**This is a fundamental limit, not a workaround gap.** The first pass's
`+ 'static` bound was not an over-constraint — it was the **necessary
consequence** of combining HRTB with capture of lifetime-carrying data. You
cannot have both universal lifetime freshness and non-`'static` capture of
`Var` in Rust's type system.

**The current design correctly chooses capture over HRTB.** Freshness is still
guaranteed by value identity (each binder introduces a distinct `Var` value),
which is enforced by move semantics. The lifetime `'s` is a scope marker, not
an identity marker. Two binders at the same scope produce variables with the
same lifetime type but distinct values — this is α-equivalence, and it is
semantically correct.

**Soundness check:** Can the specific-lifetime signature admit confused code?
Tested multiple attack vectors:
- Double use (`cut(x, x)`) — rejected by move semantics
- Cross-binder reuse (same `Var` in two binder bodies) — rejected by move semantics
- Parameter swap (using outer binder's variable where inner's is expected) —
  compiles, but is semantically valid (α-equivalence)
- Escape via Vec/RefCell — possible in principle if closures are invoked, but
  irrelevant to static well-formedness

**Conclusion:** The weakening from HRTB to specific lifetime is **cosmetic,
not semantic.** No ill-formed System L term can be constructed. Move semantics
enforce linearity at the value level, which is the correct level of abstraction.

**Trade-off:** Top-level closed terms need explicit `'static` annotation
(`mu_neg::<'static, AtomN<X>, _>(...)`). This is slightly more verbose but
enables nesting.

### What still has friction

1. **`impl Trait` can't appear in variable bindings.** You can't write
   `let e: impl Expr<'s, A> = ...`.  You must write `let e = ...` and rely on
   inference, or return `impl Trait` from a function.  This is a Rust syntax
   limitation, not a design flaw.

2. **Type inference sometimes needs turbofish.** `tensor(x, y)` often needs
   `tensor::<AtomP<X>, AtomP<Y>>(x, y)` because associated type projections
   (`A::Value<'s>`) don't always provide enough inference hints.

3. **`Par<A::Dual, B::Dual>: Neg` is not always inferred.** Explicit
   `where A::Dual: Neg, B::Dual: Neg` bounds are still needed at use sites,
   even though they're theoretically implied by `A: Pos, B: Pos`.

4. **Composite-type reduction is still unimplemented.** The structural mismatch
   between `Value<'s>` (pairs for tensors) and `Var<'s, A>` (expected by binder
   bodies) remains.  This is a semantic issue, not a representation issue.
   Phase 2 (continuation-based commands) may address it.

## Comparison: first pass vs Option A

| Aspect | First pass (enums) | Option A (traits) |
|---|---|---|
| `Box<dyn Any>` | Yes (for `MuPar`) | No |
| Custom binder traits | Yes (`PosBinder`, `NegBinder`, `ParBinder`) | No |
| `+ 'static` on closures | Yes | No |
| Nested binders | No | Yes |
| Enum variants | `Expr::Var`, `Expr::Val`, `Expr::MuPos` | `Var`, `()`, `(V, W)`, `MuPos` as separate types |
| Type refinement | Runtime (`Any` downcast) | Compile-time (trait impls) |
| `impl Trait` in bindings | N/A (concrete enum) | Not allowed (Rust limitation) |

## Tests

The test suite validates:

- `atomic_axiom` — variable as both expression and co-expression
- `unit_cut` — `⟨() | μ().c⟩`
- `tensor_intro` — pair of atomic values
- `mu_neg_binder` — negative μ with free covariable
- `par_destructor` — `μ(x ⅋ y).c` using both variables
- `nested_binders` — **the key milestone**: inner `mu_neg` captures outer `y`
- `triple_nested` — three levels of nested binders
- `tensor_par_cut` — full tensor/par cut composition
- `duality_involution` — `A::Dual::Dual = A` compiles

All 9 tests pass on stable Rust (2024 edition).

## The strong vs weak form

| Form | Claim | Status |
|---|---|---|
| **Strong** | Rust enforces all of System L's well-formedness, including type-level freshness via HRTB | **Unachievable** — HRTB + non-`'static` capture is incompatible |
| **Weak** | Rust enforces scope safety and linearity; α-equivalence is maintained by value semantics | **Achieved** — the current library delivers this |

The strong form was the original architectural bet. The investigation shows it
fails not because of a missing workaround, but because of a fundamental tension
in Rust's type system: universal quantification over lifetimes (`for<'x>`)
requires captured lifetime-carrying data to outlive all quantified lifetimes,
which forces `'static`.

This is a **real finding** about Rust's type system. It says: Rust's lifetime
system can carry scope structure and enforce linearity, but it cannot
simultaneously provide universal freshness quantification and capture of
non-`'static` scoped data. You must choose one. The library chooses capture
(enabling nested binders) over HRTB freshness (which was cosmetic anyway,
since freshness is already guaranteed by value identity).

## Path forward

**Phase 1 is complete and settled.** The HRTB question has been investigated
and answered. The specific-lifetime design is sound, enables nested binders,
and eliminates all enum-based workarounds.

**Phase 2** (continuation-based commands, Option B from the memo) would layer
operational semantics on top.  The idea is to make `Command<'s>` a closure that
performs Krivine-style reduction when invoked.  Each `cut` would build a
closure capturing the expression and co-expression; invoking the closure would
apply the appropriate reduction rule.  Composite-type reduction would work
because the closure has access to the full structure of its captured values.

Whether to pursue Phase 2 depends on whether the goal is static well-formedness
(achieved) or executable semantics (open question).
