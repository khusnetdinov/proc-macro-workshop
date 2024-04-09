extern crate proc_macro;

#[derive(Debug)]
#[allow(dead_code)]
struct SeqMacroInput {
    ident: syn::Ident,
    range: std::ops::Range<i64>,
    body: proc_macro2::TokenStream,
}

impl syn::parse::Parse for SeqMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let _ = input.parse::<syn::Token![in]>()?;

        let start_range = input.parse::<syn::LitInt>()?.base10_parse::<i64>()?;
        let _ = input.parse::<syn::Token![..]>()?;
        let inclusive = input.parse::<syn::Token![=]>().is_ok();
        let end_range = input.parse::<syn::LitInt>()?.base10_parse::<i64>()?;
        let range = if inclusive {
            start_range..(end_range + 1)
        } else {
            start_range..end_range
        };

        let content;
        let _braces = syn::braced!(content in input);
        let body: proc_macro2::TokenStream = content.parse()?;

        Ok(Self { ident, range, body })
    }
}

impl Into<proc_macro::TokenStream> for SeqMacroInput {
    fn into(self) -> proc_macro::TokenStream {
        let mut quote = proc_macro2::TokenStream::new();

        for value in self.range.clone().into_iter() {
            let expanded = self.expand(self.body.clone(), value);

            quote = quote::quote! {
                #quote
                #expanded
            }
        }

        quote.into()
    }
}

impl SeqMacroInput {
    fn expand(
        &self,
        input_stream: proc_macro2::TokenStream,
        value: i64,
    ) -> proc_macro2::TokenStream {
        let mut tokens = input_stream.into_iter();
        let mut output_stream = proc_macro2::TokenStream::new();

        while let Some(token) = tokens.next() {
            let expanded = match token {
                proc_macro2::TokenTree::Group(ref group) => {
                    let mut expanded = proc_macro2::Group::new(
                        group.delimiter(),
                        self.expand(group.stream(), value),
                    );
                    expanded.set_span(group.span());

                    proc_macro2::TokenTree::Group(expanded)
                }
                proc_macro2::TokenTree::Ident(ref ident) if ident == &self.ident => {
                    let mut lit = proc_macro2::Literal::i64_unsuffixed(value);
                    lit.set_span(token.span());

                    proc_macro2::TokenTree::from(lit)
                }
                proc_macro2::TokenTree::Ident(ref prefix) => {
                    let mut look_forward_iterator = tokens.clone();

                    match (look_forward_iterator.next(), look_forward_iterator.next()) {
                        (
                            Some(proc_macro2::TokenTree::Punct(punct)),
                            Some(proc_macro2::TokenTree::Ident(ref ident)),
                        ) if punct.as_char() == '~' && ident == &self.ident => {
                            tokens.next();
                            tokens.next();

                            let concat = format!("{}{}", prefix, value);
                            let concat_ident = proc_macro2::Ident::new(&concat, token.span());

                            proc_macro2::TokenTree::from(concat_ident)
                        }
                        _ => token,
                    }
                }
                _ => token,
            };

            output_stream.extend(proc_macro2::TokenStream::from(expanded))
        }

        output_stream
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let seq_macro_input = syn::parse_macro_input!(input as SeqMacroInput);

    seq_macro_input.into()
}
