#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{spanned::Spanned, Data, DeriveInput, GenericParam, Generics, Index, LitByteStr};

#[proc_macro_derive(Serialize)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                syn::Fields::Named(ref fields) => {
                    let n = fields.named.len();
                    let recurse = fields.named.iter().enumerate().map(|(i, f)| {
                        let name = &f.ident;

                        let mut res = String::new();
                        

                        let mut key = Vec::<u8>::new();
                        if i == 0 {
                            key.push(b'{');
                        }
                        if i >= 1 && i < n - 1 {
                            key.push(b',');
                        }
                        key.extend(name.clone().unwrap().to_string().as_bytes());
                        key.push(b'"');
                        key.push(b':');

                        quote_spanned! {f.span()=>
                            writer.write_all(&[#(#key,)*])?;
                            serialize::Serialize::serialize(&self.#name, writer)
                        }
                    });
                    quote! {
                        #(#recurse?;)*
                        Ok(())
                    }
                },
                syn::Fields::Unnamed(ref fields) => {
                    let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let index = Index::from(i);
                        quote_spanned! {f.span()=>
                            serialize::Serialize::serialize(&self.#index, writer)
                        }
                    });
                    quote! {
                        #(#recurse?;)*
                        Ok(())
                    }
                }
                syn::Fields::Unit => {
                    quote!(0)
                }
            }
        }
        Data::Enum(ref data) => {
            unimplemented!()
        }
        Data::Union(_) => {
            unimplemented!()
        } 
    };

    let expanded = quote! {
        impl #impl_generics serialize::Serialize for #name #ty_generics #where_clause {
            fn serialize<W>(&self, writer: &mut W) -> Result<(), SerializeError> where W: Write {
                #body
            }
        }
    };
    TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(serialize::Seralize));
        }
    }
    generics
}
