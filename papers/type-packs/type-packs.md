# Abstract

This paper covers different aspects of type packs language feature, elaborates on system design implications for this feature support and discusses possible alternative designs.

## Table of contents

1. [Motivation](#motivation)
1. [Feature aspects](#feature-aspects)
   1. [Declaring type packs](#declaring-type-packs)
   1. [Pack expansion and type patterns](#pack-expansion-and-type-patterns)
   1. [Constraining type packs](#constraining-type-packs)
      1. [Constraint patterns](#constraint-patterns)
      1. [Bound patterns](#bound-patterns)
   1. [Pack zipping](#pack-zipping)
   1. [Limitations](#limitations)
      1. [Expansion sites](#expansion-sites)
      1. [Patterns of expansion](#patterns-of-expansion)
      1. [Associated types and type packs](#associated-types-and-type-packs)
      1. [`Self`-agnostic traits and constraints](#self-agnostic-traits-and-constraints)
1. [Epilogue](#epilogue)
   1. [Proper implementations for tuple type](#proper-implementations-for-tuple-type)
   1. [Designer's note](#designers-note)

# Motivation

Generics are a powerful feature that allows programmers to express an intent of providing a variable compile-time interfaces for data types and functions. Currently our generics are extremely rigid. One can't express an intent of accepting varying number of generic parameters.

> Defaulted generic arguments are capable of imitating this functionality, but they are just a syntactic sugar to provide better user experience. From the type system point of view they do not actually exist as they are eliminated/substituted during name resolution.

This strictness is actually a problem since one can imagine that some core functionality (e.g. tuple type, `Fn` traits, etc.) can't be expressed without either variable number of type parameters or some amount of compiler magic that generates code on demand.

We are not the first to encounter this problem. Folkes familliar with Rust programming language might know that their core library provides [some implementations](https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations-1) only for tuples up to the fixed length.

So to fix these issues we want to support a new kind of generic argument that accept varying number of type parameters as its value. From now on we will call this feature type packs.

# Feature aspects

## Declaring type packs

As it was already said, type packs are a new kind of generic arguments, so the first thing to look at is how to accept a type pack as an argument. To do so we propose the following syntax:

```rust
struct Foo<[Ts]> { ... }
```

This syntax alludes to the fact that type packs are similar to the slice of types in some sense.

Furthermore, after proper adoption of const generics we can extend the syntax to allow the following:

```rust
struct Bar<[Ts; N], const N: usize> { ... }
```

This syntax allows one to infer/constrain type pack length, which can be useful when one wants to ensure that two packs have same length.

## Pack expansion and type patterns

After the pack was declared it must be used in some way. To use packs one can use the proposed expansion syntax:

```rust
impl<[Args]> FnOnce(Args..) for Foo { ... }
```

There are situations where one would want to modify elements of the type pack in some way. To do so we use type expansion patterns:

```rust
impl<[Ts]> ((&mut Ts)..) { ... }
```

Type patterns can be used in other places as well, which we will discuss in the following section.

## Constraining type packs

In this paper we do not consider the usage of packs within function bodies because it is not relevant for the solver, however we still need to be able to constrain type packs to enable this future use.

Firstly, we will consider the most primitive usage of type packs within `where` clauses:

```rust
trait Foo<[Ts]>
where
   Ts: Clone;
```

This example doesn't contain any new syntax within the `where` clause, but it brings new interpretation of certain bounds. If type argument of a bound is a type pack, the bound reads as "_for every type_ `T` _in a pack, bound_ `T: Trait` _holds_".

To increase consistency we also allow the following shorthand for these kinds of bounds:

```rust
trait Foo<[Ts]: Clone>;
```

Furthermore, we propose to allow type patterns to serve as type arguments of bounds:

```rust
trait Bar<[Ts]>
where
   (&mut Ts): Baz;
```

This bound reads pretty similar to the previous one: "_for every type_ `T` _in a pack, bound_ `(&mut T): Trait` holds".

In the following sections we will talk about new syntax introduced to bounds and constraits that allows one to express more complex relations with type packs.

### Constraint patterns

First syntax addition to `where` clauses targets constraints. We add a notion of _constraint patterns_, which are constraints that mention unexpanded type packs or type patterns. Similarly to them, constraint patterns can be expanded.

Let's consider the following example:

```rust
trait Foo<T, [Ts]>
where
   T: (PartialEq<Ts>)..;
```

If one were to instantiate `Foo` with `Ts = A, B` they would have to ensure that bound `T: PartialEq<A> + PartialEq<B>` holds.

In other words, constraint `(Trait<Ts>)..` has a reading "_for every type_ `T` _in a pack, bound_ `Self: Trait<T>` _must hold_", which is identical to the _"sum"_ of `Trait<T>` for `T` in `Ts`.

### Bound patterns

Another new syntax within `where` clauses is called _bound patterns_. Bound patterns, similar to constraint patterns, are bounds that mention unexpanded type packs or type patterns and just like them bound patterns can be expanded.

```rust
trait Foo<[Ts]>
where
   (Ts: Clone)..;
```

If one were to instantiate `Foo` with `Ts = A, B` they would have to proof bounds: `A: Clone, B: Clone`.

So, bound `(Ts: Trait)..` has a reading "_for every type_ `T` _in a pack, bound_ `T: Trait` _must hold_". If this sounds familiar, you would be correct. When `Trait` doesn't mention any type packs, this is precisely equal to `Ts: Trait` bound. For that reason bound patterns are useful only when `Trait` mentions some type pack, but we didn't yet talk about patterns that contain multiple packs.

## Pack zipping

Patterns are not restricted by the amount of type packs that appear within them, but when pattern mentions two or more type packs we perform _pack zipping_. When _"expanded"_, zipped packs do not yield all combinations of types inside packs, but rather _"share type indices"_ with each other.

This may sound complicated, so let's look at the example:

```rust
trait Foo<[Ts]>
where
   (Ts: PartialEq<Ts>)..;
```

Here we see a bound pattern that mentions `Ts` twice. If one were to instantiate `Foo` with `Ts = A, B` they would then need to proof bounds: `A: PartialEq<A>, B: PartialEq<B>`.

To avoid confusion when we zip type packs we require that their lengths compare equal, which means that until we support `[Ts; N]` functionality for declaring type packs one can zip type packs only with themselves.

## Limitations

### Expansion sites

Pack and pattern expansions are not allowed everywhere because we must ensure that whoever happens to recieve the expanded pack must be prepared for it. We call such places expansion sites.

Currently `where` clauses are the only bound pattern expansion site.

Similarly, bound constraints are the only constraint pattern expansion site as of this proposal. In the future we might consider making `dyn` constraints expansion site as well.

Type pattern expansion sites are:

1. Arguments of the built-in tuple type.
1. Generic arguments, only if the type pack is expected on the receiving side.

### Patterns of expansion

Type pack pattern expansions can really badly interfere with type inference and unification if used arbitrarily. For that reason we restrict usage of pattern expansions only to the following scenarios:

```rust
// Marker trait to provide type pack accepting site
trait Foo<[Ts]>;

// Ordinary pack expansion
impl<[Ts]> Foo<Ts..> for ();

// Pack expansion preceded by fixed type arguments
impl<T, U, [Ts]> Foo<T, U, Ts..> for ();

// Bad: pack expansion can't be followed by other arguments
impl<[Ts], T, [Us]> Foo<Ts.., T, Us..> for ();
```

By disallowing uses that do not follow the pattern `T_1, ..., T_n, Ts..` we guarantee that we can unambiguously infer all type parameters, do not cause any problems with unification and do not lose ability to ease these restrictions later.

### Interactions with associated types

...

### `Self`-agnostic traits and constraints

When we use type pattern as a type argument of a bound outside of a bound pattern we may encounter ambiguity if constraints mention `Self` in certain ways. Consider following examples:

```rust
trait Foo<[Ts]>
where
   Ts: PartialEq;

trait Bar<[Ts]>
where
   Ts: Eq;
```

In the `Foo` case bound `Ts: PartialEq` is synonimous to `Ts: PartialEq<Ts>` bound. And one can see that it has two possible interpretations: `(Ts: PartialEq<Ts>)..` and `Ts: (PartialEq<Ts>)..`. Both interpretations are viable and neither is more obvious than another.

The `Bar` case is a little more tricky. On the surface there is nothing wrong with `Ts: Eq` bound, however if we look at `Eq` bounds we will find `Self: PartialEq` bound. It is ambiguous for the same reasons as the previous example.

To prevent such ambiguities we add a notion of `Self`-_agnostic_ traits and constraints.

Constraint is `Self`-agnostic if:

- After substitution of default arguments it doesn't mention `Self` within its generic arguments
- Trait itself is `Self`-agnostic

Trait is `Self`-agnostic if:

- All constraints of the trait are `Self`-agnostic

All constraints on type patterns outside of bound patterns must be `Self`-agnostic.

Also note that we do not currently consider associated types for determining if trait is `Self`-agnostic because traits with associated types can't be used to constrain type packs as of this proposal.

# Epilogue

## Proper implementations for tuple type

In this section we will show how this proposal allows us to express most of the implementations for built-in tuple type without any compiler magic.

Simplest implementations do not actually require any bound or constraint patterns and can be written as follows:

```rust
impl<[Ts]> Clone for (Ts..)
where
    Ts: Clone { ... }

impl<[Ts]> Debug for (Ts..)
where
    Ts: Debug { ... }

impl<[Ts]> Default for (Ts..)
where
    Ts: Default { ... }

impl<[Ts]> Hash for (Ts..)
where
    Ts: Hash { ... }
```

Comparisons for tuples require elements to be comparable with themselves, which is precisely expressed by bound patterns and pack zipping:

```rust
impl<[Ts]> PartialEq for (Ts..)
where
    (Ts: PartialEq<Ts>).. { ... }

impl<[Ts]> PartialOrd for (Ts..)
where
    (Ts: PartialOrd<Ts>).. { ... }
```

Because `Eq` and `Ord` traits transitively mention `Self` type within constraints we require them to be put inside bound patterns:

```rust
impl<[Ts]> Eq for (Ts..)
where
    (Ts: Eq)..;

impl<[Ts]> Ord for (Ts..)
where
    (Ts: Ord).. { ... }
```

One implementation that can't be expressed with this proposal is `From<[T; N]>` implementation. If we want to be able to write it, we will need const generics and _"repeated type expansion patterns"_ feature:

```rust
impl<T, const N: usize> From<[T; N]> for ([T; N]..) { ... }
```

## Pack pinning

...

Pack _"zipping"_ is pretty universal, but sometimes it might not be desired. To allow one to _"zip"_ only some packs within expansion we provide a way to opt-out of it using pack _"pinning"_ via the syntax #5. Pack _"pinning"_ is available only for constraint and bound _"expansions"_.

## Designer's note:

Sentence _"Type packs can really badly interfere with type inference and unification if used arbitrarily"_ that was used in the beginning of the [Pack expansion limitations](#pack-expansion-limitations) section don't really have strong argumentation behind it.
Early in the development of this feature I discarded the idea of arbitrary mixing of pack expansions and fixed parameters because I couldn't formulate the procedure to infer types for an implementation with such mixed parameters.

After that I decided to stick to the expansions that followed the pattern `T_1, ..., T_n, Ts_1.., ..., Ts_m..`. This pattern is more permissive than one proposed in the final paper. It allows for an expansion of arbitrary amount of type packs after fixed arguments.

This pattern still had issues with type inference, so I improvised a rule, where each type pack must have been used otherwise in a pattern `T_1, ..., T_n, Ts..`. I believe this suffices for the type inference, but not for the unification. Lets look at following example:

```rust
trait Foo<A, B, C> { ... }

impl<[Ts], [Us]> Foo<(Ts..), (Us..), (Ts.., Us..)> for () { ... }

impl<[Ts], [Us]> Foo<(Ts..), (Us..), (Us.., Ts..)> for () { ... }
```

When we check these two implementations for overlap we end up with the following unification problem: `?[1].., ?[2].. = ?[2].., ?[1]..`. And... there are actually several solutions to this equation:

```
1: ?[1] = <>, ?[2] = X..
2: ?[1] = X.., ?[2] = <>
3: ?[1] = ?[2] = X..
// And perhaps others that I couldn't find
```

For this reason I decided to stick to the pattern where such equations can't show up.

Maybe this issue is not that awful and we can teach the compiler how to solve these equations, but this brings a lot of complexity into the compiler and to the peoples' experiences, because being unable to unify types with your eyes (which I can't do in this case) is really bad in my opinion.
