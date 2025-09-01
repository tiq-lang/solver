pub mod interner;
pub mod patterns;
pub mod primitives;

#[macro_export]
macro_rules! param_to_kind {
    ($ident:ident) => {
        $crate::primitives::GenericArgType::Type
    };
}

#[macro_export]
macro_rules! add_item {
    ($interner:expr, struct $name:ident< $($params:ident),* >) => {
        $interner.new_adt(
            stringify!($name).into(),
            [$($crate::param_to_kind!($params)),*].into()
        )
    };
    ($interner:expr, trait $name:ident< $($params:ident),* >) => {
        $interner.new_trait(
            stringify!($name).into(),
            [$($crate::param_to_kind!($params)),*].into()
        )
    };
}

#[macro_export]
macro_rules! add_items {
    ($interner:expr, { $( $kinds:ident $items:ident $(< $($params:ident),* >)?; )* }) => {
        (
            $(
                $crate::add_item!($interner, $kinds $items <$($( $params ),*)?> )
            ),*
        )
    };
}
