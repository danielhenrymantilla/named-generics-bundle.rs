use super::*;

pub(crate) struct Args {
    pub(crate) module_path: Option<Path>,
    pub(crate) krate: Option<Path>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        const USAGE: &str = "help:\n\
Usage:\
    #[named_generics_bundle(
        // Optional. Must be an absolute path (leading `crate`).
        path_to_this_very_module = crate::some::path,
        // Optional.
        path_to_named_generics_bundle_crate = some::path,
    )]\
        ";
        || -> Result<_> {
            let mut module_path = None;
            let mut krate = None;

            while input.is_empty().not() {
                mod kw {
                    ::syn::custom_keyword!(path_to_this_very_module);
                    ::syn::custom_keyword!(path_to_named_generics_bundle_crate);
                }

                let snoopy = input.lookahead1();
                match () {
                    _case if snoopy.peek(kw::path_to_this_very_module) => {
                        if module_path.is_some() {
                            return Err(input.error("duplicate entry"));
                        }
                        let _: kw::path_to_this_very_module = input.parse().unwrap();
                        let _: Token![=] = input.parse()?;
                        if input.peek(Token![crate]).not() {
                            return Err(input.error("\
                                path must be absolute and start with `crate::` \
                                (instead of `your_crate_name`)\
                            "));
                        }
                        module_path = Some(Path::parse_mod_style(input)?);
                    },
                    _case if snoopy.peek(kw::path_to_named_generics_bundle_crate) => {
                        if krate.is_some() {
                            return Err(input.error("duplicate entry"));
                        }
                        let _: kw::path_to_named_generics_bundle_crate = input.parse().unwrap();
                        let _: Token![=] = input.parse()?;
                        krate = Some(Path::parse_mod_style(input)?);
                    },
                    _default => return Err(snoopy.error()),
                }
                let _: Option<Token![,]> = input.parse()?;
            }

            Ok(Self {
                module_path,
                krate,
            })
        }().map_err(|mut err| {
            err.combine(Error::new_spanned(&err.to_compile_error(), USAGE));
            err
        })
    }
}
