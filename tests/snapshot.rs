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

    insta::assert_snapshot!(prettyplease::unparse(&syn::parse2(tokens).unwrap()));
}
