use std::marker::PhantomData;

use crate::machine::Command;

// =============================================================================
// 0.  Linear resource token
// =============================================================================

/// A linear resource token at type `A` in scope `'x`.
///
/// Under the resource-theoretic reading of linear logic, a formula is a
/// resource that gets consumed in proofs. A `Resource<'x, A>` is the
/// proof-theoretic token for such a resource: it carries no runtime data
/// (it is `PhantomData` under the hood), but it represents the right to
/// use a value of type `A` exactly once in scope `'x`.
///
/// Non-`Copy`, non-`Clone` — using it twice is a compile error. Move
/// semantics enforce linearity at the type-system level. This is the
/// axiom rule in sequent-calculus terms: a resource of type `A` is
/// trivially a term (or coterm) of type `A`.
///
/// See `Grokking §2] for an accessible introduction to the sequent-calculus
/// treatment of variables as first-class resources.
pub struct Resource<'x, A> {
    _marker: PhantomData<&'x ()>,
    _type: PhantomData<A>,
}

impl<'x, A> Resource<'x, A> {
    /// Create a fresh resource token.
    ///
    /// In a real term, resources are introduced by binders; this
    /// constructor is useful for building open terms (e.g. the axiom
    /// rule) and for testing.
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
/// In the λμμ̃-calculus, positive terms are the data side of the
/// duality. Three variants, symmetric with [`Coterm`] under De Morgan
/// duality:
///
/// * [`Term::Axiom`] — the axiom rule. A resource of type `A` is
///   trivially a term of type `A`. This is the identity proof.
///
/// * [`Term::Intro`] — a structural introduction form. Each positive
///   connective defines its own introduction rule (tensor pair, plus
///   injection, unit value, etc.). The associated type [`Positive::Intro<'s>`]
///   carries the per-connective payload.
///
/// * [`Term::Mu`] — the μ-binder. A general continuation closure that
///   receives a coterm of the dual type and produces a [`Command`].
///   This is the control operator on the positive side: it captures
///   the current evaluation context (covariable) as a first-class
///   object `Grokking §3.2].
///
/// The three-variant structure mirrors the classical sequent calculus:
/// variables (axiom), structural forms (intro/elim), and μ/μ̃-bound
/// continuations. See `MMM §7] and `Spiwack` for the underlying calculus.
pub enum Term<'s, A: Positive> {
    /// Axiom rule: a resource is a term of its type.
    Axiom(Resource<'s, A>),
    /// Structural introduction form (per-connective).
    Intro(A::Intro<'s>),
    /// μ-binder: general continuation closure.
    ///
    /// `μα.c` binds a covariable `α` of type `A::Dual` (negative).
    /// The body receives the coterm that `α` stands for.
    Mu(Box<dyn FnOnce(Coterm<'s, A::Dual>) -> Command<'s> + 's>),
}

/// A negative coterm at scope `'s`.
///
/// In the λμμ̃-calculus, negative coterms are the codata side of the
/// duality. Three variants, symmetric with [`Term`] under De Morgan
/// duality:
///
/// * [`Coterm::Axiom`] — the axiom rule. A resource of type `N` is
///   trivially a coterm of type `N`.
///
/// * [`Coterm::Elim`] — a structural elimination form. Each negative
///   connective defines its own elimination rule (par destructor,
///   with case analysis, bottom destructor, etc.). The associated
///   type [`Negative::Elim<'s>`] carries the per-connective payload.
///
/// * [`Coterm::MuTilde`] — the μ̃-binder. A general continuation closure
///   that receives a term of the dual type and produces a [`Command`].
///   This is the control operator on the negative side: it captures
///   the current term (variable) as a first-class object
///   `Grokking §3.2].
///
/// The symmetry between [`Term`] and [`Coterm`] is not superficial
/// sameness — it is the theory's duality made visible. Data and
/// codata are exactly dual to each other `Grokking §4.1].
pub enum Coterm<'s, N: Negative> {
    /// Axiom rule: a resource is a coterm of its type.
    Axiom(Resource<'s, N>),
    /// Structural elimination form (per-connective).
    Elim(N::Elim<'s>),
    /// μ̃-binder: general continuation closure.
    ///
    /// `μ̃x.c` binds a variable `x` of type `N::Dual` (positive).
    /// The body receives the term that `x` stands for.
    MuTilde(Box<dyn FnOnce(Term<'s, N::Dual>) -> Command<'s> + 's>),
}

impl<'s, A: Positive> From<Resource<'s, A>> for Term<'s, A> {
    fn from(r: Resource<'s, A>) -> Self {
        Term::Axiom(r)
    }
}

impl<'s, N: Negative> From<Resource<'s, N>> for Coterm<'s, N> {
    fn from(r: Resource<'s, N>) -> Self {
        Coterm::Axiom(r)
    }
}

// =============================================================================
// 2.  Polarity traits
// =============================================================================

/// Positive types of linear classical L.
///
/// The positive connectives are: atoms `X`, unit `1`, tensor `A ⊗ B`,
/// plus `A ⊕ B`, and bang `!A`. Each has a De Morgan dual (a negative
/// type) and a canonical introduction form at scope `'s`.
///
/// The [`Positive::Dual`] associated type computes the De Morgan dual,
/// always in negation-normal form. The duality is involutive:
/// `<A as Positive>::Dual::Dual = A`.
///
/// See `MMM §7] for the linear call-by-push-value L-calculus, and
/// `Spiwack` for the polarized system L treatment.
pub trait Positive: Sized + 'static {
    /// The De Morgan dual — a negative type.
    type Dual: Negative<Dual = Self>;
    /// The concrete introduction form at scope `'s`.
    ///
    /// For atomic types this is `std::convert::Infallible` — atoms have no
    /// introduction form besides the axiom rule (variables).
    /// For composite types this is a product, sum, or unit value
    /// carrying the components of the introduction.
    type Intro<'s>;
}

/// Negative types of linear classical L.
///
/// The negative connectives are: dual atoms `X⊥`, unit `⊥`,
/// par `A ⅋ B`, with `A & B`, and whynot `?A`. Each has a De Morgan
/// dual (a positive type) and a canonical elimination form at scope `'s`.
///
/// The [`Negative::Dual`] associated type computes the De Morgan dual.
/// The duality is involutive: `<N as Negative>::Dual::Dual = N`.
///
/// Every elim type must implement [`Resolution`] — the elim-side
/// dispatch for axiom-elim interactions. See [`Resolution`] for
/// why this lives on the elim rather than on the positive type.
pub trait Negative: Sized + 'static {
    /// The De Morgan dual — a positive type.
    type Dual: Positive<Dual = Self>;
    /// The concrete elimination form at scope `'s`.
    ///
    /// Each elim type carries the body of a destructor: a closure
    /// that consumes the components introduced by the dual positive
    /// connective. The elim must implement [`Resolution`] so that
    /// `cut` can dispatch axiom-elim interactions elim-side.
    type Elim<'s>: Resolution<'s, PosType = Self::Dual>;
}

// =============================================================================
// 3.  Interaction (pair-relation trait)
// =============================================================================

/// The interaction rule for the positive connective `A` and its dual.
///
/// An interaction is a symmetric active-pair event: `A`'s intro form meets its
/// dual's elim form, and the β-rule for the connective fires. Both participants
/// are structurally active and contribute to the reduction; the event is
/// symmetric in the sense that neither side owns the logic — the rule is a
/// pair-relation.
///
/// The trait is placed on [`Positive`] for Rust's sake; it could equivalently be
/// placed on [`Negative`], since interaction is a property of the connective pair
/// rather than of either polarity alone [Spiwack, module `Reduction`].
///
/// Each positive connective implements exactly one interaction rule:
///
/// | Connective | Interaction behaviour |
/// |------------|-----------------------|
/// | `AtomP<X>` | Unreachable (`Infallible` intro) |
/// | `One` | Invoke the bot elim's body with no arguments |
/// | `Tensor<A,B>` | Destructure the pair, pass components to par body |
/// | `Plus<A,B>` | Branch on injection, pass injected term to matching with arm |
/// | `Bang<A>` | Pass the `BangIntro` producer to the whynot elim body |
pub trait Interaction: Positive
where
    Self::Dual: Negative,
{
    /// Perform the interaction: destructure the intro and invoke the elim's
    /// body with the resulting components. For atoms, unreachable
    /// (`match intro {}` on `Infallible`), since atoms have no intro form.
    fn interact<'s>(intro: Self::Intro<'s>, elim: <Self::Dual as Negative>::Elim<'s>) -> Command<'s>;
}

// =============================================================================
// 4.  Resolution at an elim (elim-side dispatch)
// =============================================================================

/// The resolution rule for an elim consuming a resource.
///
/// A resolution is an asymmetric absorption event: a bare variable
/// ([`Resource`]) meets an elim, and the elim resolves the variable
/// against its body. The elim's structural continuation is preserved;
/// only the variable is absorbed [Spiwack, module `Reduction`, rule `mu`].
///
/// For atoms, resolution is non-trivial: the elim body receives the resource
/// wrapped as [`Term::Axiom`]. For composites, resolution is blocked
/// — a composite destructor cannot destructure a bare variable — so the
/// result is [`Command::Normal`] (blocked, awaiting outer substitution).
///
/// The `PosType` associated type avoids a GAT projection problem:
/// writing `Resolution<'s, Plus<A::Dual, B::Dual>>` as a trait parameter
/// fails because the compiler cannot prove `Plus<A::Dual, B::Dual>: Positive`
/// from `A: Negative, B: Negative` in that position. Using an associated type
/// sidesteps the issue entirely.
pub trait Resolution<'s> {
    /// The positive type whose resources this elim can consume.
    type PosType: Positive;
    /// Resolve a resource into this elim's body.
    ///
    /// For atoms: invokes the body with the resource as a term.
    /// For composites: returns [`Command::Normal`] (blocked).
    fn resolve(self, resource: Resource<'s, Self::PosType>) -> Command<'s>;
}

// =============================================================================
// 5.  Atoms
// =============================================================================

/// Positive atom `X`.
///
/// An atomic positive type. Atoms have no structural introduction form
/// besides variables — `AtomP::Intro<'s>` is `std::convert::Infallible`. The only
/// way to introduce an atom is the axiom rule: a resource of type `X`
/// is trivially a term of type `X`.
pub struct AtomP<X>(PhantomData<X>);

/// Negative atom `X⊥` — the De Morgan dual of `AtomP<X>`.
///
/// The elimination form for `AtomN<X>` is [`AtomElim`], which carries a
/// body consuming a positive term of type `AtomP<X>`. This is the
/// atomic μ̃-reduction: `<x | μ̃y.c>` reduces to `c[x/y]` [Spiwack, `mu`].
pub struct AtomN<X>(PhantomData<X>);

impl<X: 'static> Positive for AtomP<X> {
    type Dual = AtomN<X>;
    type Intro<'s> = std::convert::Infallible;
}

impl<X: 'static> Negative for AtomN<X> {
    type Dual = AtomP<X>;
    type Elim<'s> = AtomElim<'s, X>;
}

/// Elimination form for `AtomN<X>`.
///
/// The body receives a [`Term<'s, AtomP<X>>`]. When cut against a
/// variable of type `AtomP<X>`, the variable is wrapped as
/// `Term::Axiom(var)` and passed to this body — this is the atomic
/// resolution rule.
pub struct AtomElim<'s, X: 'static> {
    /// Body consuming a positive term.
    pub body: Box<dyn FnOnce(Term<'s, AtomP<X>>) -> Command<'s> + 's>,
}

impl<X: 'static> Interaction for AtomP<X> {
    fn interact<'s>(intro: Self::Intro<'s>, _elim: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
        match intro {}
    }
}

