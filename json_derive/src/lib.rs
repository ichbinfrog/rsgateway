#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput};

#[proc_macro_derive(Serialize)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(ref data) => {}
        Data::Enum(ref data) => {}
        Data::Union(_) => {}
    }
    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // ...
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
