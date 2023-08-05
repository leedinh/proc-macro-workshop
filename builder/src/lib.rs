extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    eprintln!("ast: {:#?}", ast);
    let name = &ast.ident;
    let bname = format!("{}Builder", name);
    let bident = syn::Ident::new(&bname, name.span());
    eprintln!("bident: {:?}", bident);
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!()
    };
    let optionzed = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_inner_type(ty).is_some() {
            quote! {
                #name: #ty
            }
        } else {
            quote! {
                #name: std::option::Option<#ty>
            }
        }
    });

    let methods = fields.iter().map(|f| {
        // if !f.attrs.is_empty() {
        //     eprintln!("GGGGattrs: {:#?}", f.attrs);
        // }
        // quote! {
        //     /* ... */
        // }
        let name = &f.ident;
        let ty = &f.ty;
        eprintln!("GGGGMETHODty: {:#?}", ty_inner_type(ty));
        let (arg_type, value) = if let Some(inner_typ) = ty_inner_type(ty) {
            (inner_typ, quote! {std::option::Option::Some(#name) })
        } else {
            (ty, quote! {std::option::Option::Some(#name) })
        };
        quote! {
            pub fn #name(&mut self, #name: #arg_type) -> &mut Self {
                self.#name = #value;
                self
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        eprintln!("GGGGty: {:#?}", ty_inner_type(ty));
        if ty_inner_type(ty).is_some() {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
            }
        }
    });

    let init_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: None }
    });

    let expanded = quote! {
        pub struct #bident {
            #(#optionzed,)*

        }
        impl #name {
            fn builder() -> #bident {
                #bident {
                    #(#init_fields,)*
                }
            }
        }

        impl #bident {
            #(#methods)*

            fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
    };

    expanded.into()
}

fn ty_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    match ty {
        syn::Type::Path(syn::TypePath { qself: None, path }) => {
            let seg = path.segments.last()?;
            if path.segments.len() != 1 || path.segments[0].ident != "Option" {
                return None;
            }
            let args = &seg.arguments;
            match args {
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) => {
                    let arg = args.first()?;
                    match arg {
                        syn::GenericArgument::Type(ty) => Some(ty),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}