impl<'s, X: 'static> Resolution<'s> for AtomElim<'s, X> {
    type PosType = AtomP<X>;
    fn resolve(self, resource: Resource<'s, AtomP<X>>) -> Command<'s> {
        (self.body)(Term::Axiom(resource))
    }
}

// =============================================================================
// 6.  Multiplicative units
// =============================================================================

/// Positive unit `1`.
///
/// The multiplicative unit of the positive side. Its introduction form
/// is the unit value `()`. The De Morgan dual is [`Bot`] (⊥).
///
/// Interaction at `One`/`Bot`: `cut((), μ̃().c)` reduces to `c`
/// `MMM §7, rule (R1)]; [Spiwack, `unit`].
pub struct One;

/// Negative unit `⊥` (bottom).
///
/// The multiplicative unit of the negative side. Its elimination form
/// is [`BotElim`], a zero-argument body. The De Morgan dual is [`One`].
///
/// Interaction at `One`/`Bot`: `cut((), μ̃().c)` reduces to `c`.
pub struct Bot;

impl Positive for One {
    type Dual = Bot;
    type Intro<'s> = ();
}

impl Negative for Bot {
    type Dual = One;
    type Elim<'s> = BotElim<'s>;
}

/// Elimination form for `Bot`.
///
/// A zero-argument body — the destructor for the unit type.
/// When cut against `unit()`, the body runs with no arguments.
pub struct BotElim<'s> {
    /// Body with no arguments.
    pub body: Box<dyn FnOnce() -> Command<'s> + 's>,
}

