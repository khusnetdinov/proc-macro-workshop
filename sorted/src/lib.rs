#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let item = syn::parse_macro_input!(input as syn::Item);

    sorted_impl(item).into()
}

fn sorted_impl(item: syn::Item) -> proc_macro2::TokenStream {
    if let syn::Item::Enum(item_enum) = item {
        quote::quote! { #item_enum }
    } else {
        quote::quote! { compile_error!("expected enum or match expression"); }
    }
}
