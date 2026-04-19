use std::sync::Arc;

use crate::machine::Command;

use either::Either;

// =============================================================================
// 1.  Linear resource tokens
// =============================================================================

/// A linear resource token at positive type `A` in scope `'x`.
///
/// Under the resource-theoretic reading of linear logic, a formula is a
/// resource that gets consumed in proofs. A `Resource<'x, A>` is the
/// proof-theoretic token for such a resource: it carries the
/// [`Positive::Witness<'x>`] of `A`, the runtime evidence that the
/// resource exists in scope `'x`.
///
/// Non-`Copy`, non-`Clone` — using it twice is a compile error. Move
/// semantics enforce linearity at the type-system level. This is the
/// axiom rule in sequent-calculus terms: a resource of type `A` is
/// trivially a term of type `A`.
///
/// See `Grokking §2]` for an accessible introduction to the sequent-calculus
/// treatment of variables as first-class resources.
pub struct Resource<'x, A: Positive> {
    witness: A::Witness<'x>,
}

impl<'x, A: Positive> Resource<'x, A> {
    /// Create a resource token from a witness.
    ///
    /// In a real term, resources are introduced by binders; this
    /// constructor is useful for building open terms (e.g. the axiom
    /// rule) and for testing.
    pub fn new(witness: A::Witness<'x>) -> Self {
        Resource { witness }
    }

    pub fn into_witness(self) -> A::Witness<'x> {
        self.witness
    }
}

// Explicitly NOT implementing Clone or Copy.
// Move semantics enforce linearity.

/// A linear resource token at negative type `N` in scope `'x`.
///
/// The negative counterpart to [`Resource`]. A `CoResource<'x, N>`
/// carries the [`Negative::Witness<'x>`] of `N`.
///
/// Non-`Copy`, non-`Clone` — move semantics enforce linearity.
pub struct CoResource<'x, N: Negative> {
    witness: N::Witness<'x>,
}

impl<'x, N: Negative> CoResource<'x, N> {
    /// Create a co-resource token from a witness.
    pub fn new(witness: N::Witness<'x>) -> Self {
        CoResource { witness }
    }

    pub fn into_witness(self) -> N::Witness<'x> {
        self.witness
    }
}

// Explicitly NOT implementing Clone or Copy.

// =============================================================================
// 2.  Uniform term and coterm enums
// =============================================================================

/// A positive term at scope `'s`.
///
/// In the λμ̃μ-calculus, positive terms are the data side of the
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
///   object `Grokking §3.2]`.
///
/// The three-variant structure mirrors the classical sequent calculus:
/// variables (axiom), structural forms (intro/elim), and μ/μ̃-bound
/// continuations. See `MMM §7]` and `Spiwack` for the underlying calculus.
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
/// In the λμ̃μ-calculus, negative coterms are the codata side of the
/// duality. Three variants, symmetric with [`Term`] under De Morgan
/// duality:
///
/// * [`Coterm::Axiom`] — the axiom rule. A co-resource of type `N` is
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
///   `Grokking §3.2]`.
///
/// The symmetry between [`Term`] and [`Coterm`] is not superficial
/// sameness — it is the theory's duality made visible. Data and
/// codata are exactly dual to each other `Grokking §4.1]`.
pub enum Coterm<'s, N: Negative> {
    /// Axiom rule: a co-resource is a coterm of its type.
    Axiom(CoResource<'s, N>),
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

impl<'s, N: Negative> From<CoResource<'s, N>> for Coterm<'s, N> {
    fn from(r: CoResource<'s, N>) -> Self {
        Coterm::Axiom(r)
    }
}

