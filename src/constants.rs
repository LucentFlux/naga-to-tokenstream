use crate::types::TypesDefinitions;

fn make_constant_value(
    constant: &naga::Constant,
    module: &naga::Module,
    types: &mut TypesDefinitions,
) -> Option<proc_macro2::TokenStream> {
    match &constant.inner {
        naga::ConstantInner::Scalar { width, value } => match (width, value) {
            (1, naga::ScalarValue::Bool(v)) => Some(quote::quote! {
                #v
            }),
            (4, naga::ScalarValue::Float(v)) => {
                let v = *v as f32;
                Some(quote::quote! {
                    #v
                })
            }
            (8, naga::ScalarValue::Float(v)) => Some(quote::quote! {
                #v
            }),
            (4, naga::ScalarValue::Sint(v)) => {
                let v = *v as i32;
                Some(quote::quote! {
                    #v
                })
            }
            (8, naga::ScalarValue::Sint(v)) => Some(quote::quote! {
                #v
            }),
            (4, naga::ScalarValue::Uint(v)) => {
                let v = *v as u32;
                Some(quote::quote! {
                    #v
                })
            }
            (8, naga::ScalarValue::Uint(v)) => Some(quote::quote! {
                #v
            }),
            _ => None,
        },
        naga::ConstantInner::Composite { ty, components } => {
            /*let components: Option<Vec<_>> = components
                .iter()
                .map(|handle| {
                    module
                        .constants
                        .try_get(*handle)
                        .ok()
                        .and_then(|constant| make_constant_value(constant, module, types))
                })
                .collect();
            let components = components?;*/

            None
        }
    }
}

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

    if let Some(specialization) = &constant.specialization {
        items.push(syn::Item::Const(syn::parse_quote! {
            pub const SPECIALIZATION: u32 = #specialization;
        }));
    }

    let ty_ident = match &constant.inner {
        naga::ConstantInner::Scalar { width, value } => match (width, value) {
            (1, naga::ScalarValue::Bool(_)) => Some(syn::parse_quote! {
                bool
            }),
            (4, naga::ScalarValue::Float(_)) => Some(syn::parse_quote! {
                f32
            }),
            (8, naga::ScalarValue::Float(_)) => Some(syn::parse_quote! {
                f64
            }),
            (4, naga::ScalarValue::Sint(_)) => Some(syn::parse_quote! {
                i32
            }),
            (8, naga::ScalarValue::Sint(_)) => Some(syn::parse_quote! {
                i64
            }),
            (4, naga::ScalarValue::Uint(_)) => Some(syn::parse_quote! {
                u32
            }),
            (8, naga::ScalarValue::Uint(_)) => Some(syn::parse_quote! {
                u64
            }),
            _ => None,
        },
        naga::ConstantInner::Composite { ty, components } => types.rust_type_ident(*ty, module),
    };
    let value = make_constant_value(constant, module, types);
    if let (Some(ty_ident), Some(value)) = (ty_ident, value) {
        items.push(syn::Item::Const(syn::parse_quote! {
            pub const VALUE: #ty_ident = #value ;
        }));
    }

    items
}

pub fn make_constants(module: &naga::Module, types: &mut TypesDefinitions) -> Vec<syn::Item> {
    let mut constants = Vec::new();

    for (_, constant) in module.constants.iter() {
        // Get name for constant module
        let constant_name = match &constant.name {
            Some(name) => name.clone(),
            None => continue,
        };
        let constant_name_ident = quote::format_ident!("{}", constant_name);

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
