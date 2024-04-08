#[derive(Debug)]
struct SeqMacroInput {}

impl syn::parse::Parse for SeqMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _name = input.parse::<syn::Ident>()?;
        let _in = input.parse::<syn::Token![in]>()?;
        let _start_range = input.parse::<syn::LitInt>()?;
        let _range = input.parse::<syn::Token![..]>()?;
        let _inclusive = input.parse::<syn::Token![=]>().is_ok();
        let _end_range = input.parse::<syn::LitInt>()?;

        let content;
        let _braces = syn::braced!(content in input);
        let _body: proc_macro2::TokenStream = content.parse()?;

        Ok(Self {})
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = syn::parse_macro_input!(input as SeqMacroInput);

    proc_macro::TokenStream::new()
}
