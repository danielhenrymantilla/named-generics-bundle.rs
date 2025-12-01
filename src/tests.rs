/// ```rust
/// extern crate named_generics_bundle as renamed;
/// extern crate core as named_generics_bundle;
///
/// mod re_exported {
///     pub use ::renamed::tests::doctest_module_path::Demo;
/// }
///
/// type A = re_exported::Demo![X = i32];
/// ```
pub mod doctest_module_path {
    #[crate::named_generics_bundle(
        // what we are testing.
        path_to_this_very_module = crate::tests::doctest_module_path,
    )]
    pub trait Demo {
        type X: Copy;
    }
}

pub use doctest_module_path::Demo;

// No unit tests at the moment (not that great with proc-macros).
