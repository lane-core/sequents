# sequents

A research prototype embedding the multiplicative fragment of linear System L
into Rust's type system. Scope structure is carried by lifetimes; linearity is
enforced by move semantics.

## What's inside

- **`src/lib.rs`** — the library: polarity traits, atoms, multiplicative
  connectives, variable tokens, expression/co-expression traits (tagless-final
  style), binder types, constructors.
- **`src/example.rs`** — concrete examples showing how to construct terms,
  including nested binders.
- **`NOTES.md`** — detailed research notes on both design iterations (enum-based
  first pass and trait-based Option A), what worked, what didn't, and why.

## Running

```bash
cargo test        # 9 tests, all passing
cargo clippy      # clean
cargo check       # type-check the library
```

## Architectural thesis

1. **A term's scope of validity is a Rust lifetime.** A term at scope Γ is a
   Rust value parameterised by a lifetime `'s`.

2. **The multiplicative connectives are scope operations.** Tensor `⊗`
   corresponds to lifetime intersection: `Tensor<A, B>::Value<'s> =
   (A::Value<'s>, B::Value<'s>)`. Rust computes the intersection automatically
   via covariance.

3. **Binders use specific lifetimes.** Each binder is scoped at a lifetime `'s`
   shared with the ambient context. The body closure can capture variables from
   that scope, enabling nested binders. Freshness is guaranteed by closure
   parameter scoping, not by `for<'x>` quantification.

4. **Linearity is enforced by non-Copy move semantics.** A `Var<'x, A>` is
   neither `Copy` nor `Clone`. Using it twice is a compile error.

5. **Polarity traits carry operational content.** `Pos` and `Neg` have
   associated types (`Dual`, `Value<'s>`, `CoValue<'s>`). The trait bounds
   enforce De Morgan structure.

6. **Types are in NNF by grammar.** No `Neg<A>` constructor. Duality is a
   method on the type tree.

7. **Expressions and co-expressions are traits, not enums.** This is the key
   design move (Option A). Each introduction form is its own type implementing
   the appropriate trait. No `Box<dyn Any>`, no custom binder traits, no
   `+ 'static` bounds.

## Key finding: nested binders work

In the first pass (enum-based AST), nested binders that capture outer variables
failed to compile because `for<'x>` combined with `'static` boxing prevented
capture. The trait-based rewrite eliminates both restrictions:

```rust
let w: Var<'static, AtomN<Y>> = Var::new();
let co = mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|x, y| {
    cut(x, mu_neg::<'_, AtomN<X>, _>(|_z| cut(y, w)))
});
```

This term — `μ(x ⅋ y).⟨x | μz⁻.⟨y | w⟩⟩` — compiles and type-checks.

## License

This is a research prototype. Use it for thinking, not for shipping.
