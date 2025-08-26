# Abstract

This paper covers different aspects of type packs language feature, elaborates on system design implications for this feature support and discusses possible alternative designs.

## Table of contents

1. [Introduction](#introduction)
1. [Syntax](#syntax)
   1. [Declaring type packs](#declaring-type-packs)
   1. [Using the type pack](#using-the-type-pack)
   1. [Constraining type packs](#constraining-type-packs)
      1. [Bound expansion](#bound-expansion)
      1. [Pack zipping](#pack-zipping)
      1. [Pack pinning](#pack-pinning)
1. [Semantics](#semantics)
   1. [Pack expansion limitations](#pack-expansion-limitations)
1. [Epilogue](#epilogue)
   1. [Designer's note](#designers-note)

# Introduction

Generics are a powerful feature that allows programmers to express an intent of providing a variable compile-time interfaces for data types and functions. Currently our generics are extremely rigid. One can't express an intent of accepting varying number of generic parameters.

> Defaulted generic arguments are capable of imitating this functionality, but they are just a syntactic sugar to provide better user experience. From the type system point of view they do not actually exist as they are eliminated/substituted during name resolution.

This strictness is actually a problem since one can imagine that some core functionality (e.g. tuple type, `Fn` traits, etc.) can't be expressed without either variable number of type parameters or some amount of compiler magic that generates code on demand.

We are not the first to encounter this problem. Folkes familliar with Rust programming language might know that their core library provides [some implementations](https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations-1) only for tuples up to the fixed length.

So to fix these issues we want to support a new kind of generic argument that accept varying number of type parameters as its value. From now on we will call this feature type packs.

# Syntax

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

## Using the type pack

After the pack was declared it must be used in some way. To use packs one can use the proposed pack expansion syntax:

```rust
impl<[Args]> FnOnce(Args..) for Foo { ... }
```

Packs are not the only new entity that can be expanded. Type patterns can also serve as the argument for the expansion _"operator"_:

```rust
impl<[Ts]> ((&mut Ts)..) { ... }
```

In this example `(&mut Ts)` is called a type pattern. Type patterns can also be used within bound and constraint patterns which we will cover below.

## Constraining type packs

In this paper we do not consider the usage of packs within function bodies because it is not relevant for the solver, but we still must be able to constrain type packs to convey the message of _"under provided constraints I will be able to write an implementation for this trait"_.

We propose the following syntax for constraining the type packs:

```rust
// syntax #1
trait Foo<[Ts]: Clone>
where
    // syntax #2
    Ts: Copy,
    // syntax #3
    Ts: (PartialEq<Ts>)..,
    // syntax #4
    (Ts: PartialEq<Ts>)..,
    // syntax #5
    Ts: (Bar<Ts, Ts, ^Ts>)..;
```

Syntaxes #1 and #2 are not actually new, they are just an extension of a well established bound syntax to the type packs. Syntax #1 is a shorthand for a more verbose syntax #2, so lets consider only the second one. Bound `Ts: ...` where `Ts` is a type pack and `...` is some constraint reads as "_for every type_ `T` _in a pack bound_ `T: ...` _holds_".

Contrary, syntaxes #3, #4 and #5 are indeed new and have a very special role. They are used to express non-trivial relations between types and type packs. Currently we are willing to support only two kinds of such relations that are discussed in the following sections.

### Bound patterns

First kind of these relations is expressed via the syntax #3 in the example. Bound `Ts: (PartialEq<Ts>)..` there _"expands"_ to the `Ts: PartialEq<Ts_0> + ... + PartialEq<Ts_n>` bound. Under this constraint if one had a tuple of type `(Ts..)` they would be able to compare any two elements of this tuple for equality.

### Pack zipping

Sometimes we desire to express relations not between a type and a pack but between two packs. To do so we use syntax #4 from the previous example. When we encounter two or more packs inside the constraint or bound that is being _"expanded"_ we _"zip"_ these packs. When _"expanded"_, _"zipped"_ packs do not yield all combinations of types inside packs, but rather _"share type indices"_ with each other.

In the example above bound `(Ts: PartialEq<Ts>)..` expresses the intent of _"zipping"_ `Ts` with itself and results in the following _"expanded"_ bound: `Ts_0: PartialEq<Ts_0>, ..., Ts_n: PartialEq<Ts_n>`. Continuing with the example from previous section, under this constraint if one had a tuple of type `(Ts..)` they would be able to compare only elements of this tuple that have same index (that's why we previously said that two packs will _"share type indices"_).

To avoid confusion when we try to _"zip"_ two type packs we require that their lengths compare equal, which means that until we support `[Ts; N]` functionality to declare type packs one can _"zip"_ type packs only with themselves (which is still powerful).

### Pack pinning

Pack _"zipping"_ is pretty universal, but sometimes it might not be desired. To allow one to _"zip"_ only some packs within expansion we provide a way to opt-out of it using pack _"pinning"_ via the syntax #5. Pack _"pinning"_ is available only for constraint and bound _"expansions"_.

# Semantics

## Pack usage

We already discussed syntactic aspect of type packs, but, as always, semantics tend to restrict the syntax even more and this feature is not an exception. In this section we will discuss where and how do we allow usage of type packs.

From the proposed syntax one can find all the places where we allow type packs. We call these places type pack accepting sites. Type pack accepting sites are pack expansion sites, type, constraint and bound patterns, and type argument of bounds. Type pack usage outside of the accepting site is an error.

### Pack as a type argument of a bound

For the most part this case is trivial, however there is a little subtlety with regards to defaulted generic arguments on traits. If trait uses `Self` as the default value for some arguments (e.g. `PartialEq`) and these arguments are not overwritten within a bound this bound is ambiguous. Consider the following example:

```rust
impl<[Ts]> PartialEq for (Ts..)
where
    Ts: PartialEq,
{ ... }
```

There are two possible interpretations of the `Ts: PartialEq` bound: `(Ts: PartialEq<Ts>)..` and `Ts: (PartialEq<Ts>)..`. Both interpretations are possible and neither is more obvious than another. For this reason we force user to explicitly state their intent by writing either of these.

### Type, constraint and bound patterns

...

### Pack expansion sites

This case is more intresting than the previous one. Let's start by listing all pack expansion sites:

1. Arguments of a built-in tuple type
1. Generic arguments of data types/type aliases/traits, where there is a type pack expected on the receiving side

## Pack expansion limitations

Type packs can really badly interfere with type inference and unification if used arbitrarily. For that reason we restrict usage of pack expansions only to the following scenarios:

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

See [Designer's note](#designers-note) to read more about the origins of the proposed expansion pattern and issues encountered along the way to it.

# Epilogue

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
