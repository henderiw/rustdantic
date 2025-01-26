extern crate proc_macro;

mod rules;
mod validate;

use proc_macro::TokenStream;

#[proc_macro_derive(Validate, attributes(cvalidate))]
pub fn derive_default(input: TokenStream) -> TokenStream {
    validate::derive(proc_macro2::TokenStream::from(input)).into()
}