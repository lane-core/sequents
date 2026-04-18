# Memo: HRTB Investigation Results

## Question 1: Is `for<'x>` compatible with capture?

**Short answer: Only for `'static` captures.**

### The experiment

Tested the proposed signature:

```rust
pub fn mu_neg_hrtb<'outer, F>(body: F) -> Command<'outer>
where
    F: for<'x> FnOnce(Var<'x, ()>) -> Command<'x> + 'outer,
```

Against four cases:

| Case | Capture lifetime | Result |
|---|---|---|
| Specific lifetime + `'static` capture | `'static` | ✓ Compiles |
| Specific lifetime + non-`'static` capture | `'a` (generic) | ✓ Compiles |
| HRTB + `'static` capture | `'static` | ✓ Compiles |
| HRTB + non-`'static` capture | `'a` (generic) | ✗ **Fails** |

### The error

```
lifetime may not live long enough
returning this value requires that 'a must outlive 'static
```

### Why it fails

The HRTB requires the closure body to work for **all** `'x`, including `'static`. When the body captures `outer: Var<'a, ()>` and uses it in `cut(inner, outer)`, Rust needs `Var<'a, ()>` to be usable as `Var<'x, ()>` for all `'x`. By covariance, this requires `'a: 'x` for all `'x`. Since `'x` includes `'static`, this requires `'a: 'static`. But `'a` is a generic lifetime parameter — it does not necessarily outlive `'static`.

### What this means

The `for<'x>` quantifier is **universal** over all lifetimes. It cannot be restricted to "lifetimes shorter than `'a`". So any captured data with lifetime-bearing types (like `Var`) must itself be at `'static` to satisfy the universal quantification.

This is exactly the limitation of the first pass: `+ 'static` on closures wasn't an accidental over-constraint — it was a **necessary consequence** of combining HRTB with capture of lifetime-carrying data.

### Conclusion for Question 1

HRTB + capture of `Var` values is **only compatible with `'static` captures**. The first pass was correct in requiring `+ 'static` for boxed closures. The rewrite correctly identified that dropping `'static` enabled capture, but the price was dropping HRTB as well.

**You cannot have both HRTB freshness and non-`'static` capture of `Var` in Rust's type system.** This is a genuine limit, not a workaround gap.

---

## Question 2: Can confused code compile under the current signature?

**Short answer: No — move semantics prevent all forms of confusion that would produce ill-formed terms.**

### What was tested

1. **Double use**: `cut(x, x)` — correctly rejected (`E0382: use of moved value`)
2. **Cross-binder reuse**: Using `outer` in `_b1` and then in `_b2` — correctly rejected
3. **Parameter swap**: Using `x` (from outer binder) where `y` (from inner binder) is expected — compiles, but is **semantically valid** (α-equivalence)
4. **Escape via Vec/RefCell**: Possible in principle if closures are invoked, but irrelevant to static well-formedness

### Why confusion is impossible

In the current library, variables are distinguished by **value identity**, not lifetime identity. Two `Var<'s, A>` values are distinct values. Move semantics ensure each value is used exactly once. There is no operation that allows:
- Duplicating a variable (no `Clone`/`Copy`)
- Forging a variable (only `Var::new()` and binders create them)
- Escaping a variable from its binder scope (closures consume parameters)

### The "same lifetime" issue is cosmetic

Two binders at `'static` both introduce variables at `'static`. At the type level, `Var<'static, AtomP<X>>` is the same type for both. But the values are distinct. This is exactly α-equivalence: variables are interchangeable under renaming, and Rust's value semantics implement this correctly.

There is **no soundness hole**. The weakening from HRTB to specific lifetime does not admit any ill-formed System L terms.

### Conclusion for Question 2

The current specific-lifetime signature is **sound**. The only thing "lost" is the type-level guarantee that each binder introduces a *syntactically distinct* lifetime — but this guarantee was always cosmetic because Rust's lifetime equality is structural, not nominal. Two lifetimes that unify are the same lifetime; there's no way to enforce "freshness" at the type level without HRTB, and HRTB is incompatible with capture.

---

## The meta-question: strong vs weak form

| Form | Claim | Status |
|---|---|---|
| **Strong** | Rust enforces all of System L's well-formedness, including type-level freshness | **Unachievable** — HRTB + non-`'static` capture is incompatible |
| **Weak** | Rust enforces scope safety and linearity; α-equivalence is maintained by value semantics | **Achieved** — the current library delivers this |

The strong form was the original architectural bet. The investigation shows it fails not because of a missing workaround, but because of a **fundamental tension** in Rust's type system: universal quantification over lifetimes (`for<'x>`) requires captured lifetime-carrying data to outlive all quantified lifetimes, which forces `'static`.

This is a **real finding** worth documenting. It says: Rust's lifetime system can carry scope structure and enforce linearity, but it cannot simultaneously provide universal freshness quantification and capture of non-`'static` scoped data. You must choose one. The library chooses capture (enabling nested binders) over HRTB freshness (which was cosmetic anyway).

---

## Recommendations

1. **Keep the current specific-lifetime design.** It is sound, enables nested binders, and eliminates the enum-based workarounds.

2. **Document the HRTB limitation in NOTES.md.** This is research output — it tells us a specific limit of Rust's type system.

3. **Do not attempt to reintroduce HRTB.** It would reintroduce the capture limitation without adding any actual safety (freshness is already guaranteed by value identity).

4. **Proceed to Phase 2 (operational semantics) if desired.** The static guarantees are settled.
