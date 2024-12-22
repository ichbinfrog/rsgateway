use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parenthesized, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Error, Expr,
    Field, Fields, Ident, Type, Variant,
};

#[derive(Debug)]
pub struct Config {
    pub condition: Option<Expr>,
    pub repr: Option<Type>,
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
        Ok(Config {
            repr,
            condition: None,
        })
    }
}

impl<'a> TryFrom<&'a Field> for Config {
    type Error = Error;

    fn try_from(f: &'a Field) -> Result<Self, Self::Error> {
        let mut config = Config {
            condition: None,
            repr: None,
        };

        for attr in f.attrs.iter() {
            if attr.path().is_ident("bitarray") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("condition") {
                        let content;
                        parenthesized!(content in meta.input);
                        config.condition = Some(content.parse()?);
                    }

                    Ok(())
                });
            }
        }

        Ok(config)
    }
}

impl Config {
    pub fn quote(&self, f: &Field) -> Result<TokenStream, Error> {
        let name = &f.ident;
        let ty = &f.ty;

        if let Some(condition) = &self.condition {
            Ok(quote_spanned! {f.span() =>
                    let mut #name = #ty::default();
                    if #condition {
                        let (tmp, n) = #ty::decode(buf)?;
                        #name = tmp;
                        i += n;
                    }
            })
        } else {
            Ok(quote_spanned! {f.span()=>
                    let (#name, n) = #ty::decode(buf)?;
                    i += n;
            })
        }
    }

    pub fn generate_unit(
        &self,
        variants: &Punctuated<Variant, Comma>,
        name: &Ident,
    ) -> TokenStream {
        let repr = self.repr.clone().unwrap();
        let repr_str = repr.clone().into_token_stream().to_string();

        let read = if repr_str.starts_with('u') {
            let n = repr_str.strip_prefix('u').unwrap();
            let n = usize::from_str_radix(n, 10).unwrap();
            let div = n / 8;

            if n % 8 == 0 {
                quote! {
                    let (kind, kind_l) = buf.read_primitive::<#repr, #div>()?;
                }
            } else {
                match n {
                    1..=7 => {
                        quote! {
                            let (kind, kind_l) = buf.read_arbitrary_u8()?;
                        }
                    }
                    9..=15 => {
                        quote! {
                            let (kind, kind_l) = buf.read_arbitrary_u16()?;
                        }
                    }
                    17..=31 => {
                        quote! {
                            let (kind, kind_l) = buf.read_arbitrary_u32()?;
                        }
                    }
                    _ => {
                        quote! {}
                    }
                }
            }
        } else {
            quote! {}
        };

        let recurse = variants.iter().map(|v| {
            match v.fields {
                Fields::Unit => match v.discriminant {
                    Some((_, ref expr)) => {
                        if let Expr::Lit(ref expr_lit) = expr {
                            if let syn::Lit::Int(ref int) = expr_lit.lit {
                                let ident = &v.ident;

                                return quote_spanned! {
                                    v.span() =>
                                        #int => Ok((#name::#ident, kind_l)),
                                };
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            unimplemented!("derive not implemented")
        });

        quote! {
            #read
            match kind {
                #(#recurse)*
                _ => {panic!()}
            }
        }
    }
}
