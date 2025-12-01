//! [`named_generics_bundle`]: `named_generics_bundle`
#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![allow(unused_braces)]

/// Main macro, entrypoint to the features of the crate.
///
/// See the [main-level docs][`crate`] for more contextual information about this attribute.
///
/// # Usage
///
/// Basic:
///
/// ```rust
/// # use ::named_generics_bundle::named_generics_bundle;
/// # use Sized as SomeBounds;
/// #
/// #[named_generics_bundle]
/// trait SomeTrait {
///     type SomeAssocType: SomeBounds + Clone;
/// }
///
/// fn some_api<S: SomeTrait>(nameable: S::SomeAssocType) {
///     // Properly implied bounds.
///     nameable.clone();
/// }
///
/// /// We shall have `Example: SomeTrait + Sized + AllStdDeriveTraits`
/// type Example = SomeTrait![
///     # SomeAssocType = (), /*
///     SomeAssocType = ...,
///     # */
/// ];
///
/// some_api::<Example>(
///     // …,
///     # (),
/// );
/// // or, directly:
/// some_api::<SomeTrait![SomeAssocType = i32]>(42);
/// ```
///
/// More precisely, regarding the attribute macro invocation:
///
/// <details open class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
/// ```rust
/// # () /*
/// /// docs…
/// #[named_generics_bundle(
///   $(
///     // Optional. Path must be an absolute path (leading `crate`).
///     // It allows making the generated eponymous `SomeTrait!` macro be usable
///     // anywhere, that is, without having to have `SomeTrait` in the current scope.
///     //
///     // The macro will tell you what to put here if you get it wrong 🤓
///     path_to_this_very_module = crate::path::to::this::very::module,
///   )?
///   $(
///     // Optional. Niche and advanced. Useful when re-exporthing this macro from
///     // another crate (e.g. through some `#[macro_export]`ed macro).
///     path_to_named_generics_bundle_crate = ::my_crate::reexports::named_generics_bundle,
///   )?
/// )]
/// $pub:vis
/// trait SomeTrait $(: 'static)? {
///   $(
///     /// docs…
///     type $EachAssocType:ident $(: $TraitBounds…)?;
///   )*
/// }
/// # */
/// ```
///
/// </details>
///
/// # Features
///
/// <details open class="custom"><summary><span class="summary-box"><span>Click to hide</span></span></summary>
///
///   - ### `MyBundle!` eponymous macro definition
///
///     The main point of this all is the definition of the convenience `Eponymous!` macro alongside
///     the trait definition:
///
///     ```rust
///     #[::named_generics_bundle::named_generics_bundle]
///     trait MyBundle {
///         type Foo;
///         // …
///     }
///
///     // defines an eponymous "type-level-instantiation" macro.
///     # type Example =
///     MyBundle![
///         Foo = i32,
///         // …
///     ]
///     # ;
///     ```
///
///   - ### The `trait` definition remains `dyn`-compatible.
///
///     As a matter of fact, this is how the generated `Eponymous![]` macro works under the hood:
///
///     ```rust
///     # () /*
///     AnnotatedTrait![ A = B, C = D, … ]
///     // expands to:
///     PhantomData::<fn(ඞ<()>) -> dyn AnnotatedTrait<ඞ<()>, A = B, C = D, …>>
///     # */
///     ```
///
///       - Add to this a trivial blanket impl of `AnnotatedTrait` for
///         `PhantomData<fn(ඞ<()>) -> impl ?Sized + AnnotatedTrait<ඞ<()>>>`, and _voilà_!
///
///     In case the `ඞ<…>` wrapper did not give this away, do note that the precise form of this
///     expansion is **not** guaranteed, and therefore susceptible to change within non-major Semver
///     bumps, as **it will not be considered a breaking change**: the disclosure of this expansion
///     is done merely for educational/informative reasons.
///
///   - ### Implied Bounds (`Sized + Copy + Clone + Send/Sync + …`)
///
///     `T : AnnotatedTrait` entails not only `: Sized`, but also every
///     stdlib `#[derive()]` trait, for convenient use with such (dumb) derives.
///
///     More precisely, using an advanced trick with
///     [`::implied_bounds`](https://docs.rs/implied_bounds), the macro makes it so the `dyn`-safe
///     `trait` nonetheless entails the following:
///
///     ```rust
///     # () /*
///     T : AnnotatedTrait,
///     // to entail:
///     T :
///         Debug +
///         Copy + // and thus, `Clone`
///         Default +
///         Ord + // and thus, `PartialOrd + Eq + PartialEq`,
///         Hash +
///         Send + Sync + Unpin +
///     ,
///     # */
///     ```
///
/// </details>
///
/// # Quirks
///
///   - To keep things simple, the attribute rejects trait with generics parameters, or associated
///     items other than types.
///
///   - ## The `path_to_this_very_module = ` attribute arg
///
///     The generated `Eponymous!` macro, by default, needs to be in scope to be invoked.
///     That is, invoking it through some qualified path is most likely to result in a path
///     resolution error:
///
///     ```rust ,compile_fail
///     mod some_module {
///         #[::named_generics_bundle::named_generics_bundle]
///         pub trait Example {}
///     }
///
///     type T = some_module::Example![]; // ❌
///     #
///     # fn main() {}
///     ```
///
///     which fails with:
///
///     <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
///     ```rust ,ignore
///     # () /*
///     error[E0405]: cannot find trait `Example` in this scope
///       --> src/_lib.rs:185:15
///        |
///     7  |     pub trait Example {}
///        |               ^^^^^^^ not found in this scope
///     ...
///     10 | type T = some_module::Example![]; // ❌
///        |          ----------------------- in this macro invocation
///        |
///        = note: this error originates in the macro `some_module::Example` (in Nightly builds, run with -Z macro-backtrace for more info)
///     # */
///     ```
///
///     </details>
///
///     This is not a deliberate choice, but a limitation of the language, wherein a macro
///     invocation such as that of [`#[named_generics_bundle]`][`named_generics_bundle] is unable to know the full/absolute
///     [`module_path!()`] whence it has been invoked; and yet such a path would be useful or
///     even necessary to make an invocation like `some_module::Example![…]` robust.
///
///     Indeed, the invocation of `Example![…]` will want to involve, and name, `dyn Example`.
///
///       - Either the macro and/or the trait is in scope, and doing `dyn Example` will Just Work™.
///
///         This is the usual, happy-case case/scenario.
///
///       - Or it won't be in scope, like in this instance, and using a more qualified path, such
///         as `dyn crate::some_module::Example`, is necessary.
///
///     Hence the "need" for the invoker of the attribute macro to provide this `crate::some_module`
///     information to the macro, which is to be done _via_ the `path_to_this_very_module =`
///     attribute arg:
///
///     ```rust
///     pub mod some_module {
///         #[::named_generics_bundle::named_generics_bundle(
///             path_to_this_very_module = crate::some_module,
///         )]
///         pub trait Example {}
///     }
///
///     type T = some_module::Example![]; // ✅
///     #
///     # fn main() {}
///     ```
///
///       - Note: [`#[named_generics_bundle]`][`named_generics_bundle] will replace the `crate::`
///         part of this specifier with `$crate::` so as to make the `Example![]` macro it
///         generates, resilient to being used across crates / from a downstream dependent crate.
///
///   - ## The `path_to_named_generics_bundle_crate = ` attribute arg
///
///     Since this macro stems from a `proc-macro = true` backend using a frontend/façade package,
///     the attribute is unable to have a 100%-resilient way to name, and refer back to, its own
///     frontend crate items. So unless told how, the attribute has no choice but to assume that
///     users of the attribute will be direct dependents of `::named_generics_bundle`, resulting in
///     the expansion involving then some hard-coded `::named_generics_bundle`-prefixes.
///
///     This is usally fine, but for the case of a _non-direct_ downstream dependent (which is kind
///     of the case if a direct dependent `#[macro_export]`s some of our functionality to _deeper_,
///     non-direct, dependents).
///
///     At this point, some re-export of this `::named_generics_bundle` crate must have proven
///     necessary, from somehere. This is the path to be providing to this attribute args.
///
///     ```rust
///     # pub extern crate named_generics_bundle;
///     #
///     #[doc(hidden)] /** Not part of the public API */ pub
///     mod __internals {
///         /// 👇 Notice whence (and how) `::named_generics_bundle` is being reëxported
///         pub use ::named_generics_bundle as nmb;
///     }
///
///     /// Demo
///     /// ```rust
///     /// ::your_crate::my_fancy_macro!(); // It Works!
///     /// ```
///     #[macro_export]
///     macro_rules! my_fancy_macro {( ... ) => (
///         #[$crate::__internals::nmb::named_generics_bundle(
///             path_to_named_generics_bundle_crate = $crate::__internals::nmb,
///         )]
///         trait SomeTrait {
///             ...
///         }
///     )}
///     #
///     # fn main() {}
///     ```
///
pub use ::named_generics_bundle_proc_macros::named_generics_bundle;

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use ::core::{
        self,
        cmp::Ord,
        default::Default,
        fmt::Debug,
        hash::Hash,
        marker::{Copy, Send, Sync, Unpin},
    };
    pub use ::implied_bounds::ImpliedPredicate;
    pub use crate::{
        const_helpers::*,
    };

    /// This type is used to convey the notion that users of this attribute are
    /// not to rely on the blanket impl of:
    /// `impl Trait for PhantomData<fn() -> ඞ<impl ?Sized + Trait<()>>>`
    ///
    /// It effectively is, currently, just an identity type wrapper, but it shall make coherence
    /// treat the resulting type as being literally any possible type, thereby reducing the risk
    /// of "breaking changes" should I decide to change what the `Eponymous![]` macro unsugars to.
    pub type ඞ<T> = <T as Identity>::ItSelf;

    pub trait Identity { type ItSelf : ?Sized; }
    impl<T : ?Sized> Identity for T { type ItSelf = Self; }
}

mod const_helpers;

#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}

#[cfg(any(test, feature = "__internal_testing"))]
extern crate self as named_generics_bundle;

#[cfg(any(test, feature = "__internal_testing"))]
pub mod tests;
