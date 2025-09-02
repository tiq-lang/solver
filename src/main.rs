use interner::NaiveInterner;
use solver_ir::add_items;
use solver_macros::impl_patterns;

mod interner;

#[allow(non_snake_case)]
fn main() {
    let interner = NaiveInterner::new();
    let (A, B, Clone) = add_items!(interner, {
        struct A;
        struct B<T>;
        trait Clone;
    });
    let (matcher, matched) = impl_patterns!(use crate solver_ir, &interner, {
        impl B<A> as Clone;
        impl B<_> as Clone;
    });
    // let mut impl_repr = String::new();
    // infer
    //     .format_as_trait_impl(&interner, Clone, &mut impl_repr)
    //     .unwrap();
    // println!("{}", impl_repr);
    println!("{}", matched.matches(&matcher));
}
