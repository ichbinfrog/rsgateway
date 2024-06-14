use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics};

#[proc_macro_derive(Deserialize, attributes(condition))]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let deserialize = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;

                    match f.attrs.last() {
                        Some(attr) => {
                            if attr.path.is_ident("condition") {
                                match attr.parse_meta() {
                                    Ok(syn::Meta::NameValue(syn::MetaNameValue {
                                        lit, ..
                                    })) => match lit {
                                        syn::Lit::Str(ref x) => {
                                            let x = x.value();
                                            let expr: syn::Expr = syn::parse_str(&x).unwrap();
                                            return quote_spanned! {f.span() =>
                                                let mut #name = #ty::default();
                                                if #expr {
                                                    let (tmp, n) = #ty::deserialize(buf)?;
                                                    #name = tmp;
                                                    i += n;
                                                }
                                            };
                                        }
                                        _ => {}
                                    },
                                    _ => {}
                                }
                            }
                        }
                        None => {}
                    };

                    quote_spanned! {f.span()=>
                        let (#name, n) = #ty::deserialize(buf)?;
                        i += n;
                    }
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
        impl #impl_generics serialize::Deserialize for #name #ty_generics #where_clause {
            type Err = DeserializeError;
            fn deserialize(buf: &mut buffer::Buffer) -> Result<(Self, usize), Self::Err> {
                #deserialize
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
                        self.#name.serialize(buf)?
                    }
                });
                quote! {
                    Ok(0 #(+ #recurse)*)
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl #impl_generics serialize::Serialize for #name #ty_generics #where_clause {
            type Err = SerializeError;
            fn serialize(&self, buf: &mut buffer::Buffer) -> Result<usize, Self::Err> {
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