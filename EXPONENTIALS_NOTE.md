# On Exponentials and the Current Architecture

## What they are

In linear logic, `!A` ("of course A") and `?A` ("why not A") are the
exponential modalities. They control the structural rules:

- `!A` is duplicable and discardable: you can use it zero, one, or many times
- `?A` is the dual: a context that can receive zero, one, or many copies

In System L, this translates to:
- Values of type `!A` can be copied or dropped
- Co-values of type `?A` can be copied or dropped

## Why this is hard for the current library

The entire architecture is built on one invariant: **all values are linear**.
`Var<'s, A>` is non-Copy, non-Clone. Move semantics enforce exactly-once use.
This is the engine that makes everything else work — scope safety, freshness,
static well-formedness.

Exponentials break this invariant. A `!A` value *must* be `Clone`. If it
weren't, you couldn't duplicate it. But if `!A::Value<'s>` is `Clone`, then
`Var<'s, A>` (which is `!A::Value<'s>` when `A` is an atom) would need to be
`Clone`, which means it wouldn't be linear anymore.

## The design tension

There are two ways to handle this, and neither is obviously right:

**Option 1: Separate the linear and classical worlds.**

Keep `Var` non-Clone. Introduce a new type family for classical values:
```rust
pub trait Classical: Pos { type Value<'s>: Clone; }
```
`!A` would be a wrapper that provides `Clone` for any `A`. But then you need
parallel trait hierarchies, and the boundary between linear and classical
becomes a design problem at every connective.

**Option 2: Make `Var` conditionally Clone.**

Add a type parameter or trait bound that marks whether a variable is
exponential. But this infects every type in the library — `Expr<'s, A>`
would need to know whether `A` is exponential, which cascades through `Pos`,
`Neg`, `Command`, and all binders. The complexity increase is substantial.

**Option 3: Don't embed `!` and `?` at the value level.**

Treat exponentials as command-level annotations rather than type-level
constructors. This is closer to how some proof nets handle them: the
exponential structure exists in the proof net (the command graph), not in
the underlying data. But this changes the character of the library from
"typed lambda calculus" to "proof net machine," which is a different
architectural bet.

## Why this needs a conversation

The choice between these options isn't a local workaround — it's a
foundational commitment about what the library is:

- Option 1 keeps the linear core pure but doubles the type surface
- Option 2 makes the library more expressive but significantly more complex
- Option 3 changes the architectural thesis entirely

None of these are wrong. But they serve different purposes, and the purpose
determines the choice. Does Lane want:

(a) A pedagogically clean embedding of full linear logic (probably Option 1)
(b) A practically usable programming language with linear types (probably
    Option 2 with heavy macro sugar)
(c) A proof net reduction engine disguised as a Rust library (Option 3)

## What I would suggest

If the goal is (a), start with Option 1. Keep `Var` non-Clone. Define
`Bang<A: Pos>` with `Bang<A>::Value<'s> = !A::Value<'s>` where the `!`
provides duplication machinery (a stack of copies, or an `Rc` reference).
The binder for `!A` would be something like `MuBang<'s, A, F>` where the
body receives a `BangValue<'s, A>` that can be `.clone()`-d.

This preserves the linear core while adding a classical extension. The cost
is that `Bang` values feel different from `Var` values — they don't move,
they copy. This is semantically correct for exponentials but ergonomically
jarring.

## The honest bottom line

The multiplicative and additive fragments were clean because they don't
contradict the linearity invariant. Exponentials do. Implementing them
requires either relaxing the invariant (with consequences for the whole
architecture) or building a parallel system (with consequences for
complexity). This is why the memo said to stop and talk first.
