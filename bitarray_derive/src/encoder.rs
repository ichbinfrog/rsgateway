use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parenthesized, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Error, Expr,
    Fields, Ident, Type, Variant,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub repr: Type,
}

impl TryFrom<Vec<Attribute>> for Config {
    type Error = Error;

    fn try_from(attrs: Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut repr: Option<Type> = None;

        for attr in attrs.iter() {
            if attr.path().is_ident("bitarray") {
                match attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("repr") {
                        let content;
                        parenthesized!(content in meta.input);
                        repr = Some(content.parse()?);
                    }
                    Ok(())
                }) {
                    Err(e) => return Err(Error::new(attr.span(), e)),
                    _ => break,
                }
            }
        }
        if repr.is_none() {
            panic!("Encoder with enums require explicit repr definition")
        };
        let repr = repr.unwrap();
        Ok(Config { repr })
    }
}

impl Config {
    pub fn generate_unit(
        &self,
        variants: &Punctuated<Variant, Comma>,
        name: &Ident,
    ) -> TokenStream {
        let repr = &self.repr;
        let repr_str = repr.clone().into_token_stream().to_string();

        let recurse = variants.iter().map(|v| {
            if let Fields::Unit = v.fields {
                match v.discriminant {
                    Some((_, ref expr)) => {
                        if let Expr::Lit(ref expr_lit) = expr {
                            if let syn::Lit::Int(ref int) = expr_lit.lit {
                                let ident = &v.ident;
                                match repr_str.as_str() {
                                    "u8" | "u16" | "u32" | "u64" | "u128" => {
                                        return quote_spanned! {v.span() =>
                                            #name::#ident => #int,
                                        }
                                    }
                                    s if s.starts_with("u") => {
                                        return quote_spanned! {v.span() =>
                                            #name::#ident => arbitrary_int::#repr::new(#int),
                                        }
                                    }
                                    _ => {
                                        unimplemented!(
                                            "only unsigned representations of enums are available"
                                        )
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            unimplemented!("derive not implemented")
        });

        quote! {
            let res: #repr = match self {
                #(#recurse)*
            };
            Ok(buf.push_primitive::<#repr>(res)?)
        }
    }
}
