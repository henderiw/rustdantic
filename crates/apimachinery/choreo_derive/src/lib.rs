extern crate proc_macro;

mod choreo_resource;

use proc_macro::TokenStream;

#[proc_macro_derive(ChoreoResource, attributes(choreo))]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    choreo_resource::derive(proc_macro2::TokenStream::from(input)).into()
}
