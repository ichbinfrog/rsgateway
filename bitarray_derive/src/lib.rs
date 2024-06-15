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

    // Hand the output tokens back to the compiler.
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
            // TODO: split into separate func &struct

            let mut repr = None::<syn::Type>;
            if let Some(attr) = input.attrs.last() {
                if attr.path().is_ident("bitarray") {
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("repr") {
                            let content;
                            parenthesized!(content in meta.input);
                            repr = Some(content.parse()?);
                        }
                        Ok(())
                    });
                }
            }

            if repr.is_none() {
                panic!("Serialization with enums require explicit repr definition")
            }
            let repr = repr.unwrap();

            let recurse = data.variants.iter().map(|v| {
                match v.fields {
                    Fields::Named(ref fields) => {
                        for field in fields.named.iter() {
                            eprintln!("{:?}", field.ident);
                        }
                    },
                    Fields::Unit => {
                        match &v.discriminant {
                            Some((_, expr)) => {
                                match expr {
                                    syn::Expr::Lit(lit) => {
                                        match lit.lit {
                                            syn::Lit::Int(ref i) => {
                                                let value = i.base10_parse::<usize>().unwrap();
                                                let ident = &v.ident;
                                                match repr.clone().into_token_stream().to_string().as_str() {
                                                    "u8" | "u16" | "u32" | "u64" | "u128" => {
                                                        return quote_spanned! {v.span() => 
                                                            #name::#ident => #value as #repr,
                                                        };
                                                    },
                                                    s if s.starts_with("u") => {
                                                        return quote_spanned! {v.span() => 
                                                            #name::#ident => arbitrary_int::#repr::new(#value),
                                                        };
                                                    }
                                                    _ => {
                                                        unimplemented!("only unsigned representations of enums are available")
                                                    }
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            },
                            _ => {}
                        };
                    },
                    Fields::Unnamed(_) => unimplemented!("unamed enum not implemented")
                }
                quote!{}
            });
            quote! {
                let res: #repr = match self {
                    #(#recurse)*
                };
                Ok(buf.push_primitive::<#repr>(res)?)
            }
        }
        Data::Union(ref data) => {
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

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(Serialize));
        }
    }
    generics
}