// =============================================================================
// 3.  Polarity traits
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
/// Each positive connective implements [`interact`](Positive::interact) —
/// the structural β-rule that fires when an intro form meets the dual's
/// elim form.
///
/// See `MMM §7]` for the linear call-by-push-value L-calculus, and
/// `Spiwack` for the polarized system L treatment.
pub trait Positive: Sized + 'static {
    /// The De Morgan dual — a negative type.
    type Dual: Negative<Dual = Self>;
    /// The concrete introduction form at scope `'s`.
    ///
    /// For atomic types this is `std::convert::Infallible` — atoms have no
    /// structural introduction form besides the axiom rule (variables).
    /// For composite types this is a product, sum, or unit value
    /// carrying the components of the introduction.
    type Intro<'s>;
    /// The witness type in scope `'x`.
    ///
    /// A witness is the runtime content carried by a bare variable
    /// (axiom) of this type. For atoms it is the atom value itself;
    /// for composites it is a structural description of the value's
    /// shape. See the witness table in the crate documentation.
    type Witness<'x>;

    /// Perform the interaction: destructure the intro and invoke the elim's
    /// body with the resulting components.
    ///
    /// This is the structural β-rule for the connective. For atoms,
    /// unreachable (`match intro {}` on `Infallible`), since atoms have
    /// no intro form.
    fn interact<'s>(
        intro: Self::Intro<'s>,
        elim: <Self::Dual as Negative>::Elim<'s>,
    ) -> Command<'s>;
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
/// Each negative connective implements [`resolve`](Negative::resolve) —
/// the elim-side dispatch for axiom-elim interactions.
pub trait Negative: Sized + 'static {
    /// The De Morgan dual — a positive type.
    type Dual: Positive<Dual = Self>;
    /// The concrete elimination form at scope `'s`.
    ///
    /// Each elim type carries the body of a destructor: a closure
    /// that consumes the components introduced by the dual positive
    /// connective.
    type Elim<'s>;
    /// The witness type in scope `'x`.
    ///
    /// A witness is the runtime content carried by a bare covariable
    /// (axiom) of this type. See the witness table in the crate
    /// documentation.
    type Witness<'x>;

    /// Resolve a resource into this elim's body.
    ///
    /// For atoms: invokes the body with the resource's witness.
    /// For composites: returns [`Command::Normal`] (blocked).
    fn resolve<'s>(elim: Self::Elim<'s>, resource: Resource<'s, Self::Dual>) -> Command<'s>;
}

// =============================================================================
// 4.  Multiplicative units
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
/// is a zero-argument closure. The De Morgan dual is [`One`].
///
/// Interaction at `One`/`Bot`: `cut((), μ̃().c)` reduces to `c`.
pub struct Bot;

impl Positive for One {
    type Dual = Bot;
    type Intro<'s> = ();
    type Witness<'x> = ();

    fn interact<'s>(_: Self::Intro<'s>, elim: <Self::Dual as Negative>::Elim<'s>) -> Command<'s> {
        elim()
    }
}

impl Negative for Bot {
    type Dual = One;
    type Elim<'s> = Box<dyn FnOnce() -> Command<'s> + 's>;
    type Witness<'x> = ();

    fn resolve<'s>(_: Self::Elim<'s>, _: Resource<'s, Self::Dual>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 5.  Multiplicative connectives
// =============================================================================

/// Tensor `A ⊗ B` (both components positive).
///
/// The multiplicative conjunction of linear logic. Introduction forms
/// are pairs of terms `(Term<'s, A>, Term<'s, B>)`. The De Morgan dual
/// is `Par<A::Dual, B::Dual>`.
///
/// Interaction at `Tensor`/`Par`: `cut((v, w), μ̃(x ⅋ y).c)` reduces to
/// `c[v/x, w/y]` `MMM §7, rule (R⊗)]; [Spiwack, `pair`].
pub struct Tensor<A: Positive, B: Positive>(std::marker::PhantomData<(A, B)>);

/// Par `A ⅋ B` (both components negative).
///
/// The multiplicative disjunction of linear logic. Its elimination form
/// is a two-argument closure consuming two terms. The De Morgan dual is
/// `Tensor<A::Dual, B::Dual>`.
///
/// Interaction at `Tensor`/`Par`: `cut((v, w), μ̃(x ⅋ y).c)` reduces to
/// `c[v/x, w/y]`.
pub struct Par<A: Negative, B: Negative>(std::marker::PhantomData<(A, B)>);

impl<A: Positive, B: Positive> Positive for Tensor<A, B>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    type Dual = Par<A::Dual, B::Dual>;
    type Intro<'s> = (Term<'s, A>, Term<'s, B>);
    type Witness<'x> = (A::Witness<'x>, B::Witness<'x>);

    fn interact<'s>(
        intro: Self::Intro<'s>,
        elim: <Self::Dual as Negative>::Elim<'s>,
    ) -> Command<'s> {
        let (a, b) = intro;
        elim(a, b)
    }
}