impl Interaction for One {
    fn interact<'s>(_intro: Self::Intro<'s>, elim: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
        (elim.body)()
    }
}

impl<'s> Resolution<'s> for BotElim<'s> {
    type PosType = One;
    fn resolve(self, _resource: Resource<'s, One>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 7.  Multiplicative connectives
// =============================================================================

/// Tensor `A ⊗ B` (both components positive).
///
/// The multiplicative conjunction of linear logic. Introduction forms
/// are pairs of terms `(Term<'s, A>, Term<'s, B>)`. The De Morgan dual
/// is `Par<A::Dual, B::Dual>`.
///
/// Interaction at `Tensor`/`Par`: `cut((v, w), μ̃(x ⅋ y).c)` reduces to
/// `c[v/x, w/y]` `MMM §7, rule (R⊗)]; [Spiwack, `pair`].
pub struct Tensor<A: Positive, B: Positive>(PhantomData<(A, B)>);

/// Par `A ⅋ B` (both components negative).
///
/// The multiplicative disjunction of linear logic. Its elimination form
/// is [`ParElim`], a body consuming two terms. The De Morgan dual is
/// `Tensor<A::Dual, B::Dual>`.
///
/// Interaction at `Tensor`/`Par`: `cut((v, w), μ̃(x ⅋ y).c)` reduces to
/// `c[v/x, w/y]`.
pub struct Par<A: Negative, B: Negative>(PhantomData<(A, B)>);

impl<A: Positive, B: Positive> Positive for Tensor<A, B>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    type Dual = Par<A::Dual, B::Dual>;
    type Intro<'s> = (Term<'s, A>, Term<'s, B>);
}

