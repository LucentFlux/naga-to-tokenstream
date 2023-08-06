pub mod entry_points;
pub mod globals;
pub mod types;

fn collect_tokenstream<I: quote::ToTokens>(
    items: impl IntoIterator<Item = I>,
) -> proc_macro2::TokenStream {
    let mut tokens = proc_macro2::TokenStream::new();
    for item in items {
        item.to_tokens(&mut tokens);
    }

    tokens
}

fn module_to_source(module: &naga::Module) -> Option<String> {
    // Reborrow for function scope
    #[allow(unused_mut)]
    let mut module = &*module;

    #[cfg(feature = "minify")]
    let mut minified;
    #[cfg(feature = "minify")]
    {
        minified = module.clone();
        wgsl_minifier::remove_identifiers(&mut minified);
        module = &minified;
    }
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::empty(),
        naga::valid::Capabilities::empty(),
    )
    .validate(module);

    let info = info.ok()?;
    let src = naga::back::wgsl::write_string(module, &info, naga::back::wgsl::WriterFlags::empty())
        .ok()?;

    #[cfg(feature = "minify")]
    let src = wgsl_minifier::minify_wgsl_source_whitespace(&src);

    return Some(src);
}

mod sealed {
    pub trait SealedModule {}
    impl SealedModule for naga::Module {}
}

pub trait ModuleToTokens: sealed::SealedModule {
    fn to_items(&self) -> Vec<syn::Item>;
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        collect_tokenstream(self.to_items())
    }
}
impl ModuleToTokens for naga::Module {
    fn to_items(&self) -> Vec<syn::Item> {
        let mut items = Vec::new();
        let mut types = types::TypesDefinitions::new();

        // Globals
        let globals = collect_tokenstream(globals::make_globals(self, &mut types));
        items.push(syn::parse_quote! {
            #[allow(unused)]
            #[doc = "Information about the globals within the module, exposed as constants and functions."]
            pub mod globals {
                #[allow(unused)]
                use super::*;

                #globals
            }
        });

        // Entry Points
        let entry_points = collect_tokenstream(entry_points::make_entry_points(self, &mut types));
        items.push(syn::parse_quote! {
            #[allow(unused)]
            #[doc = "Information about the entry points within the module, exposed as constants and functions."]
            pub mod entry_points {
                #[allow(unused)]
                use super::*;

                #entry_points
            }
        });

        // Types
        let types = collect_tokenstream(types::make_types(self, types));
        items.push(syn::parse_quote! {
            #[allow(unused)]
            #[doc = "Equivalent Rust definitions of the types defined in this module."]
            pub mod types {
                #types
            }
        });
        // We use all the types from the types mod in other modules.
        items.push(syn::parse_quote! {
            #[allow(unused)]
            use types::*;
        });

        // Source string
        // This must be done last since we want to minify only after everything else has been generated about the shader.
        if let Some(src) = module_to_source(self) {
            items.push(syn::parse_quote! {
                #[doc = "The sourcecode for the shader, as a constant string."]
                pub const SOURCE: &'static str = #src;
            });
        }

        items
    }
}
