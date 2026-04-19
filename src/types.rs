use std::marker::PhantomData;

use crate::machine::Command;

// =============================================================================
// 0.  Linear resource token
// =============================================================================

/// A linear resource token at type `A` in scope `'x`.
///
/// Non-`Copy`, non-`Clone` — using it twice is a compile error.
/// The lifetime `'x` is the scope in which this resource is valid.
pub struct Resource<'x, A> {
    _marker: PhantomData<&'x ()>,
    _type: PhantomData<A>,
}

impl<'x, A> Resource<'x, A> {
    /// Create a fresh resource token.
    pub fn new() -> Self {
        Resource {
            _marker: PhantomData,
            _type: PhantomData,
        }
    }
}

impl<'x, A> Default for Resource<'x, A> {
    fn default() -> Self {
        Self::new()
    }
}

// Explicitly NOT implementing Clone or Copy.
// Move semantics enforce linearity.

// =============================================================================
// 1.  Uniform term and coterm enums
// =============================================================================

/// A positive term at scope `'s`.
///
/// Three variants, symmetric with `Coterm` under duality:
/// - `Axiom`: the axiom rule — a resource is a term of its type.
/// - `Intro`: a structural introduction form (per-connective).
/// - `Mu`: the μ-binder, a general continuation closure.
pub enum Term<'s, A: Pos> {
    Axiom(Resource<'s, A>),
    Intro(A::Intro<'s>),
    Mu(Box<dyn FnOnce(Coterm<'s, A::Dual>) -> Command<'s> + 's>),
}

/// A negative coterm at scope `'s`.
///
/// Three variants, symmetric with `Term` under duality:
/// - `Axiom`: the axiom rule — a resource is a coterm of its type.
/// - `Elim`: a structural elimination form (per-connective).
/// - `MuTilde`: the μ̃-binder, a general continuation closure.
pub enum Coterm<'s, N: Neg> {
    Axiom(Resource<'s, N>),
    Elim(N::Elim<'s>),
    MuTilde(Box<dyn FnOnce(Term<'s, N::Dual>) -> Command<'s> + 's>),
}

impl<'s, A: Pos> From<Resource<'s, A>> for Term<'s, A> {
    fn from(r: Resource<'s, A>) -> Self {
        Term::Axiom(r)
    }
}

impl<'s, N: Neg> From<Resource<'s, N>> for Coterm<'s, N> {
    fn from(r: Resource<'s, N>) -> Self {
        Coterm::Axiom(r)
    }
}

// =============================================================================
// 2.  Polarity traits
// =============================================================================

/// Positive types: atoms `X`, unit `1`, tensor `A ⊗ B`, plus `A ⊕ B`, bang `!A`.
pub trait Pos: Sized + 'static {
    /// The De Morgan dual — a negative type.
    type Dual: Neg<Dual = Self>;
    /// The concrete introduction form at scope `'s`.
    /// For atomic types this is `Infallible` (no intro form besides axioms).
    type Intro<'s>;
}

/// Negative types: dual atoms `X⊥`, unit `⊥`, par `A ⅋ B`, with `A & B`, whynot `?A`.
pub trait Neg: Sized + 'static {
    /// The De Morgan dual — a positive type.
    type Dual: Pos<Dual = Self>;
    /// The concrete elimination form at scope `'s`.
    type Elim<'s>: Substitution<'s, PosType = Self::Dual>;
}

// =============================================================================
// 3.  Reduction cut trait (pair-relation)
// =============================================================================

/// The reduce cut rule for the positive connective `A` and its dual.
///
/// This is a pair-relation between `A`'s intro forms and its dual's elim
/// forms. The trait is placed on `Pos` for Rust's sake; it could
/// equivalently be placed on `Neg`. The reduction itself is symmetric —
/// either side describes the same rule.
pub trait Reduction: Pos
where
    Self::Dual: Neg,
{
    /// Reduction cut: intro form meets elim form. For atoms, unreachable
    /// (`match intro {}` on `Infallible`). For composites, destructures the
    /// intro and invokes the elim's body.
    fn reduce<'s>(intro: Self::Intro<'s>, elim: <Self::Dual as Neg>::Elim<'s>) -> Command<'s>;
}

// =============================================================================
// 4.  Axiom-elim interaction (elim-side dispatch)
// =============================================================================

/// Axiom-elim interaction: an Elim consumes a Resource.
///
/// The reduction logic lives with the Elim because the Elim has the body.
/// For atoms, the body receives the resource as a term.
/// For composites, the interaction is blocked — returns `Normal`.
pub trait Substitution<'s> {
    type PosType: Pos;
    fn substitute(self, resource: Resource<'s, Self::PosType>) -> Command<'s>;
}

// =============================================================================
// 5.  Atoms
// =============================================================================

/// Positive atom `X`.
pub struct AtomP<X>(PhantomData<X>);

/// Negative atom `X⊥`.
pub struct AtomN<X>(PhantomData<X>);

impl<X: 'static> Pos for AtomP<X> {
    type Dual = AtomN<X>;
    type Intro<'s> = std::convert::Infallible;
}

impl<X: 'static> Neg for AtomN<X> {
    type Dual = AtomP<X>;
    type Elim<'s> = AtomElim<'s, X>;
}

