extern crate named_generics_bundle as renamed;
extern crate core as named_generics_bundle;

#[::renamed::named_generics_bundle(path_to_named_generics_bundle_crate = ::renamed)]
/// Outer.
trait _MyBundle : 'static {
    //! Inner.
    type A: Iterator;
}

type _B = _MyBundle![A = ::core::iter::Empty<()>];
