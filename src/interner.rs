use solver_ir::{
    interner::{AdtId, GenericArg, Interner, ItemId, TraitId},
    primitives::{AdtData, GenericArgData, GenericArgType, TraitData, TypeData},
};
use std::{cell::UnsafeCell, num::NonZero};

#[derive(Debug)]
enum InternerItem {
    Adt(AdtData),
    Trait(TraitData),
}

#[derive(Debug, Default)]
pub struct NaiveInterner {
    items: UnsafeCell<Vec<Box<InternerItem>>>,
}

impl NaiveInterner {
    pub fn new() -> Self {
        Self::default()
    }

    fn into_items_index(id: ItemId) -> usize {
        id.get() as usize - 1
    }

    fn new_item_id(index: usize) -> ItemId {
        // Safe because `1 + x` can't be zero without overflow
        unsafe {
            ItemId(NonZero::new_unchecked(
                index.checked_add(1).unwrap().try_into().unwrap(),
            ))
        }
    }

    pub fn get_adt(&self, id: AdtId<&Self>) -> &AdtData {
        // Safe because we are single threaded and references to `self.items` do not live longer
        // than any of our methods (methods do not invoke each other)
        let items = unsafe { &*self.items.get() };
        match &*items[Self::into_items_index(*id)] {
            InternerItem::Adt(data) => data,
            _ => unreachable!(),
        }
    }

    pub fn new_adt(&self, name: Box<str>, generic_args: Box<[GenericArgType]>) -> AdtId<&Self> {
        // See `get_adt` for safety
        let items = unsafe { &mut *self.items.get() };
        let id = AdtId::new(Self::new_item_id(items.len()));
        items.push(Box::new(InternerItem::Adt(AdtData {
            name,
            generic_args,
        })));
        id
    }

    pub fn get_trait(&self, id: TraitId<&Self>) -> &TraitData {
        // See `get_adt` for safety
        let items = unsafe { &*self.items.get() };
        match &*items[Self::into_items_index(*id)] {
            InternerItem::Trait(data) => data,
            _ => unreachable!(),
        }
    }

    pub fn new_trait(&self, name: Box<str>, generic_args: Box<[GenericArgType]>) -> TraitId<&Self> {
        // See `get_adt` for safety
        let items = unsafe { &mut *self.items.get() };
        let id = TraitId::new(Self::new_item_id(items.len()));
        items.push(Box::new(InternerItem::Trait(TraitData {
            name,
            generic_args,
        })));
        id
    }
}

impl<'a> Interner for &'a NaiveInterner {
    type InternedType = Box<TypeData<Self>>;
    type InternedGenericArg = GenericArgData<Self>;
    type InternedSubstitution = Box<[GenericArg<Self>]>;
    type InternedAdtData = &'a AdtData;
    type InternedTraitData = &'a TraitData;

    fn type_data(self, ty: &Self::InternedType) -> &TypeData<Self> {
        &*ty
    }

    fn generic_arg_data(self, arg: &Self::InternedGenericArg) -> &GenericArgData<Self> {
        arg
    }

    fn substitution_data(self, subst: &Self::InternedSubstitution) -> &[GenericArg<Self>] {
        &*subst
    }

    fn adt_data(self, adt: &Self::InternedAdtData) -> &AdtData {
        adt
    }

    fn trait_data(self, r#trait: &Self::InternedTraitData) -> &TraitData {
        r#trait
    }

    fn get_adt_by_id(self, id: AdtId<Self>) -> Self::InternedAdtData {
        self.get_adt(id)
    }

    fn get_trait_by_id(self, id: TraitId<Self>) -> Self::InternedTraitData {
        self.get_trait(id)
    }
}
