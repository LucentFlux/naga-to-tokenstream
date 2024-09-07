#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::collections::HashSet;

/// Methods for converting sets of `naga::Constant`s to token streams.
pub mod constants;
/// Methods for converting sets of `naga::EntryPoint`s to token streams.
pub mod entry_points;
/// Methods for converting sets of `naga::GlobalVariable`s to token streams.
pub mod globals;
/// Methods for converting sets of `naga::Type`s to token streams.
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

fn module_to_source(module: &naga::Module, retain_entry_point: Option<String>) -> Option<String> {
    // Clone since we sometimes modify things
    #[allow(unused_mut)]
    let mut module = module.clone();

    // We allow only a single entry point, for specialised source strings per entry point.
    if let Some(retain_entry_point) = retain_entry_point {
        entry_points::filter_entry_points(&mut module, retain_entry_point);
    }

    // If we minify, do the first pass before writing out
    #[cfg(feature = "minify")]
    {
        wgsl_minifier::minify_module(&mut module);
    }

    // Mini validation to get module info
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::empty(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module);

    // Write to wgsl
    let info = info.ok()?;
    let src =
        naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
            .ok()?;

    // Remove whitespace if minifying
    #[cfg(feature = "minify")]
    let src = wgsl_minifier::minify_wgsl_source(&src);

    Some(src)
}

/// The configuration required to create a token stream describing a module.
#[derive(Default)]
pub struct ModuleToTokensConfig {
    /// A filter on the structs to expose. This is useful specifically when using the `encase` feature,
    /// since many structs can't be encoded or decoded. It is therefore the using crate's responsibility
    /// to expose this in some way, for example by having structs that should be exported to Rust require
    /// an attribute.
    pub structs_filter: Option<HashSet<String>>,
    /// Generate `glam` types.
    pub gen_glam: bool,
    /// Generate `encase` types.
    pub gen_encase: bool,
    /// Generate `naga` types.
    pub gen_naga: bool,
}

mod sealed {
    pub trait SealedModule {}
    impl SealedModule for naga::Module {}
}

/// An extension trait for `naga::Module` which exposes the functionality of this crate.
///
/// # Usage
///
/// ```
/// use naga_to_tokenstream::{ModuleToTokens, ModuleToTokensConfig};
///
/// let my_module = naga::Module::default();
/// let token_representation = my_module.to_tokens(ModuleToTokensConfig::default());
/// ```
pub trait ModuleToTokens: sealed::SealedModule {
    /// Converts a module to a set of `syn` module items, representing the module.
    fn to_items(&self, cfg: ModuleToTokensConfig) -> Vec<syn::Item>;
    /// Convenience method which calls `to_items` and then flattens the items to a single tokenstream.
    fn to_tokens(&self, cfg: ModuleToTokensConfig) -> proc_macro2::TokenStream {
        collect_tokenstream(self.to_items(cfg))
    }
}
impl ModuleToTokens for naga::Module {
    fn to_items(&self, cfg: ModuleToTokensConfig) -> Vec<syn::Item> {
        let mut items = Vec::new();
        let mut types = types::TypesDefinitions::new(self, cfg.structs_filter.clone(), &cfg);

        // Globals
        let globals = collect_tokenstream(globals::make_globals(self, &mut types, &cfg));
        items.push(syn::parse_quote! {
            #[allow(unused)]
            #[doc = "Information about the globals within the module, exposed as constants and functions."]
            pub mod globals {
                #[allow(unused)]
                use super::*;

                #globals
            }
        });

        // Constants
        let constants = collect_tokenstream(constants::make_constants(self, &mut types, &cfg));
        items.push(syn::parse_quote! {
            #[allow(unused)]
            #[doc = "Information about the constants within the module, exposed as constants and functions."]
            pub mod constants {
                #[allow(unused)]
                use super::*;

                #constants
            }
        });

        // Entry Points
        let entry_points = collect_tokenstream(entry_points::make_entry_points(self, &mut types, &cfg));
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
        let types = collect_tokenstream(types.definitions());
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
        if let Some(src) = module_to_source(self, None) {
            items.push(syn::parse_quote! {
                #[doc = "The sourcecode for the shader, as a constant string."]
                pub const SOURCE: &'static str = #src;
            });
        }

        items
    }
}
