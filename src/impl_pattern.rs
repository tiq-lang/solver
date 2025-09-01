use solver_ir::{
    interner::{Interner, TraitId},
    primitives::{GenericArgType, PtrQual, RefQual, TypeId},
};
use std::{iter::FusedIterator, ops::Deref};

/// TODO: write docs
#[derive(Debug, PartialEq)]
pub enum PatternKind {
    Type,
}

impl PartialEq<GenericArgType> for PatternKind {
    fn eq(&self, other: &GenericArgType) -> bool {
        *self == PatternKind::from(*other)
    }
}

impl From<GenericArgType> for PatternKind {
    fn from(value: GenericArgType) -> Self {
        match value {
            GenericArgType::Type => PatternKind::Type,
        }
    }
}

/// TODO: write docs
#[derive(Clone, Copy, Debug)]
pub enum PatternElement<I: Interner> {
    /// Representation of concrete types (e.g. built-ins, structs, enums, etc.)
    TypeConstructor {
        args_length: usize,
        type_id: TypeId<I>,
    },
    /// Representation of opaque types (e.g. generics, opaque aliases, etc.)
    TypePlaceholder,
    /// Representation of yet unknown types (i.e. inference variables)
    InferredType,
}

impl<I: Interner> PatternElement<I> {
    /// Returns whether given `PatternElement` represents an entity that needs to be inferred.
    pub fn is_inference_var(&self) -> bool {
        match self {
            PatternElement::TypeConstructor { .. } | PatternElement::TypePlaceholder => false,
            PatternElement::InferredType => true,
        }
    }

    /// TODO: write docs
    pub fn kind(&self) -> PatternKind {
        match self {
            PatternElement::TypeConstructor { .. }
            | PatternElement::TypePlaceholder
            | PatternElement::InferredType => PatternKind::Type,
        }
    }
}

/// TODO: write docs
#[derive(Debug)]
#[repr(transparent)]
pub struct PatternSeq<I: Interner>([PatternElement<I>]);

impl<I: Interner> Deref for PatternSeq<I> {
    type Target = [PatternElement<I>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Iterator over `PatternSeq` that yields individual `Pattern`s.
#[derive(Clone, Copy, Debug)]
pub struct PatternSeqIter<'a, I: Interner>(Option<&'a PatternSeq<I>>);

impl<'a, I: Interner> Iterator for PatternSeqIter<'a, I> {
    type Item = &'a Pattern<I>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(pat) = self.0 else {
            return None;
        };
        let (head, tail) = pat.split_first();
        self.0 = tail;
        Some(head)
    }
}

impl<'a, I: Interner> FusedIterator for PatternSeqIter<'a, I> {}

impl<'a, I: Interner> IntoIterator for &'a PatternSeq<I> {
    type IntoIter = PatternSeqIter<'a, I>;
    type Item = &'a Pattern<I>;

    fn into_iter(self) -> Self::IntoIter {
        PatternSeqIter(Some(self))
    }
}

/// TODO: write docs
#[derive(Debug)]
#[repr(transparent)]
pub struct ExactPatternSeq<I: Interner>(PatternSeq<I>);

impl<I: Interner> Deref for ExactPatternSeq<I> {
    type Target = PatternSeq<I>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Iterator over `ExactPatternSeq` that yields individual `ExactPattern`s.
#[derive(Clone, Copy, Debug)]
pub struct ExactPatternSeqIter<'a, I: Interner>(Option<&'a ExactPatternSeq<I>>);

impl<'a, I: Interner> Iterator for ExactPatternSeqIter<'a, I> {
    type Item = &'a ExactPattern<I>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(pat) = self.0 else {
            return None;
        };
        let (head, tail) = pat.split_first();
        self.0 = tail;
        Some(head)
    }
}

impl<'a, I: Interner> FusedIterator for ExactPatternSeqIter<'a, I> {}

impl<'a, I: Interner> IntoIterator for &'a ExactPatternSeq<I> {
    type IntoIter = ExactPatternSeqIter<'a, I>;
    type Item = &'a ExactPattern<I>;

    fn into_iter(self) -> Self::IntoIter {
        ExactPatternSeqIter(Some(self))
    }
}

impl<I: Interner> PatternSeq<I> {
    /// Creates new `PatternSeq` from provided slice without any checks.
    ///
    /// # Safety
    /// `pattern` must be a valid `PatternSeq`.
    pub unsafe fn new_unchecked(pattern: &[PatternElement<I>]) -> &Self {
        let pat = pattern as *const _ as *const Self;
        // Safe because `pat` was obtained from valid reference
        unsafe { &*pat }
    }

