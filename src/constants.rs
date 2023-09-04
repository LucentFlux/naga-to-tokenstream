use crate::types::TypesDefinitions;

fn make_constant_value(
    constant: &naga::Constant,
    module: &naga::Module,
    _types: &mut TypesDefinitions,
) -> Option<proc_macro2::TokenStream> {
    let expr = module.const_expressions.try_get(constant.init).ok()?;

    match expr {
        naga::Expression::Literal(lit) => Some(match lit {
            naga::Literal::F64(v) => quote::quote! {
                #v
            },
            naga::Literal::F32(v) => quote::quote! {
                #v
            },
            naga::Literal::U32(v) => quote::quote! {
                #v
            },
            naga::Literal::I32(v) => quote::quote! {
                #v
            },
            naga::Literal::Bool(v) => quote::quote! {
                #v
            },
        }),
        naga::Expression::ZeroValue(_) => Some(quote::quote! {
            Default::default();
        }),
        _ => None,
    }
}

/// Converts a constant in a module into a collection of Rust definitions including the type and value of the constant,
/// if representable.
pub fn make_constant(
    constant: &naga::Constant,
    module: &naga::Module,
    types: &mut TypesDefinitions,
) -> Vec<syn::Item> {
    let mut items = Vec::new();

    if let Some(name) = &constant.name {
        items.push(syn::Item::Const(syn::parse_quote! {
            pub const NAME: &'static str = #name;
        }));
    }

    if let naga::Override::ByNameOrId(id) = &constant.r#override {
        items.push(syn::Item::Const(syn::parse_quote! {
            pub const OVERRIDE_ID: u32 = #id;
        }));
    }

    let ty_ident = types.rust_type_ident(constant.ty, module);
    let value = make_constant_value(constant, module, types);
    if let (Some(ty_ident), Some(value)) = (ty_ident, value) {
        items.push(syn::Item::Const(syn::parse_quote! {
            pub const VALUE: #ty_ident = #value ;
        }));
    }

    items
}

/// Builds a collection of constants into a collection of Rust module definitions containing
/// each of the constants properties, such as type and value.
pub fn make_constants(module: &naga::Module, types: &mut TypesDefinitions) -> Vec<syn::Item> {
    let mut constants = Vec::new();

    for (_, constant) in module.constants.iter() {
        // Get name for constant module
        let constant_name = match &constant.name {
            Some(name) => name.clone(),
            None => continue,
        };
        let constant_name_ident = syn::parse_str::<syn::Ident>(&constant_name);
        let constant_name_ident = match constant_name_ident {
            Ok(constant_name_ident) => constant_name_ident,
            Err(_) => continue,
        };

        // Make items within module
        let constant_items = crate::collect_tokenstream(make_constant(constant, module, types));

        // Collate into an inner module
        let doc = format!(
            "Information about the `{}` constant variable within this shader module.",
            constant_name
        );
        constants.push(syn::parse_quote! {
            #[doc = #doc]
            pub mod #constant_name_ident {
                #[allow(unused)]
                use super::*;

                #constant_items
            }
        })
    }

    constants
}
