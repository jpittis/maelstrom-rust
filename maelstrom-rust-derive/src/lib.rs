use convert_case::{Case, Casing};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Payload)]
pub fn derive_payload(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let tag = ident
        .to_string()
        .from_case(Case::Pascal)
        .to_case(Case::Snake);
    let output = quote! {
        impl Payload for #ident {
            fn tag() -> &'static str {
                #tag
            }
        }
    };
    TokenStream::from(output)
}