/// Elimination form for `AtomN<X>`: body consuming a positive term.
pub struct AtomElim<'s, X: 'static> {
    pub body: Box<dyn FnOnce(Term<'s, AtomP<X>>) -> Command<'s> + 's>,
}

impl<X: 'static> Reduction for AtomP<X> {
    fn reduce<'s>(intro: Self::Intro<'s>, _elim: <Self::Dual as Neg>::Elim<'s>) -> Command<'s> {
        match intro {}
    }
}

impl<'s, X: 'static> Substitution<'s> for AtomElim<'s, X> {
    type PosType = AtomP<X>;
    fn substitute(self, resource: Resource<'s, AtomP<X>>) -> Command<'s> {
        (self.body)(Term::Axiom(resource))
    }
}

// =============================================================================
// 6.  Multiplicative units
// =============================================================================

/// Positive unit `1`.
pub struct One;

/// Negative unit `⊥`.
pub struct Bot;

impl Pos for One {
    type Dual = Bot;
    type Intro<'s> = ();
}

impl Neg for Bot {
    type Dual = One;
    type Elim<'s> = BotElim<'s>;
}

/// Elimination form for `Bot`: a zero-argument body.
pub struct BotElim<'s> {
    pub body: Box<dyn FnOnce() -> Command<'s> + 's>,
}

impl Reduction for One {
    fn reduce<'s>(_intro: Self::Intro<'s>, elim: <Self::Dual as Neg>::Elim<'s>) -> Command<'s> {
        (elim.body)()
    }
}

impl<'s> Substitution<'s> for BotElim<'s> {
    type PosType = One;
    fn substitute(self, _resource: Resource<'s, One>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 7.  Multiplicative connectives
// =============================================================================

/// Tensor `A ⊗ B` (both components positive).
pub struct Tensor<A: Pos, B: Pos>(PhantomData<(A, B)>);

/// Par `A ⅋ B` (both components negative).
pub struct Par<A: Neg, B: Neg>(PhantomData<(A, B)>);

impl<A: Pos, B: Pos> Pos for Tensor<A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    type Dual = Par<A::Dual, B::Dual>;
    type Intro<'s> = (Term<'s, A>, Term<'s, B>);
}

impl<A: Neg, B: Neg> Neg for Par<A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type Dual = Tensor<A::Dual, B::Dual>;
    type Elim<'s> = ParElim<'s, A::Dual, B::Dual>;
}

/// Elimination form for `Par<A, B>`: body consuming two terms.
pub struct ParElim<'s, A: Pos, B: Pos> {
    pub body: Box<dyn FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s> + 's>,
}

impl<A: Pos, B: Pos> Reduction for Tensor<A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    fn reduce<'s>(intro: Self::Intro<'s>, elim: <Self::Dual as Neg>::Elim<'s>) -> Command<'s> {
        let (a, b) = intro;
        (elim.body)(a, b)
    }
}

