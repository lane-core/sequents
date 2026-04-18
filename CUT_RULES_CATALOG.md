# Catalog of Cut Rules for Linear Classical L

## Method

For each positive type `A`, enumerate every `Expr<'s, A>` (introduction form)
and every `CoExpr<'s, A::Dual>` (co-introduction form). The type system ensures
only matching duals can be cut; mismatches are compile errors. Each valid pair
is a well-typed cut configuration.

Cut configurations fall into four categories:

1. **Principal cuts** — value vs destructor. These reduce structurally.
2. **Axiom cuts** — variable vs covariable. These are normal forms.
3. **Uninhabited cuts** — `MuPos` vs `Var` for non-atomic types. The covariable
   side has `CoValue = Infallible`, so these can never be constructed.
4. **Commuting conversions** — binder vs binder. Both sides are μ-binders.
   These reduce by substitution, not by structural decomposition.

---

## `A = AtomP<X>` (atomic)

| Positive (`Expr`) | Negative (`CoExpr`) | Status | Rule |
|---|---|---|---|
| `Var<'s, AtomP<X>>` | `Var<'s, AtomN<X>>` | Normal form | `cut` — stuck |
| `Var<'s, AtomP<X>>` | `MuNeg<'s, AtomN<X>, F>` | ✅ `cut_atom` | Principal |
| `MuPos<'s, AtomP<X>, F>` | `Var<'s, AtomN<X>>` | ✅ `cut_pos_atom` | Principal |
| `MuPos<'s, AtomP<X>, F>` | `MuNeg<'s, AtomN<X>, F2>` | ❌ Missing | Commuting conversion |

---

## `A = One` (unit)

| Positive (`Expr`) | Negative (`CoExpr`) | Status | Rule |
|---|---|---|---|
| `()` | `Var<'s, Bot>` | Normal form | `cut` — stuck |
| `()` | `MuUnit<'s, F>` | ✅ `cut_unit` | Principal |
| `MuPos<'s, One, F>` | `Var<'s, Bot>` | Uninhabited | `Bot::CoValue = Infallible` |
| `MuPos<'s, One, F>` | `MuUnit<'s, F2>` | ❌ Missing | Commuting conversion |

---

## `A = Tensor<C, D>` (tensor)

| Positive (`Expr`) | Negative (`CoExpr`) | Status | Rule |
|---|---|---|---|
| `(C::Value, D::Value)` | `Var<'s, Par<C::Dual, D::Dual>>` | Normal form | `cut` — stuck |
| `(C::Value, D::Value)` | `MuPar<'s, C, D, F>` | ✅ `cut_par` | Principal |
| `MuPos<'s, Tensor<C,D>, F>` | `Var<'s, Par<...,>>` | Uninhabited | `Par::CoValue = Infallible` |
| `MuPos<'s, Tensor<C,D>, F>` | `MuPar<'s, C, D, F2>` | ❌ Missing | Commuting conversion |

---

## `A = Plus<C, D>` (sum)

| Positive (`Expr`) | Negative (`CoExpr`) | Status | Rule |
|---|---|---|---|
| `PlusValue<'s, C, D>` | `Var<'s, With<C::Dual, D::Dual>>` | Normal form | `cut` — stuck |
| `PlusValue<'s, C, D>` | `MuCase<'s, C, D, F1, F2>` | ✅ `cut_plus` | Principal |
| `MuPos<'s, Plus<C,D>, F>` | `Var<'s, With<...,>>` | Uninhabited | `With::CoValue = Infallible` |
| `MuPos<'s, Plus<C,D>, F>` | `MuCase<'s, C, D, F1, F2>` | ❌ Missing | Commuting conversion |

---

## `A = Bang<C>` (exponential)

| Positive (`Expr`) | Negative (`CoExpr`) | Status | Rule |
|---|---|---|---|
| `BangValue<'s, C>` | `Var<'s, Whynot<C::Dual>>` | Normal form | `cut` — stuck |
| `BangValue<'s, C>` | `MuBang<'s, C, F>` | ✅ `cut_bang` | Principal |
| `MuPos<'s, Bang<C>, F>` | `Var<'s, Whynot<C::Dual>>` | Uninhabited | `Whynot::CoValue = Infallible` |
| `MuPos<'s, Bang<C>, F>` | `MuBang<'s, C, F2>` | ❌ Missing | Commuting conversion |

