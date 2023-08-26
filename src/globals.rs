use std::collections::HashMap;

use crate::types::TypesDefinitions;

fn make_global_binding(
    binding: &naga::ResourceBinding,
    _global: &naga::GlobalVariable,
    _module: &naga::Module,
) -> Vec<syn::Item> {
    let mut binding_items = Vec::new();

    let group = binding.group;
    let binding = binding.binding;
    binding_items.push(syn::Item::Const(syn::parse_quote! {
        pub const GROUP: u32 = #group;
    }));
    binding_items.push(syn::Item::Const(syn::parse_quote! {
        pub const BINDING: u32 = #binding;
    }));

    return binding_items;
}

#[cfg(feature = "naga")]
fn address_space_to_tokens(address_space: naga::AddressSpace) -> proc_macro2::TokenStream {
    match address_space {
        naga::AddressSpace::Function => quote::quote!(naga::AddressSpace::Function),
        naga::AddressSpace::Private => quote::quote!(naga::AddressSpace::Private),
        naga::AddressSpace::WorkGroup => quote::quote!(naga::AddressSpace::WorkGroup),
        naga::AddressSpace::Uniform => quote::quote!(naga::AddressSpace::Uniform),
        naga::AddressSpace::Storage { access } => {
            let bits = access.bits();
            quote::quote!(naga::AddressSpace::Storage {
                access: naga::StorageAccess::from_bits_retain(#bits)
            })
        }
        naga::AddressSpace::Handle => quote::quote!(naga::AddressSpace::Handle),
        naga::AddressSpace::PushConstant => quote::quote!(naga::AddressSpace::PushConstant),
    }
}

pub fn make_global(
    global: &naga::GlobalVariable,
    module: &naga::Module,
    types: &mut TypesDefinitions,
) -> Vec<syn::Item> {
    let mut global_items = Vec::new();

    if let Some(name) = &global.name {
        global_items.push(syn::Item::Const(syn::parse_quote! {
            pub const NAME: &'static str = #name;
        }));
    }

    #[cfg(feature = "naga")]
    {
        let space = address_space_to_tokens(global.space);
        global_items.push(syn::Item::Const(syn::parse_quote! {
            #[allow(unused)]
            pub const SPACE: naga::AddressSpace = #space;
        }));
    }

    if let Some(type_ident) = types.rust_type_ident(global.ty, module) {
        global_items.push(syn::Item::Type(syn::parse_quote! {
            pub type Ty = #type_ident;
        }));
    }

    if let Some(binding) = &global.binding {
        let binding_items = make_global_binding(binding, global, module);
        if binding_items.len() > 0 {
            let binding_items = crate::collect_tokenstream(binding_items);

            global_items.push(syn::Item::Mod(syn::parse_quote! {
                pub mod binding {
                    #binding_items
                }
            }));
        }
    }

    return global_items;
}

pub fn make_globals(module: &naga::Module, types: &mut TypesDefinitions) -> Vec<syn::Item> {
    let mut globals = Vec::new();

    // Info about each global individually
    for (_, global) in module.global_variables.iter() {
        // Get name for global module
        let global_name = match &global.name {
            Some(name) => name.clone(),
            None => continue,
        };
        let global_name_ident = syn::parse_str::<syn::Ident>(&global_name);
        let global_name_ident = match global_name_ident {
            Ok(global_name_ident) => global_name_ident,
            Err(_) => continue,
        };

        // Make items within module
        let global_items = crate::collect_tokenstream(make_global(global, module, types));

        // Collate into an inner module
        let doc = format!(
            "Information about the `{}` global variable within this shader module.",
            global_name
        );
        globals.push(syn::parse_quote! {
            #[doc = #doc]
            pub mod #global_name_ident {
                #[allow(unused)]
                use super::*;

                #global_items
            }
        })
    }

    // Info about all globals together
    let mut groups = HashMap::new();
    for (_, global) in module.global_variables.iter() {
        if let Some(binding) = &global.binding {
            groups.entry(binding.group).or_insert(vec![]).push(global)
        }
    }
    //TODO: Create `create_bind_groups` ctr function

    return globals;
}
