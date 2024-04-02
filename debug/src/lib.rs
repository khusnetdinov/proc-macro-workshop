use proc_macro_error::{abort_call_site, proc_macro_error};

#[proc_macro_error]
#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);

    // dbg!(&ast);

    let struct_ident = &ast.ident;
    let data_struct = match &ast.data {
        syn::Data::Struct(data_struct) => data_struct,
        _ => abort_call_site!("derive(CustomDebug) expected struct"),
    };

    let named = match &data_struct.fields {
        syn::Fields::Named(fields) => fields.named.clone(),
        _ => abort_call_site!("derive(CustomDebug) expected struct with named fields"),
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
                    abort_call_site!("attribute macro expected format `#[debug = \"formatter\"]`")
                }
            }
        }

        String::from("\"{}\"")
    });

    let quote = quote::quote! {
        impl std::fmt::Debug for #struct_ident {
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