impl<'s, A: Pos, B: Pos> Substitution<'s> for ParElim<'s, A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    type PosType = Tensor<A, B>;
    fn substitute(self, _resource: Resource<'s, Tensor<A, B>>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 8.  Additive connectives
// =============================================================================

/// Positive sum `A ⊕ B` (choice made at introduction time).
pub struct Plus<A: Pos, B: Pos>(PhantomData<(A, B)>);

/// Negative with `A & B` (choice made at destruction time).
pub struct With<A: Neg, B: Neg>(PhantomData<(A, B)>);

/// Introduction form for `Plus<A, B>`: either left or right injection.
pub enum PlusIntro<'s, A: Pos, B: Pos> {
    Inl(Term<'s, A>),
    Inr(Term<'s, B>),
}

impl<A: Pos, B: Pos> Pos for Plus<A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    type Dual = With<A::Dual, B::Dual>;
    type Intro<'s> = PlusIntro<'s, A, B>;
}

impl<A: Neg, B: Neg> Neg for With<A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type Dual = Plus<A::Dual, B::Dual>;
    type Elim<'s> = WithElim<'s, A, B>;
}

/// Elimination form for `With<A, B>`: two continuations, one per injection.
pub struct WithElim<'s, A: Neg, B: Neg>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    pub left: Box<dyn FnOnce(Term<'s, A::Dual>) -> Command<'s> + 's>,
    pub right: Box<dyn FnOnce(Term<'s, B::Dual>) -> Command<'s> + 's>,
}

impl<A: Pos, B: Pos> Reduction for Plus<A, B>
where
    A::Dual: Neg,
    B::Dual: Neg,
{
    fn reduce<'s>(intro: Self::Intro<'s>, elim: <Self::Dual as Neg>::Elim<'s>) -> Command<'s> {
        match intro {
            PlusIntro::Inl(a) => (elim.left)(a),
            PlusIntro::Inr(b) => (elim.right)(b),
        }
    }
}

impl<'s, A: Neg, B: Neg> Substitution<'s> for WithElim<'s, A, B>
where
    A::Dual: Pos,
    B::Dual: Pos,
{
    type PosType = Plus<A::Dual, B::Dual>;
    fn substitute(self, _resource: Resource<'s, Plus<A::Dual, B::Dual>>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 9.  Exponential connectives
// =============================================================================

/// Positive exponential `!A` — duplicable terms.
pub struct Bang<A: Pos>(PhantomData<A>);

/// Negative exponential `?A` — dual of `!A`.
pub struct Whynot<N: Neg>(PhantomData<N>);

/// Introduction form for `Bang<A>`: a duplicable producer of terms.
pub struct BangIntro<'s, A: Pos> {
    pub(crate) producer: std::rc::Rc<dyn Fn() -> Term<'s, A> + 's>,
    pub(crate) _marker: PhantomData<&'s ()>,
}

impl<'s, A: Pos> Clone for BangIntro<'s, A> {
    fn clone(&self) -> Self {
        BangIntro {
            producer: self.producer.clone(),
            _marker: PhantomData,
        }
    }
}

impl<A: Pos> Pos for Bang<A>
where
    A::Dual: Neg,
{
    type Dual = Whynot<A::Dual>;
    type Intro<'s> = BangIntro<'s, A>;
}

impl<N: Neg> Neg for Whynot<N>
where
    N::Dual: Pos,
{
    type Dual = Bang<N::Dual>;
    type Elim<'s> = WhynotElim<'s, N>;
}

/// Elimination form for `Whynot<N>`: body consuming a `BangIntro`.
pub struct WhynotElim<'s, N: Neg> {
    pub body: Box<dyn FnOnce(BangIntro<'s, N::Dual>) -> Command<'s> + 's>,
}

impl<A: Pos> Reduction for Bang<A>
where
    A::Dual: Neg,
{
    fn reduce<'s>(intro: Self::Intro<'s>, elim: <Self::Dual as Neg>::Elim<'s>) -> Command<'s> {
        (elim.body)(intro)
    }
}

impl<'s, N: Neg> Substitution<'s> for WhynotElim<'s, N>
where
    N::Dual: Pos,
{
    type PosType = Bang<N::Dual>;
    fn substitute(self, _resource: Resource<'s, Bang<N::Dual>>) -> Command<'s> {
        Command::Normal
    }
}