impl<A: Negative, B: Negative> Negative for Par<A, B>
where
    A::Dual: Positive,
    B::Dual: Positive,
{
    type Dual = Tensor<A::Dual, B::Dual>;
    type Elim<'s> = ParElim<'s, A::Dual, B::Dual>;
}

/// Elimination form for `Par<A, B>`.
///
/// The body receives the two components of the tensor pair that
/// triggered this reduction. This is the par destructor
/// `μ̃(x ⅋ y).c` — a μ̃-form specialized to pattern-matching on
/// tensor pairs `Grokking §4.1].
pub struct ParElim<'s, A: Positive, B: Positive> {
    /// Body consuming two terms.
    pub body: Box<dyn FnOnce(Term<'s, A>, Term<'s, B>) -> Command<'s> + 's>,
}

impl<A: Positive, B: Positive> Interaction for Tensor<A, B>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    fn interact<'s>(intro: Self::Intro<'s>, elim: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
        let (a, b) = intro;
        (elim.body)(a, b)
    }
}

impl<'s, A: Positive, B: Positive> Resolution<'s> for ParElim<'s, A, B>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    type PosType = Tensor<A, B>;
    fn resolve(self, _resource: Resource<'s, Tensor<A, B>>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 8.  Additive connectives
// =============================================================================

/// Positive sum `A ⊕ B` (choice made at introduction time).
///
/// The additive disjunction of linear logic. Introduction forms are
/// left or right injections ([`PlusIntro::Inl`] or [`PlusIntro::Inr`]).
/// The De Morgan dual is `With<A::Dual, B::Dual>`.
///
/// Interaction at `Plus`/`With`: `cut(inl(v), μ̃case(x ⇒ c₁, y ⇒ c₂))`
/// reduces to `c₁[v/x]`; similarly for `inr` and the right branch
/// `MMM §7]; [Spiwack, `iota1`/`iota2`].
pub struct Plus<A: Positive, B: Positive>(PhantomData<(A, B)>);

/// Negative with `A & B` (choice made at destruction time).
///
/// The additive conjunction of linear logic. Its elimination form is
/// [`WithElim`], carrying two continuations — one per injection.
/// The De Morgan dual is `Plus<A::Dual, B::Dual>`.
///
/// The with destructor `μ̃case(x ⇒ c₁, y ⇒ c₂)` is a μ̃-form that
/// pattern-matches on plus injections, dispatching to the appropriate
/// arm `Grokking §4.1].
pub struct With<A: Negative, B: Negative>(PhantomData<(A, B)>);

/// Introduction form for `Plus<A, B>`.
///
/// Either left or right injection. The choice is made at construction
/// time, not at destruction time — this is the additive character.
pub enum PlusIntro<'s, A: Positive, B: Positive> {
    /// Left injection: `inl(v)`.
    Inl(Term<'s, A>),
    /// Right injection: `inr(w)`.
    Inr(Term<'s, B>),
}

impl<A: Positive, B: Positive> Positive for Plus<A, B>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    type Dual = With<A::Dual, B::Dual>;
    type Intro<'s> = PlusIntro<'s, A, B>;
}

impl<A: Negative, B: Negative> Negative for With<A, B>
where
    A::Dual: Positive,
    B::Dual: Positive,
{
    type Dual = Plus<A::Dual, B::Dual>;
    type Elim<'s> = WithElim<'s, A, B>;
}

/// Elimination form for `With<A, B>`.
///
/// Two continuations, one per injection. When cut against a `PlusIntro`,
/// the appropriate body is invoked with the injected term. This is the
/// case destructor `μ̃case(x ⇒ c₁, y ⇒ c₂)`.
pub struct WithElim<'s, A: Negative, B: Negative>
where
    A::Dual: Positive,
    B::Dual: Positive,
{
    /// Body for the left injection.
    pub left: Box<dyn FnOnce(Term<'s, A::Dual>) -> Command<'s> + 's>,
    /// Body for the right injection.
    pub right: Box<dyn FnOnce(Term<'s, B::Dual>) -> Command<'s> + 's>,
}

impl<A: Positive, B: Positive> Interaction for Plus<A, B>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    fn interact<'s>(intro: Self::Intro<'s>, elim: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
        match intro {
            PlusIntro::Inl(a) => (elim.left)(a),
            PlusIntro::Inr(b) => (elim.right)(b),
        }
    }
}

impl<'s, A: Negative, B: Negative> Resolution<'s> for WithElim<'s, A, B>
where
    A::Dual: Positive,
    B::Dual: Positive,
{
    type PosType = Plus<A::Dual, B::Dual>;
    fn resolve(self, _resource: Resource<'s, Plus<A::Dual, B::Dual>>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 9.  Exponential connectives
// =============================================================================

/// Positive exponential `!A` — duplicable terms.
///
/// The exponential modality of linear logic. Terms of type `!A` are
/// classical (duplicable) — they can be used zero, one, or many times.
/// The introduction form is [`BangIntro`], a producer closure that can
/// be cloned to yield fresh linear terms on demand.
///
/// The De Morgan dual is `Whynot<A::Dual>`.
///
/// Interaction at `Bang`/`Whynot`: `cut(!v, μ̃!x.c)` reduces to `c[!v/x]`
/// [Spiwack, `exponential`].
pub struct Bang<A: Positive>(PhantomData<A>);

/// Negative exponential `?A` — dual of `!A`.
///
/// The dual exponential modality. Its elimination form is
/// [`WhynotElim`], a body consuming a [`BangIntro`]. The De Morgan
/// dual is `Bang<N::Dual>`.
pub struct Whynot<N: Negative>(PhantomData<N>);

/// Introduction form for `Bang<A>`.
///
/// A duplicable producer of terms. The producer is an `Rc`-wrapped
/// closure that can be cloned any number of times, each invocation
/// yielding a fresh linear term. This enforces the structural rule
/// of exponentials at the value level: duplication is explicit and
/// controlled.
///
/// The producer must be closed (no free linear variables) — this is
/// enforced by Rust's move semantics on the closure.
pub struct BangIntro<'s, A: Positive> {
    pub(crate) producer: std::rc::Rc<dyn Fn() -> Term<'s, A> + 's>,
    pub(crate) _marker: PhantomData<&'s ()>,
}

impl<'s, A: Positive> Clone for BangIntro<'s, A> {
    fn clone(&self) -> Self {
        BangIntro {
            producer: self.producer.clone(),
            _marker: PhantomData,
        }
    }
}

impl<A: Positive> Positive for Bang<A>
where
    A::Dual: Negative,
{
    type Dual = Whynot<A::Dual>;
    type Intro<'s> = BangIntro<'s, A>;
}

impl<N: Negative> Negative for Whynot<N>
where
    N::Dual: Positive,
{
    type Dual = Bang<N::Dual>;
    type Elim<'s> = WhynotElim<'s, N>;
}

/// Elimination form for `Whynot<N>`.
///
/// The body receives a [`BangIntro`] — a duplicable producer of
/// terms. The body may clone the producer any number of times
/// (zero, one, many), or drop it without use. This is the
/// exponential destructor `μ̃!x.c`.
pub struct WhynotElim<'s, N: Negative> {
    /// Body consuming a `BangIntro`.
    pub body: Box<dyn FnOnce(BangIntro<'s, N::Dual>) -> Command<'s> + 's>,
}

impl<A: Positive> Interaction for Bang<A>
where
    A::Dual: Negative,
{
    fn interact<'s>(intro: Self::Intro<'s>, elim: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
        (elim.body)(intro)
    }
}

impl<'s, N: Negative> Resolution<'s> for WhynotElim<'s, N>
where
    N::Dual: Positive,
{
    type PosType = Bang<N::Dual>;
    fn resolve(self, _resource: Resource<'s, Bang<N::Dual>>) -> Command<'s> {
        Command::Normal
    }
}
