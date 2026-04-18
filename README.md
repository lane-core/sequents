# sequents

A research prototype embedding the multiplicative fragment of linear System L
into Rust's type system. Scope structure is carried by lifetimes; linearity is
enforced by move semantics; binders use higher-rank lifetime quantification
(`for<'x>`).

## What's inside

- **`src/lib.rs`** — the library: polarity traits, atoms, multiplicative
  connectives, variable tokens, expressions, co-expressions, commands,
  constructors, and atomic reduction functions.
- **`src/example.rs`** — concrete examples showing how to construct terms and
  apply reductions.
- **`NOTES.md`** — detailed notes on what worked, what didn't, and where Rust's
  type system cooperated or pushed back.

## Running

```bash
cargo test        # run the test suite
cargo check       # type-check the library
```

## Architectural thesis

1. **A term's scope of validity is a Rust lifetime.** A term at scope Γ is a
   Rust value parameterised by a lifetime `'s` that represents the region where
   Γ's variables are valid.

2. **The multiplicative connectives are scope operations.** Tensor `⊗`
   corresponds to lifetime intersection: `Tensor<A, B>::Value<'s> =
   (A::Value<'s>, B::Value<'s>)`. Rust computes the intersection automatically
   via covariance.

3. **Binders are higher-rank lifetime quantifications.** The μ-binder introduces
   a fresh variable via `for<'x>`. The body is a closure parameterised over
   `'x`; variable occurrences inside consume `Var<'x, A>` tokens.

4. **Linearity is enforced by non-Copy move semantics.** A `Var<'x, A>` is
   neither `Copy` nor `Clone`. Using it twice is a compile error.

5. **Polarity traits carry operational content.** `Pos` and `Neg` have
   associated types (`Dual`, `Value<'s>`, `CoValue<'s>`) and methods. The
   trait bounds enforce De Morgan structure.

6. **Types are in NNF by grammar.** No `Neg<A>` constructor. Duality is a
   method on the type tree.

## Friction points (see NOTES.md)

- **Composite-type reduction** requires an implicit polarity flip that NNF
  forbids. Reductions work for atoms but not for general composite types.
- **Nested binders** cannot capture outer non-'static variables because
  `for<'x>` closures must be valid for all lifetimes.
- **`FnOnce` trait objects** are not callable on stable Rust; custom traits
  with `self: Box<Self>` are used instead.
- **`CoExpr` enum** needs `Box<dyn Any>` for the `MuPar` variant because Rust
  lacks GADTs.

## License

This is a research prototype. Use it for thinking, not for shipping.