    /// Creates new `PatternSeq` by checking that provided slice forms a sequence of valid
    /// `Pattern`s.
    pub fn new(interner: I, pattern: &[PatternElement<I>]) -> Option<&Self> {
        if pattern.is_empty() {
            return None;
        }
        let mut pat = pattern;
        while !pat.is_empty() {
            pat = Pattern::new_any(interner, pat)?.1;
        }
        // Safe because we just checked that `pattern` is valid `PatternSeq`
        Some(unsafe { Self::new_unchecked(pattern) })
    }

    /// TODO: write docs
    pub fn new_trait_impl(
        interner: I,
        pattern: &[PatternElement<I>],
        trait_id: TraitId<I>,
    ) -> Option<&Self> {
        let seq = Self::new(interner, pattern)?;
        let trait_data = interner.get_trait_by_id(trait_id);
        let generic_args = &*interner.trait_data(&trait_data).generic_args;
        if !seq.has_same_structure_as(
            std::iter::once(GenericArgType::Type).chain(generic_args.iter().copied()),
        ) {
            None
        } else {
            Some(seq)
        }
    }

    /// TODO: write docs
    pub fn boxed(&self) -> Box<Self> {
        let pat = Box::into_raw(Box::<[_]>::from(&self[..])) as *mut Self;
        // Safe because `pat` was obtained from `Box::into_raw`
        unsafe { Box::from_raw(pat) }
    }

    /// TODO: write docs
    pub fn split_first(&self) -> (&Pattern<I>, Option<&Self>) {
        let first_len = match self.get(0).expect("`PatternSeq` should be non-empty") {
            PatternElement::TypeConstructor { args_length, .. } => 1 + args_length,
            PatternElement::TypePlaceholder | PatternElement::InferredType => 1,
        };
        (
            // Safe because ...
            unsafe { Pattern::new_unchecked(&self[0..first_len]) },
            if first_len != self.len() {
                // Safe because ...
                Some(unsafe { Self::new_unchecked(&self[first_len..]) })
            } else {
                None
            },
        )
    }

    /// TODO: write docs
    pub fn has_same_structure_as<J>(&self, iter: J) -> bool
    where
        J: IntoIterator,
        J::Item: Into<PatternKind>,
    {
        let mut self_iter = self.into_iter();
        let mut kinds_iter = iter.into_iter();
        loop {
            match (self_iter.next(), kinds_iter.next()) {
                (None, None) => break true,
                (Some(_), None) | (None, Some(_)) => break false,
                (Some(pat), Some(kind)) => {
                    if pat.kind() != kind.into() {
                        break false;
                    }
                }
            }
        }
    }

    /// TODO: write docs
    pub fn matches(&self, pattern: &ExactPatternSeq<I>) -> bool {
        let (mut head, mut maybe_tail) = self.split_first();
        let (mut head_pat, mut maybe_tail_pat) = pattern.split_first();
        loop {
            if !head.matches(head_pat) {
                break false;
            }
            match (maybe_tail, maybe_tail_pat) {
                (Some(tail), Some(tail_pat)) => {
                    (head, maybe_tail) = tail.split_first();
                    (head_pat, maybe_tail_pat) = tail_pat.split_first();
                }
                (None, None) => break true,
                (Some(_), None) | (None, Some(_)) => break false,
            }
        }
    }

    /// TODO: write docs
    pub fn format(&self, interner: I, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let (mut head, mut maybe_tail) = self.split_first();
        head.format(interner, f)?;
        while let Some(tail) = maybe_tail {
            write!(f, ", ")?;
            (head, maybe_tail) = tail.split_first();
            head.format(interner, f)?;
        }
        Ok(())
    }

    /// TODO: write docs
    ///
    /// # Panics
    /// This panics if `self` doesn't form a valid pattern for inherent impl.
    pub fn format_as_inherent_impl(
        &self,
        interner: I,
        f: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        write!(f, "impl ")?;
        let (implementor, tail) = self.split_first();
        if implementor.kind() != PatternKind::Type {
            panic!(
                "implementor of an inherent impl must be a type\nimplementor: {:?}",
                implementor
            );
        }
        if let Some(_) = tail {
            panic!(
                "pattern is too long for an inherent impl\npattern: {:?}",
                self
            );
        }
        implementor.format(interner, f)
    }

