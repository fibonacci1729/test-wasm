use heck::ToKebabCase;
use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{quote, format_ident};
use wit_parser::{Resolve, UnresolvedPackage};

#[proc_macro_attribute]
pub fn test_wasm(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func_item = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func_item.sig.ident;
    let new_name = format_ident!(
        "__test_wasm_{}", 
        func_name,
    );
    let component_type = generate_world(func_name);
    let export = format!("{func_name}").to_kebab_case();
    let tokens = quote!(
        #func_item

        #component_type

        #[export_name = #export]
        #[doc(hidden)]
        #[cfg(target_arch = "wasm32")]
        unsafe extern "C" fn #new_name() {
            #func_name();
        }
    );
    tokens.into()
}

fn generate_world(ident: &syn::Ident) -> proc_macro2::TokenStream {
    let component_type_name = format_ident!("_WIT_BINDGEN_COMPONENT_TYPE_{}", format!("{ident}").to_uppercase());
    let version = env!("CARGO_PKG_VERSION");
    let world_name = format!("{ident}").to_kebab_case();
    let world_text = format!(r#"
        package test:test;

        world {world_name} {{
            export {world_name}: func();
        }}
    "#);
    let link_section = format!("component-type:test_wasm:{version}:{world_name}:encoded world");
    let mut resolve = Resolve::default();
    let unresolved = UnresolvedPackage::parse("macro-input".as_ref(), &world_text).expect("creating world");
    let package_id = resolve.push(unresolved).expect("pushing world");
    let world = resolve.select_world(package_id, Some(&world_name)).expect("selecting world");

    let component_type = wit_component::metadata::encode(
        &resolve,
        world,
        wit_component::StringEncoding::UTF8,
        None,
    ).expect("encoding component type");

    let len = component_type.len();
    let b = Literal::byte_string(&component_type);

    quote!(
        #[doc(hidden)]
        #[cfg(target_arch = "wasm32")]
        #[link_section = #link_section]
        pub static #component_type_name: [u8; #len] = *#b;
    )
}