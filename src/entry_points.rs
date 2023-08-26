use crate::types::TypesDefinitions;

/// Converts an entry point in a module into a collection of Rust definitions including the name and workgroup size
/// of the entry point, if representable.
pub fn make_entry_point(
    entry_point: &naga::EntryPoint,
    _module: &naga::Module,
    _types: &mut TypesDefinitions,
) -> Vec<syn::Item> {
    let mut items = Vec::new();

    let name = &entry_point.name;
    items.push(syn::Item::Const(syn::parse_quote! {
        pub const NAME: &'static str = #name;
    }));

    let x = entry_point.workgroup_size[0];
    let y = entry_point.workgroup_size[1];
    let z = entry_point.workgroup_size[2];
    if x != 0 && y != 0 && z != 0 {
        items.push(syn::Item::Const(syn::parse_quote! {
            pub const WORKGROUP_SIZE: [u32; 3] = [#x, #y, #z];
        }));
    }

    return items;
}

/// Builds a collection of entry points into a collection of Rust module definitions containing
/// each of the entry points' properties, such as name and workgroup size.
pub fn make_entry_points(module: &naga::Module, types: &mut TypesDefinitions) -> Vec<syn::Item> {
    let mut items = Vec::new();

    for entry_point in module.entry_points.iter() {
        let entry_point_items =
            crate::collect_tokenstream(make_entry_point(entry_point, module, types));

        let entry_point_name_ident = syn::parse_str::<syn::Ident>(&entry_point.name);
        let entry_point_name_ident = match entry_point_name_ident {
            Ok(entry_point_name_ident) => entry_point_name_ident,
            Err(_) => continue,
        };

        items.push(syn::Item::Mod(syn::parse_quote! {
            pub mod #entry_point_name_ident {
                #entry_point_items
            }
        }))
    }

    return items;
}