    /// TODO: write docs
    ///
    /// # Panics
    /// This panics if `self` doesn't form a valid pattern for trait impl for provided `trait_id`.
    pub fn format_as_trait_impl(
        &self,
        interner: I,
        trait_id: TraitId<I>,
        f: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        write!(f, "impl ")?;
        let (implementor, trait_args) = self.split_first();
        if implementor.kind() != PatternKind::Type {
            panic!(
                "implementor of a trait impl must be a type\nimplementor: {:?}",
                implementor
            );
        }
        implementor.format(interner, f)?;
        write!(f, " as ")?;
        let trait_data = interner.get_trait_by_id(trait_id);
        let (trait_name, trait_generics) = {
            let trait_data = interner.trait_data(&trait_data);
            (&*trait_data.name, &*trait_data.generic_args)
        };
        write!(f, "{}", trait_name)?;
        if trait_generics.len() != 0 {
            let Some(trait_args) = trait_args else {
                panic!(
                    "trait `{}` expected generic arguments, but none were provided\n\
                     trait args: {:?}",
                    trait_name, trait_generics
                );
            };
            if !trait_args.has_same_structure_as(trait_generics.iter().copied()) {
                panic!(
                    "invalid generic arguments were provided for trait `{}`\n\
                     trait args: {:?}\nprovided args: {:?}",
                    trait_name, trait_generics, trait_args
                );
            }
            write!(f, "<")?;
            trait_args.format(interner, f)?;
            write!(f, ">")?;
        } else if let Some(trait_args) = trait_args {
            panic!(
                "trait `{}` didn't expect generic arguments, but some were provided\n\
                 provided args: {:?}",
                trait_name, trait_args
            );
        }
        Ok(())
    }
}

impl<I: Interner> ExactPatternSeq<I> {
    /// Creates new `ExactPatternSeq` from provided slice without any checks.
    ///
    /// # Safety
    /// `pattern` must be a valid `ExactPatternSeq`.
    pub unsafe fn new_unchecked(pattern: &[PatternElement<I>]) -> &Self {
        let pat = pattern as *const _ as *const Self;
        // Safe because `pat` was obtained from valid reference
        unsafe { &*pat }
    }

    /// Creates new `ExactPatternSeq` by checking that provided `PatternSeq` doesn't contain any
    /// inference variables.
    pub fn new(seq: &PatternSeq<I>) -> Option<&Self> {
        if seq.iter().any(|elem| elem.is_inference_var()) {
            return None;
        }
        // Safe because we just checked that `seq` is a valid `ExactPatternSeq`
        Some(unsafe { Self::new_unchecked(seq) })
    }

    /// TODO: write docs
    pub fn boxed(&self) -> Box<Self> {
        let pat = Box::into_raw(Box::<[_]>::from(&self[..])) as *mut Self;
        // Safe because `pat` was obtained from `Box::into_raw`
        unsafe { Box::from_raw(pat) }
    }

    /// TODO: write docs
    pub fn split_first(&self) -> (&ExactPattern<I>, Option<&Self>) {
        let (head, tail) = (**self).split_first();
        (
            // Safe because subparts of `self` are valid `ExactPattern`s
            unsafe { ExactPattern::new_unchecked(head) },
            tail.map(|tail| unsafe { Self::new_unchecked(tail) }),
        )
    }

    /// TODO: write docs
    pub fn disjoint_with(&self, other: &Self) -> bool {
        let (mut self_head, mut maybe_self_tail) = self.split_first();
        let (mut other_head, mut maybe_other_tail) = other.split_first();
        loop {
            if self_head.disjoint_with(other_head) {
                break true;
            }
            match (maybe_self_tail, maybe_other_tail) {
                (Some(self_tail), Some(other_tail)) => {
                    (self_head, maybe_self_tail) = self_tail.split_first();
                    (other_head, maybe_other_tail) = other_tail.split_first();
                }
                (None, None) => break false,
                (Some(_), None) | (None, Some(_)) => break true,
            }
        }
    }
}

/// TODO: write docs
#[derive(Debug)]
#[repr(transparent)]
pub struct Pattern<I: Interner>(PatternSeq<I>);

