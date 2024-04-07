use syn::spanned::Spanned;

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

    fn inner_ty(ty: &syn::Type, outer: Option<&str>) -> std::option::Option<syn::Type> {
        if let syn::Type::Path(ref ty_path) = ty {
            if ty_path.path.segments.len() != 1 {
                return std::option::Option::None;
            }

            if let Some(outer_ty) = outer {
                if ty_path.path.segments[0].ident != outer_ty {
                    return std::option::Option::None;
                }
            }

            if let syn::PathArguments::AngleBracketed(ref angle_bracketed) =
                ty_path.path.segments[0].arguments
            {
                if let syn::GenericArgument::Type(ref unwrapped_inner_ty) = angle_bracketed.args[0]
                {
                    return std::option::Option::Some(unwrapped_inner_ty.clone());
                }
            }
        }

        std::option::Option::None
    }

    fn associated_ty(ty: &syn::Type, generic_types: &[&syn::Ident]) -> Option<syn::TypePath> {
        if let Some(inner_ty) = inner_ty(ty, None) {
            return associated_ty(&inner_ty, &generic_types).clone();
        }

        if let syn::Type::Path(type_path) = ty {
            if type_path.path.segments.len() < 2 {
                return None;
            }

            let type_ident = &type_path.path.segments[0].ident;
            if generic_types.contains(&type_ident) {
                return Some(type_path.clone());
            }
        }

        None
    }

    fn add_impl_generics_bounds(
        mut generics: syn::Generics,
        phantom_data_idents: &Vec<syn::Ident>,
        associated_types: &Vec<syn::TypePath>,
        predicates: &Vec<Option<syn::WherePredicate>>,
    ) -> syn::Generics {
        let associated_types_idents = associated_types
            .iter()
            .map(|ty| &ty.path.segments[0].ident)
            .collect::<Vec<_>>();

        if predicates.is_empty() {
            for param in &mut generics.params {
                if let syn::GenericParam::Type(ref mut type_param) = *param {
                    if phantom_data_idents.contains(&type_param.ident) {
                        continue;
                    }

                    if associated_types_idents.contains(&&type_param.ident) {
                        continue;
                    }

                    type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
                }
            }

            let where_clause = generics.make_where_clause();
            for associated_type in associated_types {
                where_clause
                    .predicates
                    .push(syn::parse_quote!(#associated_type : ::std::fmt::Debug))
            }
        } else {
            let predicate = predicates.first().unwrap();

            if let Some(bound) = predicate {
                let where_clause = generics.make_where_clause();
                where_clause.predicates.push(bound.clone());
            }
        }

        generics
    }

    fn debug_attribute(attr: &syn::Attribute) -> Option<String> {
        if attr.path().is_ident("debug") {
            if let Some(syn::MetaNameValue { path, value, .. }) =
                attr.parse_args::<syn::MetaNameValue>().ok()
            {
                if path.is_ident("bound") {
                    let result = match value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) => Ok(lit_str.value()),
                        _ => Err(syn::Error::new(
                            value.span(),
                            "value of debug is not string",
                        )),
                    };

                    return result.ok();
                };
            }
        }

        None
    }

    let phantom_data_inner_types = named
        .iter()
        .filter_map(|field: &syn::Field| {
            let inner_ty = inner_ty(&field.ty, Some("PhantomData"));

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
        .collect::<Vec<_>>();

    let generic_types = &ast
        .generics
        .type_params()
        .map(|t| &t.ident)
        .collect::<Vec<&syn::Ident>>();

    let associated_types = named
        .iter()
        .filter_map(|field: &syn::Field| associated_ty(&field.ty, &generic_types))
        .collect::<Vec<syn::TypePath>>();

    let predicates = ast
        .attrs
        .iter()
        .map(|attr: &syn::Attribute| {
            if let Some(string) = debug_attribute(&attr) {
                return syn::parse_str::<syn::WherePredicate>(&string).ok();
            }

            None
        })
        .collect::<Vec<_>>();

    let generics = add_impl_generics_bounds(
        ast.generics,
        &phantom_data_idents,
        &associated_types,
        &predicates,
    );
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
