use syn::WhereClause;

#[proc_macro_error::proc_macro_error]
#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);

    // dbg!(&ast);

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

    fn add_impl_generics_bounds(
        mut generics: syn::Generics,
        phantom_data_idents: &Vec<syn::Ident>,
    ) -> syn::Generics {
        for param in &mut generics.params {
            if let syn::GenericParam::Type(ref mut type_param) = *param {
                let is_present = phantom_data_idents
                    .iter()
                    .find(|&ident| *ident == type_param.ident);

                if is_present.is_none() {
                    type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
                }
            }
        }

        generics
    }


    fn add_where_clause_bounds(where_clause: Option<&WhereClause>, phantom_data_bounds: Vec<proc_macro2::TokenStream>
    ) -> proc_macro2::TokenStream {
        if phantom_data_bounds.is_empty() {
            quote::quote! { #where_clause }
        } else {
            quote::quote! { where #(#phantom_data_bounds),* }
        }
    }

    let phantom_data_types = named
        .iter()
        .filter_map(|field: &syn::Field| {
            let inner_ty = inner_ty("PhantomData", &field.ty);

            if inner_ty.is_some() {
                return Some(&field.ty);
            }

            None
        })
        .collect::<Vec<&syn::Type>>();

    let phantom_data_bounds = phantom_data_types
        .iter()
        .map(|ty: &&syn::Type| {
            quote::quote! {
                #ty: std::fmt::Debug
            }
        })
        .collect::<Vec<_>>();

    let phantom_data_inner_types = named
        .iter()
        .filter_map(|field: &syn::Field| {
            let inner_ty = inner_ty("PhantomData", &field.ty);

            if inner_ty.is_some() {
                return Some(inner_ty.unwrap());
            }

            None
        })
        .collect::<Vec<syn::Type>>();

    let phantom_data_idents = phantom_data_inner_types
        .iter()
        .filter_map(|ty: &syn::Type| {
            if let syn::Type::Path(ref ty_path) = ty {
                if ty_path.path.segments.len() == 1 {
                    return Some(ty_path.path.segments[0].ident.clone());
                }
            }

            None
        })
        .collect::<Vec<syn::Ident>>();

    let generics = add_impl_generics_bounds(ast.generics, &phantom_data_idents);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = add_where_clause_bounds(where_clause, phantom_data_bounds);

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