impl<I: Interner> Deref for Pattern<I> {
    type Target = PatternSeq<I>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// TODO: write docs
#[derive(Debug)]
#[repr(transparent)]
pub struct ExactPattern<I: Interner>(Pattern<I>);

impl<I: Interner> Deref for ExactPattern<I> {
    type Target = Pattern<I>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Interner> Pattern<I> {
    /// Creates new `Pattern` from provided elements slice without any checks.
    ///
    /// # Safety
    /// `pattern` must be a valid `Pattern`.
    pub unsafe fn new_unchecked(pattern: &[PatternElement<I>]) -> &Self {
        let pat = pattern as *const _ as *const Self;
        // Safe because `pat` was obtained from valid reference
        unsafe { &*pat }
    }

    /// TODO: write docs
    pub fn new_any(
        interner: I,
        pattern: &[PatternElement<I>],
    ) -> Option<(&Self, &[PatternElement<I>])> {
        match pattern.first()? {
            PatternElement::TypeConstructor { .. }
            | PatternElement::TypePlaceholder
            | PatternElement::InferredType => Self::new_type(interner, pattern),
        }
    }

    /// TODO: write docs
    pub fn new_of_kind(
        interner: I,
        pattern: &[PatternElement<I>],
        kind: PatternKind,
    ) -> Option<(&Self, &[PatternElement<I>])> {
        match kind {
            PatternKind::Type => Self::new_type(interner, pattern),
        }
    }

    /// TODO: write docs
    pub fn new_type(
        interner: I,
        pattern: &[PatternElement<I>],
    ) -> Option<(&Self, &[PatternElement<I>])> {
        let pat_len = match pattern.first()? {
            &PatternElement::TypeConstructor {
                args_length,
                type_id: ty,
            } => {
                let adt_data: I::InternedAdtData;
                let generic_args = match ty.generic_arg_types() {
                    Ok(args) => args,
                    Err(adt_id) => {
                        adt_data = interner.get_adt_by_id(adt_id);
                        &interner.adt_data(&adt_data).generic_args
                    }
                };
                let mut args_pat = pattern.get(1..args_length + 1)?;
                for &arg in generic_args {
                    args_pat = Self::new_of_kind(interner, args_pat, arg.into())?.1;
                }
                args_length + 1
            }
            PatternElement::TypePlaceholder | PatternElement::InferredType => 1,
        };
        Some((
            // Safe because we just checked that `pattern[0..pat_len]` is valid `Pattern`
            unsafe { Self::new_unchecked(&pattern[0..pat_len]) },
            &pattern[pat_len..],
        ))
    }

    /// Returns the kind of provided `Pattern`.
    pub fn kind(&self) -> PatternKind {
        self.first().kind()
    }

    /// Returns the first element of underlying slice of a `Pattern`.
    ///
    /// This is useful because `<[T]>::first` returns `Option<T>`, but `Pattern` is guaranteed to be
    /// non-empty.
    pub fn first(&self) -> &PatternElement<I> {
        self.0.first().expect("`Pattern` shouldn't be empty")
    }

    /// TODO: write docs
    pub fn args(&self) -> Option<&PatternSeq<I>> {
        if self.len() != 1 {
            // NOTE: this safety will need adjustments after introduction of const patterns
            // Safety:
            //   * `self` can't be a trivial pattern because `self.len() > 1`
            //   * therefore `self` must be a type constructor with non-empty arguments
            Some(unsafe { PatternSeq::new_unchecked(&self[1..]) })
        } else {
            None
        }
    }

    /// TODO: write docs
    pub fn matches(&self, pattern: &ExactPattern<I>) -> bool {
        match (self.first(), pattern.first()) {
            (
                PatternElement::TypeConstructor {
                    type_id: first_ty, ..
                },
                PatternElement::TypeConstructor {
                    type_id: second_ty, ..
                },
            ) => {
                if first_ty != second_ty {
                    return false;
                }
                match (self.args(), pattern.args()) {
                    (Some(args), Some(args_pat)) => args.matches(args_pat),
                    (None, None) => true,
                    (Some(_), None) | (None, Some(_)) => unreachable!(),
                }
            }
            (PatternElement::TypeConstructor { .. }, PatternElement::TypePlaceholder)
            | (PatternElement::TypePlaceholder, PatternElement::TypePlaceholder) => true,
            (PatternElement::TypePlaceholder, PatternElement::TypeConstructor { .. }) => false,
            (PatternElement::InferredType, pat) => pat.kind() == PatternKind::Type,
            (_, PatternElement::InferredType) => unreachable!(),
        }
    }

    /// TODO: write docs
    pub fn format(&self, interner: I, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        match self.first() {
            PatternElement::TypePlaceholder => write!(f, "_"),
            PatternElement::InferredType => write!(f, "?"),
            PatternElement::TypeConstructor {
                type_id: TypeId::Scalar(ty),
                ..
            } => write!(f, "{}", ty),
            PatternElement::TypeConstructor {
                type_id: TypeId::Never,
                ..
            } => write!(f, "!"),
            &PatternElement::TypeConstructor {
                type_id: TypeId::NonZero,
                ..
            } => {
                // Safe because `NonZero` has single argument and `self` is a valid `Pattern`
                let arg = unsafe { Self::new_unchecked(&self[1..]) };
                write!(f, "NonZero<")?;
                arg.format(interner, f)?;
                write!(f, ">")
            }
            &PatternElement::TypeConstructor {
                type_id: TypeId::Slice,
                ..
            } => {
                // Safe because `[T]` has single argument and `self` is a valid `Pattern`
                let arg = unsafe { Self::new_unchecked(&self[1..]) };
                write!(f, "[")?;
                arg.format(interner, f)?;
                write!(f, "]")
            }
            &PatternElement::TypeConstructor {
                type_id: TypeId::Ref(qual),
                ..
            } => {
                // Safe because `&` has single argument and `self` is a valid `Pattern`
                let arg = unsafe { Self::new_unchecked(&self[1..]) };
                write!(f, "&{}", RefQual::repr(qual))?;
                arg.format(interner, f)
            }
            &PatternElement::TypeConstructor {
                type_id: TypeId::Ptr(qual),
                ..
            } => {
                // Safe because `*` has single argument and `self` is a valid `Pattern`
                let arg = unsafe { Self::new_unchecked(&self[1..]) };
                write!(f, "*{}", PtrQual::repr(qual))?;
                arg.format(interner, f)
            }
            &PatternElement::TypeConstructor {
                type_id: TypeId::Adt(adt_id),
                args_length,
            } => {
                let adt_data = interner.get_adt_by_id(adt_id);
                write!(f, "{}", interner.adt_data(&adt_data).name)?;
                if args_length != 0 {
                    write!(f, "<")?;
                    // Safe because type `Pattern` constructor arguments form a valid `PatternSeq`
                    let args = unsafe { PatternSeq::new_unchecked(&self[1..]) };
                    args.format(interner, f)?;
                    write!(f, ">")?;
                }
                Ok(())
            }
        }
    }
}

impl<I: Interner> ExactPattern<I> {
    /// Creates new `ExactPattern` from `Pattern` without any checks.
    ///
    /// # Safety
    /// `pattern` must be a valid `ExactPattern`.
    pub unsafe fn new_unchecked(pattern: &Pattern<I>) -> &Self {
        let pat = pattern as *const _ as *const Self;
        // Safe because `pat` was obtained from valid reference
        unsafe { &*pat }
    }

