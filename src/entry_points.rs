use crate::types::TypesDefinitions;

/// Converts an entry point in a module into a collection of Rust definitions including the name and workgroup size
/// of the entry point, if representable.
pub fn make_entry_point(
    entry_point: &naga::EntryPoint,
    module: &naga::Module,
    _types: &mut TypesDefinitions,
) -> Vec<syn::Item> {
    let mut items = Vec::new();

    // Entry point name
    let name = &entry_point.name;
    items.push(syn::Item::Const(syn::parse_quote! {
        pub const NAME: &'static str = #name;
    }));

    // Workgroup size
    let x = entry_point.workgroup_size[0];
    let y = entry_point.workgroup_size[1];
    let z = entry_point.workgroup_size[2];
    if x != 0 && y != 0 && z != 0 {
        items.push(syn::Item::Const(syn::parse_quote! {
            pub const WORKGROUP_SIZE: [u32; 3] = [#x, #y, #z];
        }));
    }

    // The module sourcecode, excluding all other entry points. Useful for more aggressive minification
    if let Some(src) = crate::module_to_source(module, Some(entry_point.name.clone())) {
        items.push(syn::parse_quote! {
            #[doc = "The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used."]
            pub const EXCLUSIVE_SOURCE: &'static str = #src;
        });
    }

    items
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

    items
}

/// Removes entry points which don't have the name given
pub(crate) fn filter_entry_points(module: &mut naga::Module, retain_entry_point: String) {
    let old_entry_points = std::mem::take(&mut module.entry_points);
    let filtered_entry_point = old_entry_points
        .into_iter()
        .find(|ep| ep.name == retain_entry_point);

    module.entry_points = match filtered_entry_point {
        Some(filtered_entry_point) => vec![filtered_entry_point],
        None => vec![],
    };
}
