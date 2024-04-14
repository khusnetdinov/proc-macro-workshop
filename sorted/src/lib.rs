use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let item = syn::parse_macro_input!(input as syn::Item);

    sorted_impl(item)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn sorted_impl(item: syn::Item) -> Result<proc_macro2::TokenStream, syn::Error> {
    if let syn::Item::Enum(item_enum) = item {
        let mut names = Vec::new();

        for variant in item_enum.variants.iter() {
            let name = variant.ident.to_string();

            if names.last().map(|last| &name < last).unwrap_or(false) {
                let next_index = names.binary_search(&name).unwrap_err();
                let should_be = &names[next_index];

                return Err(syn::Error::new(
                    variant.span(),
                    format!("{name} should sort before {should_be}"),
                ));
            }

            names.push(name)
        }

        Ok(quote::quote! { #item_enum })
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}