    /// Creates new `ExactPattern` by checking that provided `Pattern` doesn't contain any inference
    /// variables.
    pub fn new(pattern: &Pattern<I>) -> Option<&Self> {
        if pattern.iter().any(|elem| elem.is_inference_var()) {
            return None;
        }
        // Safe because we just checked that `pattern` is a valid `ExactPattern`
        Some(unsafe { Self::new_unchecked(pattern) })
    }

    /// TODO: write docs
    pub fn args(&self) -> Option<&ExactPatternSeq<I>> {
        if self.len() != 1 {
            // NOTE: this safety will need adjustments after introduction of const patterns
            // Safety:
            //   * any subpattern of `self` is `ExactPatternSeq`, because `self` is `ExactPattern`
            //   * `self` can't be a trivial pattern because `self.len() > 1`
            //   * therefore `self` must be a type constructor with non-empty arguments
            Some(unsafe { ExactPatternSeq::new_unchecked(&self[1..]) })
        } else {
            None
        }
    }

    /// TODO: write docs
    pub fn disjoint_with(&self, other: &Self) -> bool {
        match (self.first(), other.first()) {
            (
                PatternElement::TypeConstructor {
                    type_id: self_ty, ..
                },
                PatternElement::TypeConstructor {
                    type_id: other_ty, ..
                },
            ) => {
                if self_ty != other_ty {
                    return true;
                }
                match (self.args(), other.args()) {
                    (Some(self_args), Some(other_args)) => self_args.disjoint_with(other_args),
                    (None, None) => false,
                    (Some(_), None) | (None, Some(_)) => unreachable!(),
                }
            }
            (PatternElement::TypeConstructor { .. }, PatternElement::TypePlaceholder)
            | (PatternElement::TypePlaceholder, PatternElement::TypeConstructor { .. })
            | (PatternElement::TypePlaceholder, PatternElement::TypePlaceholder) => false,
            (PatternElement::InferredType, _) | (_, PatternElement::InferredType) => unreachable!(),
        }
    }
}
