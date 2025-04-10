#[test]
fn readme_snapshot() {
    // This is the code in the README, so make sure these match if you make a change here or there.
    let src = naga::front::wgsl::parse_str(
        r#"
    const ELEMENTS_LENGTH: u32 = 128u;

    struct Foo {
        a: i32,
        b: vec4<u32>,
        c: vec4<u32>,
    }
    struct Bar {
        size: u32,
        elements: array<vec2<bool>, ELEMENTS_LENGTH>,
        foos: array<Foo>
    }

    @group(0) @binding(0) var<storage> bar: Bar;

    @compute
    @workgroup_size(256,1,1)
    fn main() {

    }
    "#,
    )
    .unwrap();
    let tokens = naga_to_tokenstream::ModuleToTokens::to_tokens(
        &src,
        naga_to_tokenstream::ModuleToTokensConfig {
            structs_filter: None,
            gen_glam: true,
            gen_encase: true,
            gen_naga: true,
        },
    );

    let expected = quote::quote! {
        #[allow(unused)]
        #[doc = "Information about the globals within the module, exposed as constants and functions."]
        pub mod globals {
            #[allow(unused)]
            use super::*;
            #[doc = "Information about the `bar` global variable within this shader module."]
            pub mod bar {
                #[allow(unused)]
                use super::*;
                pub const NAME: &'static str = "bar";
                #[allow(unused)]
                pub const SPACE: naga::AddressSpace = naga::AddressSpace::Storage {
                    access: naga::StorageAccess::from_bits_retain(1u32)
                };
                pub type Ty = Bar;
                pub mod binding {
                    pub const GROUP: u32 = 0u32;
                    pub const BINDING: u32 = 0u32;
                }
            }
        }
        #[allow(unused)]
        #[doc = "Information about the constants within the module, exposed as constants and functions."]
        pub mod constants {
            #[allow(unused)]
            use super::*;
            #[doc = "Information about the `ELEMENTS_LENGTH` constant variable within this shader module."]
            pub mod ELEMENTS_LENGTH {
                #[allow(unused)]
                use super::*;
                pub const NAME: &'static str = "ELEMENTS_LENGTH";
                pub const VALUE: u32 = 128u32;
            }
        }
        #[allow(unused)]
        #[doc = "Information about the entry points within the module, exposed as constants and functions."]
        pub mod entry_points {
            #[allow(unused)]
            use super::*;
            pub mod main {
                pub const NAME: &'static str = "main";
                pub const STAGE: naga::ShaderStage = naga::ShaderStage::Compute;
                pub const WORKGROUP_SIZE: [u32; 3] = [256u32, 1u32, 1u32];
            }
        }
        #[allow(unused)]
        #[doc = "Equivalent Rust definitions of the types defined in this module."]
        pub mod types {
            #[allow(unused, non_camel_case_types)]
            #[derive(Debug, PartialEq, Clone, encase::ShaderType,)]
            pub struct Foo {
                pub a: i32,
                pub b: glam::u32::UVec4,
                pub c: glam::u32::UVec4,
            }
            #[allow(unused, non_camel_case_types)]
            #[derive(Debug, PartialEq, Clone, encase::ShaderType,)]
            pub struct Bar {
                pub size: u32,
                pub elements: [glam::bool::BVec2; 128u32 as usize],
                #[size(runtime)]
                pub foos: Vec<Foo>,
            }
        }
        #[allow(unused)]
        use types::*;
    };

    assert_eq!(tokens.to_string(), expected.to_string());
}
