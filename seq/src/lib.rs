#[derive(Debug)]
struct SeqMacroInput {}

impl syn::parse::Parse for SeqMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _name = input.parse::<syn::Ident>()?;
        let _in = input.parse::<syn::Token![in]>()?;
        let _start_range = input.parse::<syn::LitInt>()?;
        let _range = input.parse::<syn::Token![..]>()?;
        let _end_range = input.parse::<syn::LitInt>()?;
        let _body = input.parse::<proc_macro2::TokenStream>()?;

        Ok(Self {})
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = syn::parse_macro_input!(input as SeqMacroInput);

    proc_macro::TokenStream::new()
}
