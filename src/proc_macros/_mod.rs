//! Crate not intended for direct use.
//! Use https:://docs.rs/named-generics-bundle instead.
// Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template
#![allow(nonstandard_style, unused_imports, unused_braces)]

use ::core::{
    mem,
    ops::Not as _,
};
use proc_macro::TokenTree;
use ::proc_macro::{
    TokenStream,
};
use ::proc_macro2::{*,
    Span,
    TokenStream as TokenStream2,
    TokenTree as TT,
};
use ::quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    Result, // Explicitly shadow it
    spanned::Spanned,
};

mod args;

mod validate_module_path;

///
#[proc_macro_attribute] pub
fn named_generics_bundle(
    args: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    named_generics_bundle_impl(args.into(), input.into())
    //  .map(|ret| { println!("{}", ret); ret })
        .unwrap_or_else(|err| {
            let mut errors =
                err .into_iter()
                    .map(|err| Error::new(
                        err.span(),
                        format_args!("`#[named_generics_bundle::named_generics_bundle]`: {}", err),
                    ))
            ;
            let mut err = errors.next().unwrap();
            errors.for_each(|cur| err.combine(cur));
            err.to_compile_error()
        })
        .into()
}

/// Like `ItemTrait`, but restricted.
struct RestrictedItemTrait {
    attrs: Vec<Attribute>,
    pub_: Visibility,
    trait_: Token![trait],
    TraitName: Ident,
    supertraits: Punctuated<TypeParamBound, Token![+]>,
    braces: token::Brace,
    body: Punctuated<TraitItemType, parse::Nothing>,
}

impl Parse for RestrictedItemTrait {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let braces;
        let mut attrs: Vec<Attribute> = Attribute::parse_outer(input)?;
        Ok(Self {
            pub_: input.parse()?,
            trait_: input.parse()?,
            TraitName: input.parse()?,
            supertraits: {
                let semi: Option<Token![:]> = input.parse()?;
                let mut ret = Punctuated::default();
                if semi.is_some() {
                    while input.peek(token::Brace).not() {
                        ret.push_value(input.parse()?);
                        if let Some(plus) = input.parse()? {
                            ret.push_punct(plus);
                        } else {
                            break;
                        }
                    }
                }
                ret
            },
            body: {
                let inner;
                braces = braced!(inner in input);
                let input = &inner;

                attrs.extend(Attribute::parse_inner(input)?.into_iter().map(|mut attr| {
                    attr.style = AttrStyle::Outer;
                    attr
                }));

                Punctuated::parse_terminated(input)?
            },
            braces,
            attrs,
        })
    }
}

