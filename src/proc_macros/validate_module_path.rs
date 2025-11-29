//! Used in conjunction with `const_helpers.rs`.

use super::*;

pub(crate)
fn validate(krate: &TokenStream2, module_path: &Option<Path>) -> Option<TokenStream2> {
    module_path.as_ref().map(|module_path| {
        let module_path = module_path.to_token_stream().into_iter().collect::<Vec<_>>();
        let start_span = module_path.first().unwrap().span();
        let end_span = module_path.last().unwrap().span();
        let panic = quote_spanned!(start_span=>
            panic!
        );
        quote_spanned!(end_span=>
            const _: () = {
                use #krate::ඞ::{
                    core::{
                        module_path,
                        panic,
                        primitive::{
                            str,
                        },
                        stringify,
                        unreachable,
                    },
                    constcat,
                    eq_modulo_whitespace,
                    find_subslice,
                };

                const MODULE_PATH: &str = {
                    const PATH: &str = find_subslice(module_path!(), b':');
                    let Ok(s) = ::core::str::from_utf8(const {&
                        constcat::<{ "crate".len() + PATH.len() }, 2>([
                            "crate",
                            PATH,
                        ])
                    }) else {
                        unreachable!();
                    };
                    s
                };

                if !eq_modulo_whitespace(
                    stringify!( #(#module_path)* ),
                    MODULE_PATH,
                )
                {
                    const PREFIX: &str = "expected `";
                    const SUFFIX: &str = "`";
                    let Ok(msg) = ::core::str::from_utf8(const {&
                        constcat::<{PREFIX.len() + MODULE_PATH.len() + SUFFIX.len() }, 3>([
                            PREFIX,
                            MODULE_PATH,
                            SUFFIX,
                        ])
                    }) else {
                        unreachable!();
                    };
                    #panic { "{}", msg }
                }
            };
        )
    })
}
