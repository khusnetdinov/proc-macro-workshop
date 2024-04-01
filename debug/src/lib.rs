use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);

    // dbg!(&ast);

    let _name = &ast.ident;
    // let data_struct = match &ast.data {
    //     syn::Data::Struct(data_struct) => data_struct,
    //     _ => abort_call_site!("expected struct"),
    // };

    // let _named = match data_struct.fields {
    //     syn::Fields::Named(fields) => fields.named,
    //     _ => abort_call_site!("expected struct with named fields"),
    // };

    let quote = quote::quote! {};

    proc_macro::TokenStream::from(quote)
}