fn named_generics_bundle_impl(
    args: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    // By default deny any attribute present.
    let mut args: args::Args = parse2(args)?;
    let RestrictedItemTrait {
        attrs,
        pub_,
        trait_,
        ref TraitName,
        mut supertraits,
        braces,
        body,
    } = parse2(input)?;
    let krate = &args.krate.as_ref().map_or_else(|| quote_spanned!(Span::mixed_site()=>
        ::named_generics_bundle
    ), ToTokens::to_token_stream);
    let dol_krate = if let Some((krate, span)) = args.krate.as_ref().and_then(|p| {
        let s = p.segments.first().unwrap();
        (s.ident == "crate").then(|| (p, s.ident.span()))
    })
    {
        &quote_spanned!(span=>
            $ #krate
        )
    } else {
        krate
    };

    if supertraits.empty_or_trailing().not() {
        supertraits.push_punct(<_>::default());
    }

    let braced_body = &mut quote::quote!();
    braces.surround(braced_body, |ts| body.to_tokens(ts));

    let EachTypeName @ _ = body.iter().map(|ty| &ty.ident);

    let ඞTraitName @ _ = &format_ident!(
        "__proper_macro_rules_scopingඞnamed_generics_bundleඞ{TraitName}",
    );
    let TraitName_doclink = &format!(" [`{TraitName}`].");

    let TraitName_macro_invocation_nudge = &format!("\
 <!--
 ```rust ,ignore
 {TraitName}![];
 ```
 -->\
    ");


    let if_pub = matches!(pub_, Visibility::Public { .. }).then_some(quote!());
    let if_pub = if_pub.as_slice();

    let validate_module_path = validate_module_path::validate(krate, &args.module_path);

    if let Some(p) = &mut args.module_path {
        let last_span = p.segments.last().unwrap().span();
        p.segments.push_punct(token::PathSep {
            spans: [last_span; 2],
        });
    }
    let mb_module_path =
        args.module_path
            .as_ref()
            .map(|p| p.to_token_stream().into_iter().collect::<Vec<_>>())
            .map(|mut tts| match tts.first().unwrap() {
                // Lift `crate` to `$crate` to be used in the `macro_rules!` def below.
                TT::Ident(krate) if krate == "crate" => {
                    let mut dollar = Punct::new('$', Spacing::Joint);
                    dollar.set_span(krate.span());
                    tts.insert(0, dollar.into());
                    tts
                },
                _ => unreachable!("as per the current `Parse` implementation"),
            })
            .unwrap_or_default()
    ;
    // let QualifiedTraitName @ _ = args.module_path.as_ref().map_or_else(
    //     || TraitName.to_token_stream(),
    //     |p| {
    //         let span = p.segments.last().unwrap().span();
    //         let TraitName @ _ = Ident::new(&TraitName.to_string(), span);
    //         quote_spanned!(span=>
    //             #p #TraitName
    //         )
    //     },
    // );
    //
    // Note: we do not use this trick to validate anymore, since we have `validate_module_path`.
    let QualifiedTraitName = TraitName;

    Ok(quote_spanned!(Span::mixed_site()=>
        #validate_module_path

        #(#attrs)*
        #pub_
        #trait_ #TraitName <ඞImpliedDeriveBounds = Self>
        :
            #supertraits

            #krate::ඞ::ImpliedPredicate<
                ඞImpliedDeriveBounds,
                Impls :
                    #krate::ඞ::Debug +
                    #krate::ඞ::Copy +
                    #krate::ඞ::Ord +
                    #krate::ඞ::Hash +
                    #krate::ඞ::Default +

                    #krate::ඞ::Send +
                    #krate::ඞ::Sync +
                    #krate::ඞ::Unpin +
                ,
            > +
        #braced_body

        // while we could just use `#TraitName` here, this gives us a simple sanity check
        // that the provided `module_path` (if any), be correct.
        impl<ඞDyn : ?#krate::ඞ::core::marker::Sized + #QualifiedTraitName<()>>
            #TraitName
        for
            #krate::ඞ::core::marker::PhantomData<fn() -> #krate::ඞ::ඞ<ඞDyn>>
        {
            #(
                type #EachTypeName = ඞDyn::#EachTypeName;
            )*
        }

        /// Helper macro to produce an on-the-fly `Sized` "bundle of generic parameters" which
        /// implements
        #[doc = #TraitName_doclink]
        ///
        /// It also implements `Debug + Copy + Ord + Hash + Default`, so as to be
        /// dumb-stdlib-`#[derive()]`-friendly.

        // Nudge `rust-analyzer` auto-complete to suggest using square brackets for these macros.
        #[doc = #TraitName_macro_invocation_nudge]
        #(#if_pub
            #[macro_export]
        )*
        #[doc(hidden)]
        macro_rules! #ඞTraitName {(
            $($named_generics:tt)*
        ) => (
            ::core::marker::PhantomData::<fn() -> #dol_krate::ඞ::ඞ<dyn #(#mb_module_path)* #TraitName<
                (),
                $($named_generics)*
            >>>
        )}
        #[doc(inline)]
        #pub_ use #ඞTraitName as #TraitName;
    ))
}
