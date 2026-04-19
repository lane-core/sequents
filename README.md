# sequents

A Rust embedding of linear classical System L. Formulas are types, proofs are programs, and cut-elimination is evaluation. Scope structure is carried by lifetimes; linearity is enforced by move semantics.

This library implements the λμ̃μ-calculus — classical sequent calculus as a programming language. Terms and coterms are symmetric three-variant enums. One `cut` function dispatches over the 3×3 pairing. Per-connective reduction is handled by the `Positive::interact` and `Negative::resolve` methods.

## What this is

A research prototype, not a production library. It demonstrates that linear classical L can be expressed directly in a modern type system, with the operational semantics (Krivine-machine style) executable as Rust code.

## Quick start

```toml
[dependencies]
sequents = "0.1.1"
```

Atoms are user-defined types that implement `Positive` or `Negative`:

```rust
use sequents::*;

struct X;          // positive atom
struct XTag;       // its negative dual

pub struct XElim<'s> {
    pub body: Box<dyn FnOnce(X) -> Command<'s> + 's>,
}

impl Positive for X {
    type Dual = XTag;
    type Intro<'s> = std::convert::Infallible;
    type Witness<'x> = X;
    fn interact<'s>(i: Self::Intro<'s>, _: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
        match i {}
    }
}

impl Negative for XTag {
    type Dual = X;
    type Elim<'s> = XElim<'s>;
    type Witness<'x> = XElim<'x>;
    fn resolve<'s>(e: Self::Elim<'s>, r: Resource<'s, Self::Dual>) -> Command<'s> {
        (e.body)(r.into_witness())
    }
}

// Axiom cut: ⟨x | y⟩ — already in normal form
let x: Resource<'static, X> = Resource::new(X);
let y: CoResource<'static, XTag> = CoResource::new(XElim { body: Box::new(|_| Command::Normal) });
let cmd = cut(x.into(), y.into());
assert!(matches!(run(cmd), Command::Normal));
```

A reduction:

```rust
let x: Resource<'static, X> = Resource::new(X);
let z: CoResource<'static, XTag> = CoResource::new(XElim { body: Box::new(|_| Command::Normal) });

// μ̃y.⟨y | z⟩ — a coterm that substitutes its argument into a cut with z
let coterm = mu_tilde::<'static, XTag>(|y| cut(y, z.into()));

// ⟨x | μ̃y.⟨y | z⟩⟩ reduces in one step to ⟨x | z⟩
let cmd = cut(Term::Axiom(x), coterm);
let outcome = run(cmd);
assert!(matches!(outcome, Command::Normal));
```

Tensor and par:

```rust
struct Y;
struct YTag;
// ... impl Positive for Y, Negative for YTag (same pattern as X)

let x: Resource<'static, X> = Resource::new(X);
let y: Resource<'static, Y> = Resource::new(Y);
let pair = tensor(x, y);

let coterm = mu_par::<'static, X, Y>(|a, b| {
    let m: CoResource<'_, XTag> = CoResource::new(XElim { body: Box::new(|_| Command::Normal) });
    let n: CoResource<'_, YTag> = CoResource::new(YElim { body: Box::new(|_| Command::Normal) });
    let _ = cut(a, m.into());
    cut(b, n.into())
});

let _cmd = cut(pair, coterm);
```

## Core concepts

- **`Term<'s, A>`** — positive terms: axiom (variable), introduction (constructors), or μ-binder (control operator)
- **`Coterm<'s, N>`** — negative coterms: axiom (covariable), elimination (destructors), or μ̃-binder (control operator)
- **`cut(term, coterm)`** — pair a term with a coterm of dual type, producing a `Command`
- **`Command<'s>`** — either `Normal` (canonical form) or `Step` (one reduction remaining)
- **`run(cmd)`** — drive a command to normal form
- **`Resource<'x, A>`** — a linear-use token carrying a `A::Witness<'x>`. Non-`Copy`, non-`Clone`. Move semantics enforce single use.
- **`CoResource<'x, N>`** — the negative counterpart to `Resource`, carrying an `N::Witness<'x>`.

## Connectives

| Positive | Negative | Description |
|----------|----------|-------------|
| User-defined `impl Positive` | User-defined `impl Negative` | Atomic types |
| `One` | `Bot` | Multiplicative unit (⊥) |
| `Tensor<A, B>` | `Par<A, B>` | Multiplicative conjunction/disjunction (⊗/⅋) |
| `Plus<A, B>` | `With<A, B>` | Additive disjunction/conjunction (⊕/&) |
| `Bang<A>` | `Whynot<N>` | Exponential modality (!/?) |

## Exponentials

`promote` wraps a witness in `Arc` for duplication. `derelict` unwraps one copy:

```rust
let bang = promote::<'static, X>(X);
let v1 = derelict::<'static, X>(bang.clone());
let v2 = derelict::<'static, X>(bang); // fresh term
```

## Documentation

Run `cargo doc --open` for full API documentation. Every public item has a doc comment explaining its theoretical basis.

Citations: `docs/CITATIONS.md`.

## References

- **MMM** — Mangel, Melliès, Munch-Maccagnoni. "Classical Notions of Computation and the Hasegawa–Thielecke Theorem." POPL 2026.
- **Spiwack** — Spiwack, Arnaud. "A Dissection of L." 2014.
- **Grokking** — Binder et al. "Grokking the Sequent Calculus." ICFP 2024.

See `docs/CITATIONS.md` for full bibliographic details.

## Tests

```
cargo test
```

56 tests covering structural typing, operational reduction, commuting conversions, canonical form production, and witness flow.

## License

BSD-3-Clause. Copyright (c) 2026, Lane Biocini.
