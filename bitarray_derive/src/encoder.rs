use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parenthesized, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Error, Expr,
    Fields, Ident, LitInt, Type, Variant,
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
            match v.fields {
                Fields::Unit => match v.discriminant {
                    Some((_, ref expr)) => {
                        if let Expr::Lit(ref expr_lit) = expr {
                            if let syn::Lit::Int(ref int) = expr_lit.lit {
                                let ident = &v.ident;
                                match repr_str.as_str() {
                                    "u8" | "u16" | "u32" | "u64" | "u128" => {
                                        return quote_spanned! {v.span() =>
                                            #name::#ident => Ok(buf.push_primitive::<#repr>(#int)?),
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
                        unimplemented!("encoder not implemented");
                    }
                    _ => {
                        unimplemented!("encoder not implemented");
                    }
                },
                Fields::Named(ref fields) => {
                    // let mut arm: Option<&LitInt> = None;
                    // match v.discriminant {
                    //     Some((_, ref expr)) => {
                    //         if let Expr::Lit(ref expr_lit) = expr {
                    //             if let syn::Lit::Int(ref integer) = expr_lit.lit {
                    //                 arm = Some(integer)
                    //             }
                    //         }
                    //     }
                    //     _ => {}
                    // }
                    // if arm.is_none() {
                    //     unimplemented!("requires discriminant")
                    // }
                    // let arm = arm.unwrap();
                    let variant = &v.ident;
                    let nested = fields.named.iter().map(|f| {
                        let ident = f.ident.clone().unwrap();
                        quote_spanned! {v.span() =>
                            #ident
                        }
                    });

                    let nested_clone = nested.clone();
                    let fields_name = quote! {
                        #(#nested_clone,)*
                    };
                    return quote! {
                        #name::#variant { #fields_name } => Ok(
                            0 #(+ #nested.encode(buf)?)*
                        ),
                    };
                }
                _ => {
                    unimplemented!("encoder not implemented");
                }
            }
        });

        let res = quote! {
            match self {
                #(#recurse)*
            }
        };
        return res;
    }
}