impl<A: Negative, B: Negative> Negative for Par<A, B>
where
    A::Dual: Positive,
    B::Dual: Positive,
{
    type Dual = Tensor<A::Dual, B::Dual>;
    type Elim<'s> = Box<dyn FnOnce(Term<'s, A::Dual>, Term<'s, B::Dual>) -> Command<'s> + 's>;
    type Witness<'x> = Box<dyn FnOnce(A::Witness<'x>, B::Witness<'x>) + 'x>;

    fn resolve<'s>(_: Self::Elim<'s>, _: Resource<'s, Self::Dual>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 6.  Additive connectives
// =============================================================================

/// Positive sum `A ⊕ B` (choice made at introduction time).
///
/// The additive disjunction of linear logic. Introduction forms are
/// left or right injections ([`Either::Left`] or [`Either::Right`]).
/// The De Morgan dual is `With<A::Dual, B::Dual>`.
///
/// Interaction at `Plus`/`With`: `cut(inl(v), μ̃case(x ⇒ c₁, y ⇒ c₂))`
/// reduces to `c₁[v/x]`; similarly for `inr` and the right branch
/// `MMM §7]`; [Spiwack, `iota1`/`iota2`].
pub struct Plus<A: Positive, B: Positive>(std::marker::PhantomData<(A, B)>);

/// Negative with `A & B` (choice made at destruction time).
///
/// The additive conjunction of linear logic. Its elimination form is
/// a pair of continuations — one per injection.
/// The De Morgan dual is `Plus<A::Dual, B::Dual>`.
///
/// The with destructor `μ̃case(x ⇒ c₁, y ⇒ c₂)` is a μ̃-form that
/// pattern-matches on plus injections, dispatching to the appropriate
/// arm `Grokking §4.1]`.
pub struct With<A: Negative, B: Negative>(std::marker::PhantomData<(A, B)>);

impl<A: Positive, B: Positive> Positive for Plus<A, B>
where
    A::Dual: Negative,
    B::Dual: Negative,
{
    type Dual = With<A::Dual, B::Dual>;
    type Intro<'s> = Either<Term<'s, A>, Term<'s, B>>;
    type Witness<'x> = Either<A::Witness<'x>, B::Witness<'x>>;

    fn interact<'s>(
        intro: Self::Intro<'s>,
        elim: <Self::Dual as Negative>::Elim<'s>,
    ) -> Command<'s> {
        let (left, right) = elim;
        match intro {
            Either::Left(a) => left(a),
            Either::Right(b) => right(b),
        }
    }
}

impl<A: Negative, B: Negative> Negative for With<A, B>
where
    A::Dual: Positive,
    B::Dual: Positive,
{
    type Dual = Plus<A::Dual, B::Dual>;
    type Elim<'s> = (
        Box<dyn FnOnce(Term<'s, A::Dual>) -> Command<'s> + 's>,
        Box<dyn FnOnce(Term<'s, B::Dual>) -> Command<'s> + 's>,
    );
    type Witness<'x> = (A::Witness<'x>, B::Witness<'x>);

    fn resolve<'s>(_: Self::Elim<'s>, _: Resource<'s, Self::Dual>) -> Command<'s> {
        Command::Normal
    }
}

// =============================================================================
// 7.  Exponential connectives
// =============================================================================

/// Positive exponential `!A` — duplicable terms.
///
/// The exponential modality of linear logic. Terms of type `!A` are
/// classical (duplicable) — they can be used zero, one, or many times.
/// The introduction form is an `Arc`-wrapped witness that can be
/// cloned to yield fresh resources on demand.
///
/// The De Morgan dual is `Whynot<A::Dual>`.
///
/// Interaction at `Bang`/`Whynot`: `cut(!v, μ̃!x.c)` reduces to `c[!v/x]`
/// [Spiwack, `exponential`].
pub struct Bang<A: Positive>(std::marker::PhantomData<A>);

/// Negative exponential `?A` — dual of `!A`.
///
/// The dual exponential modality. Its elimination form is a closure
/// consuming an `Arc`-wrapped witness. The De Morgan dual is
/// `Bang<N::Dual>`.
pub struct Whynot<N: Negative>(std::marker::PhantomData<N>);

impl<A: Positive> Positive for Bang<A>
where
    A::Dual: Negative,
{
    type Dual = Whynot<A::Dual>;
    type Intro<'s> = Arc<A::Witness<'s>>;
    type Witness<'x> = Arc<A::Witness<'x>>;

    fn interact<'s>(
        intro: Self::Intro<'s>,
        elim: <Self::Dual as Negative>::Elim<'s>,
    ) -> Command<'s> {
        elim(intro)
    }
}

impl<N: Negative> Negative for Whynot<N>
where
    N::Dual: Positive,
{
    type Dual = Bang<N::Dual>;
    type Elim<'s> = Box<dyn FnOnce(Arc<<N::Dual as Positive>::Witness<'s>>) -> Command<'s> + 's>;
    type Witness<'x> = Box<dyn FnMut(N::Witness<'x>) + 'x>;

    fn resolve<'s>(_: Self::Elim<'s>, _: Resource<'s, Self::Dual>) -> Command<'s> {
        Command::Normal
    }
}
