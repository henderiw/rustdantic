extern crate proc_macro;

mod default;

use proc_macro::TokenStream;

#[proc_macro_derive(Default, attributes(cdefault))]
pub fn derive_default(input: TokenStream) -> TokenStream {
    default::derive(proc_macro2::TokenStream::from(input)).into()
}


