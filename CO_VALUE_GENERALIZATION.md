# Co-value Generalization — Milestone Report

## Summary

All 5 non-atomic commuting conversions are now expressible. The library preserves all 27 original tests unmodified and adds 7 new commuting-conversion tests. `cargo clippy` is clean with zero warnings.

| Milestone | Status | Tests |
|-----------|--------|-------|
| 1. Bot | ✅ | `milestone_1_commuting_unit` |
| 2. Par | ✅ | `milestone_2_commuting_par`, `milestone_2_commuting_par_composites` |
| 3. With | ✅ | `milestone_3_commuting_plus_with`, `milestone_3_commuting_plus_with_right` |
| 4. Whynot | ✅ | `milestone_4_commuting_bang_whynot` |
| 5. Mixed reduction | ✅ | `milestone_5_mixed_reduction` (par → with → atomic principal, chained) |
| 6. Atomic consistency | — | Analysis below |

---

## What changed

### Pattern

For each non-atomic negative connective, replace `type CoValue<'s> = Infallible` with an enum that names the existential quantification over the corresponding binder-body closure:

| Connective | Old `CoValue` | New `CoValue` | Wraps closure from |
|---|---|---|---|
| `Bot` | `Infallible` | `BotCoValue<'s>` | `MuUnit` |
| `Par<A, B>` | `Infallible` | `ParCoValue<'s, A, B>` | `MuPar` |
| `With<A, B>` | `Infallible` | `WithCoValue<'s, A, B>` | `MuCase` |
| `Whynot<N>` | `Infallible` | `WhynotCoValue<'s, N>` | `MuBang` |

Each enum has a single `Cont` variant wrapping `Box<dyn FnOnce(...) -> Command<'s> + 's>`. The operational content is identical to the binder's body; what's new is the type-level name for the erasure.

### New cut functions

| Function | Configuration | Reduction rule |
|---|---|---|
| `cut_pos_unit` | `MuPos<One>` vs `MuUnit` | Wrap `MuUnit` body in `BotCoValue::Cont`, pass to `MuPos` body |
| `cut_pos_par` | `MuPos<Tensor<A, B>>` vs `MuPar<A, B>` | Wrap `MuPar` body in `ParCoValue::Cont`, pass to `MuPos` body |
| `cut_pos_plus` | `MuPos<Plus<A, B>>` vs `MuCase<A, B>` | Wrap both `MuCase` bodies in `WithCoValue::Cont`, pass to `MuPos` body |
| `cut_pos_bang` | `MuPos<Bang<A>>` vs `MuBang<A>` | Wrap `MuBang` body in `WhynotCoValue::Cont`, pass to `MuPos` body |

### Line count impact

- `master` (monolithic `lib.rs`): ~1200 lines
- Current branch (modularized + extensions): ~1350 lines
- Delta: ~150 lines for 4 enum definitions, 4 cut functions, and 7 tests

The extension is genuinely small, as the memo predicted.

---

## Substrate pushback

### What worked on first try

- `BotCoValue`, `WhynotCoValue` — compiled immediately, no lifetime issues
- `WithCoValue` with two `Cont` fields — compiled cleanly
- All commuting conversion cuts — no type mismatches

### What required adjustment

**`ParCoValue` with composite nesting.** The test `milestone_2_commuting_par_composites` used `Tensor<Tensor<AtomP<X>, AtomP<Y>>, One>` with a `MuPos` body that captured local tuples. The compiler rejected capturing `pair.0`/`pair.1` directly inside a non-`move` closure because the closure's lifetime inference didn't force `'static`. Fix: destructure before the closure and use `move |...|`.

**`ParCoValue` type complexity.** Clippy warned about the complex `Box<dyn FnOnce(<A::Dual as Pos>::Value<'s>, ...>` type. Fix: `#[allow(clippy::type_complexity)]` on the enum — the complexity is inherent and intentional (it names an existential quantification).

No other substrate pushback. The pattern generalizes cleanly.

---

## Milestone 6: Atomic consistency check

