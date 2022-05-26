//! ## Usage
//!
//! ``` rust
//! use test_collector_derive::collect_test;
//!
//!     #[collect_test]
//!     #[test]
//!     pub fn sync_test_success() {
//!         println!("Executed sync!");
//!         assert_eq!(true, true);
//!     }
//!
//!     #[collect_test(async)]
//!     #[actix_web::test]
//!     pub async fn async_test_success() {
//!         let client = reqwest::Client::builder()
//!             .build()
//!             .expect("error during client build");
//!         let response = client.get("http://localhost:9090/").send().await;
//!         assert!(response.is_ok());
//!     }
//!
//! ```
extern crate core;

use proc_macro::{TokenStream};
use proc_macro2::Span;

use syn::{AttributeArgs, Ident, ItemFn, Lit, Meta, NestedMeta, parse_macro_input, Path};
use quote::{quote, TokenStreamExt, ToTokens};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn collect_test(args: TokenStream, input: TokenStream) -> TokenStream {
    // Read the user test
    let is_async = is_async(parse_macro_input!(args as AttributeArgs));
    let fn_user_test = parse_macro_input!(input as ItemFn);

    // Add some random to the generated function names so
    // we can support using the macro multiple times in the same file
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let test_name = fn_user_test.sig.ident.to_string();

    let wrapper_test_name = format!("{}_{}", test_name, &rand_string);

    let wrapped_test_iden = Ident::new(&wrapper_test_name, Span::call_site());

    let test_body = &fn_user_test.block;

    //Creating another function, because I was not able to find how to put exact same function to struct
    let output_fn = if is_async {
        quote! {
            inventory::submit!{
                test_collector_utils::IntegrationTestMeta::for_async_fn(
                  #test_name.to_string(),
                  Box::new(|| Box::pin(#wrapped_test_iden()))
                )
            }
            pub async fn #wrapped_test_iden() {
                #test_body
            }
        }
    } else {
        quote! {
            inventory::submit!{
                test_collector_utils::IntegrationTestMeta::for_sync_fn(
                  #test_name.to_string(),
                  #wrapped_test_iden
                )
            }

            pub fn #wrapped_test_iden() {
                #test_body
            }
        }
    };

    let mut fn_user_test = fn_user_test.to_token_stream();

    fn_user_test.append_all(output_fn);

    fn_user_test.into()
}

fn is_async(args: AttributeArgs) -> bool {
    let mut is_async = false;
    for attr in args {
        if is_async {
            break;
        }
        match attr {
            NestedMeta::Meta(meta) => {
                is_async = from_meta(meta);
            }
            _ => panic!("invalid syntax {:?}", attr.span()),
        }
    }
    return is_async;
}

fn from_meta(meta: Meta) -> bool {
    match meta {
        Meta::NameValue(name_value) => match get_key(&name_value.path).as_str() {
            "async" => async_nv(&name_value),
            _ => panic!("Unsupported key {:?}", name_value.span()),
        },
        Meta::Path(path) => match get_key(&path).as_str() {
            "async" => true,
            _ => panic!("Unsupported key {:?}", path.span()),
        },
        _ => panic!("Unsupported attribute: {:?}", meta.span()),
    }
}

fn async_nv(name_value: &syn::MetaNameValue) -> bool {
    if let Lit::Bool(bool_lit) = &name_value.lit {
        bool_lit.value()
    } else {
        panic!("Async must be LitBool or have no key {:?}", name_value.span())
    }
}

fn get_key(p: &Path) -> String {
    let mut key: Vec<String> = p
        .segments
        .iter()
        .map(|args| args.ident.to_string())
        .collect();

    key.pop().expect("Key expected")
}
