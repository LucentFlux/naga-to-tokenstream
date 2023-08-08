# Naga to TokenStream
![crates.io](https://img.shields.io/crates/v/naga-to-tokenstream.svg)
![crates.io](https://img.shields.io/crates/l/naga-to-tokenstream.svg)

This library takes a Naga module and produces a `proc_macro::TokenStream` giving information about the module. It is intended for use in other libraries which process a shader at compile time, for example in a proc macro or build script, to expose a large collection of useful information to the Rust compiler (and hence programmer) about items in the module.

# Generated Items

As an example, take the following shader, written in wgsl:

```wgsl
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
    ...
}
```

Then this crate would generate something like the following:

```rust
/// Equivalent Rust definitions of the constants defined in this module
pub mod constants {
    pub mod ELEMENTS_LENGTH {
        pub const VALUE: u32 = 128;
        pub type Ty = u32;
    }
}
/// Equivalent Rust definitions of the types defined in this module
pub mod types {
    // `encase::ShaderType` is only derived if the `encase` feature is enabled.
    #[derive(Debug, PartialEq, Hash, Clone, encase::ShaderType)] 
    pub struct Foo {
        a: i32,
        // `glam` objects are only generated if the `glam` feature is enabled.
        b: glam::u32::UVec4, 
        c: glam::u32::UVec4,
    }
    #[derive(Debug, PartialEq, Hash, Clone, encase::ShaderType)]
    pub struct Bar {
        size: u32,
        elements: [glam::bool::BVec2; 128],
        #[size(runtime)] // Only added if the `encase` feature is enabled.
        foos: Vec<Foo>,
    }
}
pub mod globals {
    /// Information about the `bar` global variable within the shader module.
    pub mod bar {
        pub const NAME: &'static str = "bar";
        pub type Ty = types::Bar;
        pub mod binding {
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;

            /// All the reuqired information that the shader doesn't contain when creating a bind group entry for this global.
            pub struct BindGroupLayoutEntryDescriptor {
                visibility: wgpu::ShaderStages,
                has_dynamic_offset: bool,
                min_binding_size: Option<std::num::NonZeroU64>,
            }
            
            /// Creates a bind group layout entry, requiring the exta information not contained in the shader.
            pub const fn create_bind_group_layout_entry(
                descriptor: BindGroupLayoutEntryDescriptor,
            ) -> wgpu::BindGroupLayoutEntry {
                wgpu::BindGroupLayoutEntry {
                    ..
                }
            }
            /// A bind group entry with sensable defaults.
            pub const DEFAULT_BIND_GROUP_LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {..};
        }
    }
}
/// Information about the entry points within the module.
pub mod entry_points {
    pub mod main {
        pub const NAME: &'static str = "main";
    }
}
/// The sourcecode for the shader, as a constant string
pub const SOURCE: &'static str = "...";
```