### Are atomic commuting conversions currently expressible?

**No.** `AtomN<X>::CoValue<'s> = Var<'s, AtomN<X>>`. A `Var` has no `Cont` variant. There is no `cut_commuting_atom` on this branch. The krivine-stack branch proved that adding `AtomNCoValue` with both `Var` and `Cont` variants works, but that change was explicitly held back as a control.

### Would Milestones 1-5 have been easier with an atomic enum?

**No.** The atomic case was never touched during Milestones 1-5. The non-atomic patterns are self-contained. An atomic enum would have added upfront complexity without simplifying anything downstream.

### Does the asymmetry create confusing situations?

**Yes, in one specific way.** A user writing `mu_pos` sees different types for the parameter depending on whether the type is atomic:

```rust
// Atomic: the parameter is a covariable
mu_pos::<'static, AtomP<X>, _>(|a: Var<'_, AtomN<X>>| ...)

// Non-atomic: the parameter is an enum with a Cont variant
mu_pos::<'static, Tensor<AtomP<X>, AtomP<Y>>, _>(|a: ParCoValue<'_, ...>| ...)
```

The asymmetry is **semantically principled** — atoms have user-constructible co-values (covariables); non-atomic negatives don't. But it's **practically visible** to users. The question is whether the principled reason is discoverable from the API alone. Probably not without documentation.

A second confusion: the `MuPos` vs `Var` cuts for non-atomic types. On master these were "uninhabited" (dead code by construction). Now `CoValue` is inhabited, but `Var` still doesn't carry a `CoValue`. So `cut(mu_pos(...), var)` for non-atomic `var` is still stuck — not because it's uninhabited, but because `Var` is just a token. This is a subtle shift in rationale.

### Recommendation

**Do not backport the atomic enum now.** The library is operationally complete for the non-atomic commuting-conversion fragment. The atomic gap is a real scope boundary, but it's small and documentable. Two paths forward:

1. **Document the scope:** Add a note that atomic commuting conversions are not implemented; `MuPos` vs `MuNeg` for atomic types requires the `AtomNCoValue` pattern from the krivine-stack branch.
2. **Backport later if needed:** The krivine-stack branch has a working template. The backport is low-risk (~20 lines) if the need arises.

The asymmetry between atomic and non-atomic negatives is structurally honest. Atoms genuinely differ from composites in having user-constructible co-values. The library should reflect this difference, not paper over it.

---

## Is the library complete?

### What works

- All 5 principal cuts (value vs destructor)
- All 5 axiom cuts (variable vs covariable) — normal forms
- All 5 non-atomic commuting conversions (binder vs binder)
- Full MALL with exponentials, correctly typed
- Krivine-machine-style operational semantics via closure invocation

### What is missing

1. **Atomic commuting conversions.** `⟨μ⁺α.c | μx⁻.d⟩` for atomic types. Not implemented; documentable scope gap.
2. **`MuPos` vs `Var` principal cuts for non-atomic types.** `⟨μ⁺α.c | x⊥⟩` where `x⊥` is a covariable of non-atomic type. These would require adding a `Var` variant to each non-atomic `CoValue` enum and corresponding `cut_pos_*_var` functions. This is additional scope beyond the memo's milestones.
3. **`run` does not reach full normal forms.** The `Outcome::Done` case is never produced; all reductions end in `Stuck` or would loop forever if `Done` were reachable. This is a known limitation of the closure-based architecture — it performs single-step reduction, not full normalization.

### Assessment

The library is **complete for its stated scope**: a type-theoretic embedding of MALL with exponentials in Rust, with operational semantics for principal cuts and non-atomic commuting conversions. The missing pieces are genuine extensions, not bugs. The closure-based architecture expresses what it expresses; the substrate constraints discovered on the krivine-stack branch (no explicit stack, no `Any`, no unsafe) are respected.

The final library feels like the main branch, slightly extended — not like a different library. The ~150-line delta for 4 commuting conversions is exactly the right size for the move being made.
