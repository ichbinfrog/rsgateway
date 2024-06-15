use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{parenthesized, spanned::Spanned, Error, Expr, Field, FieldsNamed};

pub struct Config {
    pub condition: Option<Expr>
}

impl<'a> TryFrom<&'a Field> for Config {
    type Error = Error;

    fn try_from(f: &'a Field) -> Result<Self, Self::Error> {
        let mut config = Config {
            condition: None,
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
}
