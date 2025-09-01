//! TODO: write docs

use super::primitives::{AdtData, GenericArgData, TraitData, TypeData};
use std::{fmt::Debug, marker::PhantomData, num::NonZero, ops::Deref};

/// TODO: write docs
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemId(pub NonZero<u32>);

impl Deref for ItemId {
    type Target = NonZero<u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// TODO: write docs
#[derive(Clone, Copy, Debug)]
pub struct AdtId<I: Interner>(pub ItemId, PhantomData<I>);

impl<I: Interner> AdtId<I> {
    pub fn new(id: ItemId) -> Self {
        Self(id, PhantomData)
    }
}

impl<I: Interner> Deref for AdtId<I> {
    type Target = ItemId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Interner> PartialEq for AdtId<I> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// TODO: write docs
#[derive(Clone, Copy, Debug)]
pub struct TraitId<I: Interner>(pub ItemId, PhantomData<I>);

impl<I: Interner> TraitId<I> {
    pub fn new(id: ItemId) -> Self {
        Self(id, PhantomData)
    }
}

impl<I: Interner> Deref for TraitId<I> {
    type Target = ItemId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Interner> PartialEq for TraitId<I> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// TODO: write docs
pub trait Interner: Debug + Copy {
    /// TODO: write docs
    type InternedType: Debug;

    /// TODO: write docs
    type InternedGenericArg: Debug;

    /// TODO: write docs
    type InternedSubstitution: Debug;

    /// TODO: write docs
    type InternedAdtData: Debug;

    /// TODO: write docs
    type InternedTraitData: Debug;

    /// TODO: write docs
    fn type_data(self, ty: &Self::InternedType) -> &TypeData<Self>;

    /// TODO: write docs
    fn generic_arg_data(self, arg: &Self::InternedGenericArg) -> &GenericArgData<Self>;

    /// TODO: write docs
    fn substitution_data(self, subst: &Self::InternedSubstitution) -> &[GenericArg<Self>];

    /// TODO: write docs
    fn adt_data(self, adt: &Self::InternedAdtData) -> &AdtData;

    /// TODO: write docs
    fn trait_data(self, r#trait: &Self::InternedTraitData) -> &TraitData;

    /// TODO: write docs
    fn get_adt_by_id(self, id: AdtId<Self>) -> Self::InternedAdtData;

    /// TODO: write docs
    fn get_trait_by_id(self, id: TraitId<Self>) -> Self::InternedTraitData;
}

/// TODO: write docs
#[derive(Clone, Debug)]
pub struct Type<I: Interner>(I::InternedType);

impl<I: Interner> Type<I> {
    pub fn data(&self, interner: I) -> &TypeData<I> {
        interner.type_data(&self.0)
    }
}

/// TODO: write docs
#[derive(Clone, Debug)]
pub struct Substitution<I: Interner>(I::InternedSubstitution);

impl<I: Interner> Substitution<I> {
    pub fn data(&self, interner: I) -> &[GenericArg<I>] {
        interner.substitution_data(&self.0)
    }
}

/// TODO: write docs
#[derive(Clone, Debug)]
pub struct GenericArg<I: Interner>(I::InternedGenericArg);

impl<I: Interner> GenericArg<I> {
    pub fn data(&self, interner: I) -> &GenericArgData<I> {
        interner.generic_arg_data(&self.0)
    }
}
