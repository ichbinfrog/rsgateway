mod decoder;
mod encoder;

use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse_macro_input, parse_quote, Data, DeriveInput, Expr, Fields, GenericParam,
    Generics,
};

#[proc_macro_derive(Decode, attributes(bitarray))]
pub fn decoder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let decode = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let config = decoder::Config::try_from(f).unwrap();
                    config.quote(f).unwrap()
                });
                let sr = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! {f.span()=>
                        #name,
                    }
                });

                quote! {
                    let mut i = 0;
                    #(#recurse)*
                    Ok((Self {
                        #(#sr)*
                    }, i))
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl #impl_generics Decoder for #name #ty_generics #where_clause {
            fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error> {
                #decode
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Encode, attributes(bitarray))]
pub fn encoder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let write = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! {f.span()=>
                        self.#name.encode(buf)?
                    }
                });
                quote! {
                    Ok(0 #(+ #recurse)*)
                }
            }
            _ => unimplemented!(),
        },
        Data::Enum(ref data) => {
            let config = encoder::Config::try_from(input.attrs).unwrap();
            config.generate_unit(&data.variants, &name)
        }
        Data::Union(_) => {
            unimplemented!("union deserialization not implemented")
        }
    };

    let expanded = quote! {
        impl #impl_generics Encoder for #name #ty_generics #where_clause {
            fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {
                #write
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(Decoder));
        }
    }
    generics
}
