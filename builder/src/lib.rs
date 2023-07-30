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
        quote! {
            #name: Option<#ty>,
        }
    });

    let methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
        }
    });

    let init_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: None }
    });

    let expanded = quote! {
        pub struct #bident {
            #(#optionzed)*

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
