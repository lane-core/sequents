use std::marker::PhantomData;

/// A variable (or covariable) token.
///
/// * Non-`Copy`, non-`Clone` — using it twice is a compile error.
/// * The lifetime `'x` is the scope in which this variable is valid.
/// * `PhantomData<&'x ()>` makes `Var` **covariant** in `'x`.
pub struct Var<'x, A> {
    _marker: PhantomData<&'x ()>,
    _type: PhantomData<A>,
}

impl<'x, A> Var<'x, A> {
    /// Create a fresh variable token.  In a real term, variables are
    /// introduced by binders; this constructor is useful for building
    /// open terms (e.g., the axiom rule) and for testing.
    pub fn new() -> Self {
        Var {
            _marker: PhantomData,
            _type: PhantomData,
        }
    }
}

impl<'x, A> Default for Var<'x, A> {
    fn default() -> Self {
        Self::new()
    }
}

// Explicitly NOT implementing Clone or Copy.
// Move semantics enforce linearity.