---

## Summary

| Category | Count | Status |
|---|---|---|
| Principal cuts (value vs destructor) | 5 | **All implemented** |
| Axiom cuts (variable vs covariable) | 5 | Normal forms — correct |
| Uninhabited cuts (`MuPos` vs `Var`) | 4 | Dead code by construction |
| Commuting conversions (binder vs binder) | 5 | **All missing** |

**Total well-typed cut configurations: 19**
**Implemented: 10 (5 principal + 5 stuck as normal forms)**
**Missing: 5 (all commuting conversions)**

---

## The missing rules: commuting conversions

All 5 missing cuts are **commuting conversions** — a positive μ-binder on the
left facing a destructor binder on the right:

- `⟨μ⁺α.c | μy⁻.d⟩` — atomic commuting conversion
- `⟨μ⁺().c | μ().d⟩` — unit commuting conversion
- `⟨μ⁺α.c | μ(x ⅋ y).d⟩` — tensor/par commuting conversion
- `⟨μ⁺α.c | μcase(...).d⟩` — plus/with commuting conversion
- `⟨μ⁺α.c | μ?x.d⟩` — bang/whynot commuting conversion

In System L, these reduce by substituting one binder into the other's body:

```
⟨μ⁺α.c | μx⁻.d⟩ ▷ c[μx⁻.d/α]
```

The positive binder's body `c` contains free occurrences of `α`. Each
occurrence is replaced by the negative binder `μx⁻.d`. This is not a
structural decomposition — it's term rewriting.

---

## Why they are not straightforward mirrors

Principal cuts work because one side is a **value** (concrete data: `Var`,
`()`, a pair, `PlusValue`, `BangValue`). The value is passed directly to the
other side's binder body as a closure argument. Shape matches shape; the
reduction is a single closure invocation.

Commuting conversions have **binders on both sides**. There is no value to
pass. The positive binder's body expects `A::Dual::CoValue<'s>` (a covariable,
not a binder), and the negative binder's body expects `A::Value<'s>` (a value,
not a binder). Neither body can consume the other directly.

To implement these, the library would need one of:

### Option 1: Environment / stack (Krivine machine)

Push the unmatched binder onto a stack and enter the other binder's body.
When the body reaches a variable, look up the stack. This requires:

- A `Command<'s>` that carries a stack of pending contexts
- `Var<'s, A>` values that know how to trigger stack lookups
- `run()` loop modified to handle stack discipline

This is a genuine Krivine machine. It would handle all commuting conversions
and reach full normal forms. But it significantly changes the architecture:
commands are no longer simple continuations; they carry state.

### Option 2: Reification

Give `Var` an optional "continuation" field. A covariable `α` bound by
`μ⁺α.c` could carry a reference to the negative binder it was substituted
with. When `α` is used in `c`, it triggers the stored binder's reduction.

This complicates `Var` (no longer a ZST, no longer purely a token) and
threatens the linearity story: a `Var` that carries a continuation is
semantically different from a plain `Var`, and the distinction must be tracked
type-safely.

### Option 3: Accept the limitation

Document that the library implements **principal-cut reduction only**.
Terms requiring commuting conversions to reach normal form will get
`Stuck(StaticOnly)`. This is honest: the library provides a correctly typed
embedding of System L with partial operational semantics.

---

## What the substrate says

Rust's type system accepts all 5 missing cuts as well-typed. There is no type-
level friction. The obstacle is architectural, not substrate-imposed.

The deeper signal: the closure-based representation of binder bodies cannot be
substituted into each other. Closures can only be invoked; they cannot be
decomposed, traversed, or rewritten. This is a fundamental property of closures
as a representation. The substrate is not pushing back because the question is
not within the substrate's scope — it's a choice about how much of the
operational semantics to encode.

The library's current design encodes **structural reduction** (values matched
against binders) but not **substitutive reduction** (binders inserted into
other binders). Both are legitimate. The choice is a design commitment, not a
type-system workaround.
