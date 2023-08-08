use std::collections::HashMap;

use proc_macro2::TokenStream;

/// Returns a base Rust or `glam` type that corresponds to a TypeInner, if one exists.
fn rust_type(type_inner: &naga::TypeInner) -> Option<syn::Type> {
    match type_inner {
        naga::TypeInner::Scalar { kind, width } => match (kind, width) {
            (naga::ScalarKind::Bool, 1) => Some(syn::parse_quote!(bool)),
            (naga::ScalarKind::Float, 4) => Some(syn::parse_quote!(f32)),
            (naga::ScalarKind::Float, 8) => Some(syn::parse_quote!(f64)),
            (naga::ScalarKind::Sint, 4) => Some(syn::parse_quote!(i32)),
            (naga::ScalarKind::Sint, 8) => Some(syn::parse_quote!(i64)),
            (naga::ScalarKind::Uint, 4) => Some(syn::parse_quote!(u32)),
            (naga::ScalarKind::Uint, 8) => Some(syn::parse_quote!(u64)),
            _ => None,
        },
        naga::TypeInner::Vector { size, kind, width } => {
            if cfg!(feature = "glam") {
                match (size, kind, width) {
                    (naga::VectorSize::Bi, naga::ScalarKind::Bool, 1) => {
                        Some(syn::parse_quote!(glam::bool::BVec2))
                    }
                    (naga::VectorSize::Tri, naga::ScalarKind::Bool, 1) => {
                        Some(syn::parse_quote!(glam::bool::BVec3))
                    }
                    (naga::VectorSize::Quad, naga::ScalarKind::Bool, 1) => {
                        Some(syn::parse_quote!(glam::bool::BVec4))
                    }
                    (naga::VectorSize::Bi, naga::ScalarKind::Float, 4) => {
                        Some(syn::parse_quote!(glam::f32::Vec2))
                    }
                    (naga::VectorSize::Tri, naga::ScalarKind::Float, 4) => {
                        Some(syn::parse_quote!(glam::f32::Vec3))
                    }
                    (naga::VectorSize::Quad, naga::ScalarKind::Float, 4) => {
                        Some(syn::parse_quote!(glam::f32::Vec4))
                    }
                    (naga::VectorSize::Bi, naga::ScalarKind::Float, 8) => {
                        Some(syn::parse_quote!(glam::f64::DVec2))
                    }
                    (naga::VectorSize::Tri, naga::ScalarKind::Float, 8) => {
                        Some(syn::parse_quote!(glam::f64::DVec3))
                    }
                    (naga::VectorSize::Quad, naga::ScalarKind::Float, 8) => {
                        Some(syn::parse_quote!(glam::f64::DVec4))
                    }
                    (naga::VectorSize::Bi, naga::ScalarKind::Sint, 4) => {
                        Some(syn::parse_quote!(glam::i32::IVec2))
                    }
                    (naga::VectorSize::Tri, naga::ScalarKind::Sint, 4) => {
                        Some(syn::parse_quote!(glam::i32::IVec3))
                    }
                    (naga::VectorSize::Quad, naga::ScalarKind::Sint, 4) => {
                        Some(syn::parse_quote!(glam::i32::IVec4))
                    }
                    (naga::VectorSize::Bi, naga::ScalarKind::Sint, 8) => {
                        Some(syn::parse_quote!(glam::i64::I64Vec2))
                    }
                    (naga::VectorSize::Tri, naga::ScalarKind::Sint, 8) => {
                        Some(syn::parse_quote!(glam::i64::I64Vec3))
                    }
                    (naga::VectorSize::Quad, naga::ScalarKind::Sint, 8) => {
                        Some(syn::parse_quote!(glam::i64::I64Vec4))
                    }
                    (naga::VectorSize::Bi, naga::ScalarKind::Uint, 4) => {
                        Some(syn::parse_quote!(glam::u32::UVec2))
                    }
                    (naga::VectorSize::Tri, naga::ScalarKind::Uint, 4) => {
                        Some(syn::parse_quote!(glam::u32::UVec3))
                    }
                    (naga::VectorSize::Quad, naga::ScalarKind::Uint, 4) => {
                        Some(syn::parse_quote!(glam::u32::UVec4))
                    }
                    (naga::VectorSize::Bi, naga::ScalarKind::Uint, 8) => {
                        Some(syn::parse_quote!(glam::u64::U64Vec2))
                    }
                    (naga::VectorSize::Tri, naga::ScalarKind::Uint, 8) => {
                        Some(syn::parse_quote!(glam::u64::U64Vec3))
                    }
                    (naga::VectorSize::Quad, naga::ScalarKind::Uint, 8) => {
                        Some(syn::parse_quote!(glam::u64::U64Vec4))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        naga::TypeInner::Matrix {
            columns,
            rows,
            width,
        } => {
            if cfg!(feature = "glam") {
                match (columns, rows, width) {
                    (naga::VectorSize::Bi, naga::VectorSize::Bi, 4) => {
                        Some(syn::parse_quote!(glam::f32::Mat2))
                    }
                    (naga::VectorSize::Tri, naga::VectorSize::Tri, 4) => {
                        Some(syn::parse_quote!(glam::f32::Mat3))
                    }
                    (naga::VectorSize::Quad, naga::VectorSize::Quad, 4) => {
                        Some(syn::parse_quote!(glam::f32::Mat4))
                    }
                    (naga::VectorSize::Bi, naga::VectorSize::Bi, 8) => {
                        Some(syn::parse_quote!(glam::f64::DMat2))
                    }
                    (naga::VectorSize::Tri, naga::VectorSize::Tri, 8) => {
                        Some(syn::parse_quote!(glam::f64::DMat3))
                    }
                    (naga::VectorSize::Quad, naga::VectorSize::Quad, 8) => {
                        Some(syn::parse_quote!(glam::f64::DMat4))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        naga::TypeInner::Atomic { kind, width } => rust_type(&naga::TypeInner::Scalar {
            kind: *kind,
            width: *width,
        }),
        _ => None,
    }
}

pub struct TypesDefinitions {
    pub definitions: Vec<syn::ItemStruct>,
    pub(crate) references: HashMap<naga::Handle<naga::Type>, syn::Type>,
}

impl TypesDefinitions {
    pub fn new(module: &naga::Module) -> Self {
        let mut res = Self {
            definitions: Vec::new(),
            references: HashMap::new(),
        };

        for (ty_handle, _) in module.types.iter() {
            if let Some(new_ty_ident) = res.try_make_type(ty_handle, module) {
                res.references.insert(ty_handle, new_ty_ident.clone());
            }
        }

        return res;
    }

    fn try_make_type(
        &mut self,
        ty_handle: naga::Handle<naga::Type>,
        module: &naga::Module,
    ) -> Option<syn::Type> {
        let ty = match module.types.get_handle(ty_handle) {
            Err(_) => return None,
            Ok(ty) => ty,
        };
        if let Some(ty_ident) = rust_type(&ty.inner) {
            return Some(ty_ident);
        };

        match &ty.inner {
            naga::TypeInner::Array { base, size, .. }
            | naga::TypeInner::BindingArray { base, size } => {
                let base_type = self.rust_type_ident(*base, module)?;
                match size {
                    naga::ArraySize::Constant(size) => {
                        let size = module.constants.try_get(*size).ok()?;
                        let size = match size.inner {
                            naga::ConstantInner::Scalar { value, .. } => match value {
                                naga::ScalarValue::Sint(v) => usize::try_from(v).ok(),
                                naga::ScalarValue::Uint(v) => usize::try_from(v).ok(),
                                _ => None,
                            },
                            _ => None,
                        };
                        let size = size?;
                        Some(syn::parse_quote!([#base_type; #size]))
                    }
                    naga::ArraySize::Dynamic => Some(syn::parse_quote!(Vec<#base_type>)),
                }
            }
            naga::TypeInner::Struct { members, .. } => {
                let members_have_names = members.iter().all(|member| member.name.is_some());
                let members: Option<Vec<_>> = members
                    .into_iter()
                    .enumerate()
                    .map(|(i_member, member)| {
                        let member_name = if members_have_names {
                            let member_name =
                                member.name.as_ref().expect("all members had names").clone();
                            quote::format_ident!("{}", member_name)
                        } else {
                            quote::format_ident!("v{}", i_member)
                        };
                        let member_ty = self.rust_type_ident(member.ty, module);

                        let mut attributes = proc_macro2::TokenStream::new();
                        // Runtime-sized fields must be marked as such when using encase
                        if cfg!(feature = "encase") {
                            let ty = module.types.get_handle(member.ty);
                            if let Ok(naga::Type { inner, .. }) = ty {
                                match inner {
                                    naga::TypeInner::Array {
                                        size: naga::ArraySize::Dynamic,
                                        ..
                                    }
                                    | naga::TypeInner::BindingArray {
                                        size: naga::ArraySize::Dynamic,
                                        ..
                                    } => attributes.extend(quote::quote!(#[size(runtime)])),
                                    _ => {}
                                }
                            }
                        }

                        member_ty.map(|member_ty| {
                            quote::quote! {
                                #attributes
                                pub #member_name: #member_ty
                            }
                        })
                    })
                    .collect();
                members.and_then(|members| {
                    ty.name.as_ref().map(|name| {
                        let name = quote::format_ident!("{}", name);

                        #[allow(unused_mut)]
                        let mut bonus_struct_derives = TokenStream::new();
                        #[cfg(feature = "encase")]
                        {
                            bonus_struct_derives.extend(quote::quote!(encase::ShaderType,))
                        }

                        self.definitions.push(syn::parse_quote! {
                            #[allow(unused, non_camel_case_types)]
                            #[derive(Debug, PartialEq, Hash, Clone, #bonus_struct_derives)]
                            pub struct #name {
                                #(#members ,)*
                            }
                        });
                        syn::parse_quote!(#name)
                    })
                })
            }
            _ => None,
        }
    }

    pub(crate) fn rust_type_ident(
        &mut self,
        ty_handle: naga::Handle<naga::Type>,
        _module: &naga::Module,
    ) -> Option<syn::Type> {
        self.references.get(&ty_handle).cloned()
    }
}

pub fn make_types(_module: &naga::Module, types: TypesDefinitions) -> Vec<syn::Item> {
    let mut items = Vec::new();

    // Add struct definitions used by other things in the module.
    items.extend(
        types
            .definitions
            .into_iter()
            .map(|item_struct| syn::Item::Struct(item_struct)),
    );

    return items;
}
