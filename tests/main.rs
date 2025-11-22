#[::named_generics_bundle::named_generics_bundle]
/// Outer.
trait MyBundle : 'static {
    //! Inner.
    type A: Iterator;
    type B: ::core::fmt::Display;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, ::core::hash::Hash)]
struct S<P: MyBundle>(
    ::core::marker::PhantomData<P>,
);

fn demo<P: MyBundle>(
    s: S<P>,
    a: P::A,
    b: P::B,
) -> impl 'static + Copy + ::core::fmt::Debug + ::core::cmp::Ord + ::core::hash::Hash {
    eprintln!("{b}");
    for _ in a {}
    s
}

#[test]
fn main() {
    _ = demo::<MyBundle![
        A = ::core::iter::Empty<()>,
        B = String,
    ]>;
}
