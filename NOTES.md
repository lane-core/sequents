# Notes: Linear System L in Rust via Lifetimes

## What worked

### 1. NNF enforced by grammar
No `Neg<A>` type constructor exists. Positive and negative types are disjoint
families (`AtomP`/`AtomN`, `One`/`Bot`, `Tensor`/`Par`). Duality is an
associated-type computation (`Pos::Dual`, `Neg::Dual`) that always produces a
type in NNF. The involution `A::Dual::Dual = A` is a structural identity on
the type tree, not a propositional equation.

### 2. Polarity traits carry operational content
`Pos` and `Neg` are not marker traits. They have associated types
`Value<'s>` and `CoValue<'s>` that determine the runtime introduction forms.
`Tensor<A, B>::Value<'s> = (A::Value<'s>, B::Value<'s>)` is the crucial line:
Rust computes the lifetime intersection automatically via covariance.

### 3. `for<'x>` for binders works on stable Rust
Higher-rank lifetime quantification in function bounds compiles cleanly:

```rust
pub fn mu_pos<A: Pos, F>(body: F) -> Expr<'static, A>
where
    F: for<'x> FnOnce(Var<'x, A>) -> Command<'x>,
```

Each call to `mu_pos` introduces an opaque fresh lifetime. The body closure is
parameterised uniformly over `'x`. This gives α-equivalence, capture-avoidance,
and fresh-name generation for free — they are exactly what `for<'x>` does.

### 4. Linearity enforced by move semantics
`Var<'x, A>` is neither `Copy` nor `Clone`. Every occurrence consumes it.
Using a variable twice in the same scope is a compile error. Never using it
means the closure body fails to produce a well-formed command, because the
command constructors require the variable to be threaded through.

### 5. Covariance of `Var` is correct
We use `PhantomData<&'x ()>` (covariant) rather than an invariant phantom.
This was initially questioned, but testing confirms it is the right choice:
- A `Var<'static, A>` can be used in any shorter scope `'x` because `'static: 'x`.
- This allows closed terms (no free variables, effectively `'static`) to be
  nested inside binder bodies without lifetime mismatch.
- Move semantics prevent duplication; lifetime variance only controls where a
  value can be used, not how many times.

### 6. Custom binder traits solve `FnOnce` trait-object limitation
Stable Rust does not support calling `Box<dyn FnOnce(...)>` directly (the
`fn_traits` feature is unstable). We define custom traits with `self: Box<Self>`
receivers:

```rust
pub trait PosBinder<A: Pos> {
    fn call<'x>(self: Box<Self>, x: Var<'x, A>) -> Command<'x>;
}
```

These traits are object-safe and allow higher-rank lifetime bounds on the
method. Implementations for concrete closures delegate to `FnOnce`. This is a
small but necessary workaround.

### 7. Atomic reductions type-check cleanly
The negative-μ reduction `⟨x | μy⁻.c⟩ ▷ c[x/y]` and the tensor/par reduction
`⟨x ⊗ y | μ(a ⅋ b).c⟩ ▷ c[x/a, y/b]` both work for atoms because:
- `AtomP<X>::Value<'s> = Var<'s, AtomP<X>>`
- The binder body expects `Var<'x, AtomP<X>>` and receives exactly that.

## What did not work / required compromise

### 1. `CoExpr` enum needs `Box<dyn Any>` for `MuPar`
`CoExpr<'s, N: Neg>` is parameterised by `N`. The `MuPar<A, B>` destructor
does not fit into a generic `N` slot. We add a `MuPar(Box<dyn Any>)` variant
that is only constructible for `N = Par<_, _>` via an inherent impl. This is
type-safe at the API level (users can only construct it for the right type)
but the internal representation uses runtime type erasure. A GADT would solve
this; Rust does not have GADTs.

### 2. Composite-type reduction requires a polarity flip
For `Tensor<A, B>` where `A` or `B` is composite, `A::Value<'s>` is a pair
`(A1::Value<'s>, A2::Value<'s>)`, not a `Var<'s, A>`. The `MuPar` body expects
`Var<'x, A>` and `Var<'x, B>`. These only coincide with the values for atoms.

The one-sided reduction rules `c[V/x]` implicitly convert a negative value
(of dual type) into a positive variable during substitution. Our strict Rust
encoding separates `Expr` and `CoExpr`, `Var<'s, P>` and `Var<'s, N>`, and
cannot express this implicit polarity flip without an explicit isomorphism
that NNF forbids.

This is the deepest friction point: the one-sided calculus's reduction rules
rely on identifying a variable with its dual across the cut boundary. Rust's
type system keeps them distinct.

### 3. `Command` is opaque
To avoid lifetime invariance issues (trait objects with lifetime-returning
methods are invariant), `Command<'s>` stores no inspectable data. It is a ZST
with `PhantomData`. Reduction is implemented as standalone functions that take
the components of a reducible shape, not as methods on `Command` that
pattern-match. A full AST with type-erased cuts would require either:
- Specialisation (unstable)
- `Any` downcasting (requires `'static`, preventing open-term inspection)
- A GADT encoding (not expressible in Rust's enum system)

### 4. No `FnOnce` trait objects on stable Rust
As noted above, we work around this with custom traits. The workaround is
mechanical but adds boilerplate.

### 5. Nested binders cannot capture outer non-'static variables
A closure passed to `mu_neg` must satisfy `for<'x> FnOnce(Var<'x, A>) -> Command<'x>`.  
Rust requires such a closure to be `'static` (or at least not capture data tied  
to a specific outer lifetime).  This means a `mu_neg` inside a `mu_par` cannot  
capture the `mu_par`'s variables.  In the one-sided calculus, terms like  
`μ(x ⅋ y).⟨x | μz⁻.⟨y | w⟩⟩` are well-formed, but our encoding rejects them  
because the inner closure captures `y` from the outer scope.

This is a direct consequence of using `for<'x>` for freshness: the closure must  
work for *all* `'x`, so it cannot depend on a specific outer lifetime.

### 6. Involutive duality bound compiles, but with caveats
The recursive bound `Dual: Neg<Dual = Self>` on `Pos` (and symmetrically on
`Neg`) compiles on stable Rust. However, it propagates slowly through the
type system. In some contexts an explicit `where A::Dual: Neg` is needed even
though it is implied by `A: Pos`. This is a known sharp edge of associated
type projections.

## Summary

The architectural thesis holds for the **static** aspects of the encoding:
- Scope safety by construction ✓
- α-equivalence for free ✓
- Capture-avoiding substitution for free ✓
- Linearity via move semantics ✓
- No separate type-checking phase ✓

The **dynamic** aspect — reduction / substitution — is where Rust fights back.
The one-sided calculus's implicit polarity conversion during substitution does
not map cleanly to Rust's strict type separation. For atoms the reduction works
because `Value<'s>` coincides with `Var<'s, A>`. For composite types, a
structural mismatch remains.

This is a real finding, not a bug in the encoding. It says that faithfully
carrying scope via lifetimes gives you static well-formedness for free, but
dynamic rewriting across polarity boundaries needs an extra mechanism that Rust
does not provide natively.
