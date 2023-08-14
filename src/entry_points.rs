use crate::types::TypesDefinitions;

fn make_entry_point(
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

pub fn make_entry_points(module: &naga::Module, types: &mut TypesDefinitions) -> Vec<syn::Item> {
    let mut items = Vec::new();

    for entry_point in module.entry_points.iter() {
        let entry_point_items =
            crate::collect_tokenstream(make_entry_point(entry_point, module, types));

        let name = quote::format_ident!("{}", entry_point.name);
        items.push(syn::Item::Mod(syn::parse_quote! {
            pub mod #name {
                #entry_point_items
            }
        }))
    }

    return items;
}
