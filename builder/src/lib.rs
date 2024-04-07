#![allow(dead_code)]

use syn::spanned::Spanned;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);

    let ident = &ast.ident;
    let builder_name = format!("{ident}Builder");
    let builder_ident = syn::Ident::new(&builder_name, ident.span());

    let data_struct = match ast.data {
        syn::Data::Struct(data) => data,
        _ => { todo!() }
    };

    let named = match data_struct.fields {
        syn::Fields::Named(fields) => fields.named,
        _ => { todo!() }
    };

    let inner_ty = |outer: &str, ty: &syn::Type| -> std::option::Option<syn::Type> {
        if let syn::Type::Path(ref ty_path) = ty {
            if ty_path.path.segments.len() == 1 && ty_path.path.segments[0].ident == outer {
                if let syn::PathArguments::AngleBracketed(ref angle_bracketed) =
                    ty_path.path.segments[0].arguments
                {
                    if let syn::GenericArgument::Type(ref unwrapped_inner_ty) =
                        angle_bracketed.args[0]
                    {
                        return std::option::Option::Some(unwrapped_inner_ty.clone());
                    }
                }
            }
        }

        std::option::Option::None
    };

    let builder_attrs = |field: &syn::Field| -> std::option::Option<syn::Attribute> {
        for attr in &field.attrs {
            if attr.path().is_ident("builder") {
                return std::option::Option::Some(attr.clone());
            }
        }

        std::option::Option::None
    };

    let nested_lit = |attr: &syn::Attribute| -> std::option::Option<String> {
        let mut inert_ident: std::option::Option<String> = std::option::Option::None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("each") {
                let value = meta.value().unwrap();
                let lit = value.parse::<syn::LitStr>().unwrap();

                inert_ident = std::option::Option::Some(lit.value());

                std::result::Result::Ok(())
            } else {
                std::result::Result::
                Err(meta.error("expected `builder(each = \"...\")`"))
            }
        }).ok()?;

        inert_ident
    };

    let builder_fields = named.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        if inner_ty("Option", ty).is_some() || builder_attrs(field).is_some() {
            return quote::quote! { #name: #ty };
        };

        quote::quote! { #name: std::option::Option<#ty> }
    });

    let builder_default_fields = named.iter().map(|field| {
        let name = &field.ident;

        if builder_attrs(field).is_some() {
            quote::quote! { #name: std::vec::Vec::new() }
        } else {
            quote::quote! { #name: None }
        }
    });

    let builder_methods = named.iter().filter_map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        if builder_attrs(field).is_none() {
            let inner_ty = inner_ty("Option", ty).unwrap_or(ty.clone());

            return std::option::Option::Some(quote::quote! {
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            });
        } else {
            let attr = builder_attrs(field).unwrap();
            let inner_ty = inner_ty("Vec", ty).unwrap_or(ty.clone());

            if let std::option::Option::Some(lit) = nested_lit(&attr) {
                let inert_ident = syn::Ident::new(&lit, attr.span());

                std::option::Option::Some(quote::quote! {
                    pub fn #inert_ident(&mut self, #inert_ident: #inner_ty) -> &mut Self {
                        self.#name.push(#inert_ident);
                        self
                    }
                })
            } else {
                std::option::Option::Some(
                    syn::Error::new_spanned(&attr.meta, "expected `builder(each = \"...\")`")
                        .to_compile_error(),
                )
            }
        }
    });

    let builder_build_fields = named.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        if builder_attrs(field).is_some() || inner_ty("Option", ty).is_some() {
            return quote::quote! { #name: self.#name.clone() };
        };

        quote::quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
        }
    });

    let quote = quote::quote! {
        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_default_fields,)*
                }
            }
        }

        pub struct #builder_ident {
            #(#builder_fields,)*
        }

        impl #builder_ident {
            pub fn build(&mut self) -> std::result::Result<#ident, std::boxed::Box<dyn std::error::Error>> {
                Ok(#ident {
                    #(#builder_build_fields),*
                })
            }

            #(#builder_methods)*
        }
    };

    proc_macro::TokenStream::from(quote)
}
