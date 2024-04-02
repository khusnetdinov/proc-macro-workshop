#[proc_macro_error::proc_macro_error]
#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);

    // dbg!(&ast);

    let struct_ident = &ast.ident;
    let data_struct = match &ast.data {
        syn::Data::Struct(data_struct) => data_struct,
        _ => proc_macro_error::abort_call_site!("derive(CustomDebug) expected struct"),
    };

    let named = match &data_struct.fields {
        syn::Fields::Named(fields) => fields.named.clone(),
        _ => proc_macro_error::abort_call_site!(
            "derive(CustomDebug) expected struct with named fields"
        ),
    };

    let struct_fields_names = named.iter().map(|field: &syn::Field| &field.ident);

    let formatter_fields_names = named.iter().map(|field: &syn::Field| {
        for attr in &field.attrs {
            if attr.path().is_ident("debug") {
                if let syn::Meta::NameValue(ref meta, ..) = attr.meta {
                    if let syn::Expr::Lit(ref expr) = meta.value {
                        if let syn::Lit::Str(ref lit_str) = expr.lit {
                            return lit_str.value();
                        }
                    }
                } else {
                    proc_macro_error::abort_call_site!(
                        "attribute macro expected format `#[debug = \"formatter\"]`"
                    )
                }
            }
        }

        String::from("{:?}")
    });

    fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
        for param in &mut generics.params {
            if let syn::GenericParam::Type(ref mut type_param) = *param {
                type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
            }
        }

        generics
    }

    let generics = add_trait_bounds(ast.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let quote = quote::quote! {
        impl #impl_generics std::fmt::Debug for #struct_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#struct_ident))
                    #(.field(
                        stringify!(#struct_fields_names),
                        &format_args!(#formatter_fields_names, &self.#struct_fields_names)
                    ))*
                    .finish()
            }
        }
    };

    proc_macro::TokenStream::from(quote)
}
