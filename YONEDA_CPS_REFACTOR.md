# Memo: Yoneda Refactor — What the Substrate Taught Us

## What we set out to do

Restructure the sequents library so that `Term` and `Coterm` are symmetric three-variant enums, with one `cut` function dispatching over the 3×3 pairing. The theory suggested this was the right shape; the only way to know what "faithfully following the plan" entailed concretely was to build it.

## What got built

- `Term<'s, A>`: `Axiom(Resource)` / `Intro(A::Intro<'s>)` / `Mu(Box<dyn FnOnce(Coterm) -> Command>)`
- `Coterm<'s, N>`: `Axiom(Resource)` / `Elim(N::Elim<'s>)` / `MuTilde(Box<dyn FnOnce(Term) -> Command>)`
- One `cut` function with a 3×3 match
- `Principal` trait for symmetric pair-relations (intro meets elim)
- `AxiomElim` trait for elim-side resource consumption
- All six binder types deleted, all thirteen cut functions collapsed
- `Command` is now `Normal` / `Step(Box<dyn FnOnce() -> Command>)`

46/46 tests pass. `cargo clippy` clean. 1634 lines total. The library shrank by ~286 lines (31%).

## What the substrate taught us

### 1. GAT projection through duality fails in trait parameter position

The original plan put `axiom_elim` on `Principal` alongside `principal`. Lane corrected this mid-refactor: `axiom_elim` belongs on the elim types, not on Pos. The corrected structure uses a new `AxiomElim<'s>` trait.

The first attempt at the corrected structure used a type parameter:

```rust
pub trait AxiomElim<'s, A: Pos> {
    fn axiom_elim(self, resource: Resource<'s, A>) -> Command<'s>;
}
```

This fails for composite types. Writing:

```rust
impl<'s, A: Neg, B: Neg> AxiomElim<'s, Plus<A::Dual, B::Dual>> for WithElim<'s, A, B>
```

the compiler cannot prove `Plus<A::Dual, B::Dual>: Pos` from `A: Neg, B: Neg` in the trait parameter position. The associated-type normalization `(A::Dual)::Dual = A` does not propagate there.

**Fix:** Restructure to an associated type:

```rust
pub trait AxiomElim<'s> {
    type PosType: Pos;
    fn axiom_elim(self, resource: Resource<'s, Self::PosType>) -> Command<'s>;
}
```

Then the bound becomes `type Elim<'s>: AxiomElim<'s, PosType = Self::Dual>` on `Neg`. This compiles cleanly. The GAT + trait bound projection through duality works; the composite type in trait parameter position does not.

### 2. Callable fields need parentheses

Elim structs like `ParElim` have `pub body: Box<dyn FnOnce(Term, Term) -> Command>`. Writing `k.body(a, b)` is parsed as a method call. Rust requires `(k.body)(a, b)`. This is a syntax-level detail that affects every elim call site in `Principal` impls and in tests that pattern-match on `Coterm::Elim`.

This was unanticipated. The fix is mechanical but pervasive — about a dozen sites needed the parentheses.

### 3. `impl FnOnce` in constructor signatures eliminates turbofish

Converting `mu_pos::<'static, AtomP<X>, _>` to `mu::<'static, AtomP<X>>` (dropping the closure-type placeholder) was predicted. What was satisfying was doing it for all constructors: `mu`, `mu_tilde`, `mu_atom`, `mu_unit`, `mu_par`, `mu_case`, `mu_bang`, `promote`. Every constructor now takes exactly the type parameters that matter (the connectives), with closure types inferred.

This eliminated approximately 40 `_` placeholders from the codebase. The ergonomics improvement is real — construction sites now read as `mu_par::<AtomP<X>, AtomP<Y>>(|a, b| ...)` instead of `mu_par::<'static, AtomP<X>, AtomP<Y>, _>(|a, b| ...)`.

### 4. Principal cuts must wrap in `Step` — the substrate said so

The first draft of `cut` had principal cuts and axiom-elim cuts return `Command` directly, while commuting conversions and mu dispatch wrapped in `Step`. A canonical-form test asserted this was wrong: it expected `Step` wrapping for every non-terminal case, and the assertion failed.

The initial interpretation was that the asymmetry reflected a real operational distinction. This was backwards. The substrate was showing that the asymmetry was an artifact, not a feature. Every non-terminal reduction is one reduction step, observable as one `Step` layer. The `Step` wrapper isn't overhead to avoid — it's the physical trace of a reduction event, which is what the library represents.

The corrected 3×3 match:

- Axiom/Axiom → `Normal`
- Axiom/Elim → `Step(Box::new(move || e.axiom_elim(v)))`
- Axiom/MuTilde → `Step(Box::new(...))`
- Intro/Axiom → `Normal`
- Intro/Elim → `Step(Box::new(move || A::principal(i, e)))`
- Intro/MuTilde → `Step(Box::new(...))`
- Mu/anything → `Step(Box::new(...))`

Uniform: every non-terminal case produces `Step`. `run()` peels one layer per reduction. One `Box` allocation per reduction step — the physical trace of the event.

### 5. `AxiomElim` as a separate trait is the right structural placement

Lane's mid-refactor correction — moving `axiom_elim` off `Principal` and onto elim types — proved correct in implementation. The two dispatch mechanisms in `cut` now match genuinely distinct cases:

- `e.axiom_elim(v)` — elim consumes resource (asymmetric, elim owns the logic)
- `A::principal(i, e)` — symmetric pair-relation (neither side owns it)

Bundling them under one trait was algorithmic grouping, not structural observation. Splitting them keeps the refactor coherent with itself.

### 6. The library is smaller at every level

| Component | Before | After | Δ |
|---|---|---|---|
| Cut functions | 13 | 1 | −12 |
| Binder types | 6 | 0 | −6 |
| `Outcome` enum | 1 | 0 | −1 |
| `StuckReason` enum | 1 | 0 | −1 |
| Marker traits (`Expr`/`CoExpr`) | 2 | 0 | −2 |
| `Var` type | 1 | 0 (merged into `Resource`) | −1 |
| Library lines | ~928 | ~642 | −286 |

Every asymmetry that got resolved reduced code. This is the eighth time in this project that pattern has held.

## Resolution

The open question about `Step` wrapping is closed. The substrate's pushback was correct: uniform `Step` wrapping for all non-terminal reductions is the right shape. The library now implements this uniformly.

The refactor is complete. 46/46 tests pass. `cargo clippy` clean. 1634 lines total.
