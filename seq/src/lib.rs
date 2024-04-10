extern crate proc_macro;

#[derive(Debug)]
#[allow(dead_code)]
struct SeqMacroInput {
    ident: syn::Ident,
    range: std::ops::Range<i64>,
    body: proc_macro2::TokenStream,
    repeated_expanded: bool,
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
        let _ = syn::braced!(content in input);
        let body: proc_macro2::TokenStream = content.parse()?;

        Ok(Self {
            ident,
            range,
            body,
            repeated_expanded: false,
        })
    }
}

impl Into<proc_macro::TokenStream> for SeqMacroInput {
    fn into(mut self) -> proc_macro::TokenStream {
        let quote = self.repeated_expand(self.body.clone());

        if self.repeated_expanded {
            quote.into()
        } else {
            self.range
                .clone()
                .into_iter()
                .fold(
                    proc_macro2::TokenStream::new(),
                    |quote: proc_macro2::TokenStream, value: i64| {
                        let expanded = self.expand(self.body.clone(), value);

                        quote::quote!(#quote #expanded)
                    },
                )
                .into()
        }
    }
}

impl SeqMacroInput {
    fn repeated_expand(
        &mut self,
        input_stream: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let mut tokens = input_stream.into_iter();
        let mut output_stream = proc_macro2::TokenStream::new();

        while let Some(token) = tokens.next() {
            let expanded = match token {
                proc_macro2::TokenTree::Group(ref group) => {
                    let mut expanded = proc_macro2::Group::new(
                        group.delimiter(),
                        self.repeated_expand(group.stream()),
                    );
                    expanded.set_span(group.span());

                    proc_macro2::TokenTree::Group(expanded)
                }
                proc_macro2::TokenTree::Punct(ref punct) if punct.as_char() == '#' => {
                    let mut peek = tokens.clone();
                    let pattern = (peek.next(), peek.next());

                    match pattern {
                        (
                            Some(proc_macro2::TokenTree::Group(group)),
                            Some(proc_macro2::TokenTree::Punct(ref punct)),
                        ) if group.delimiter() == proc_macro2::Delimiter::Parenthesis
                            && punct.as_char() == '*' =>
                        {
                            self.repeated_expanded = true;

                            tokens.next();
                            tokens.next();

                            let repeated = self.range.clone().into_iter().fold(
                                proc_macro2::TokenStream::new(),
                                |quoted: proc_macro2::TokenStream, value: i64| {
                                    let expanded = self.expand(group.stream(), value);

                                    quote::quote!(#quoted #expanded)
                                },
                            );

                            let mut expanded =
                                proc_macro2::Group::new(proc_macro2::Delimiter::None, repeated);
                            expanded.set_span(group.span());

                            proc_macro2::TokenTree::Group(expanded)
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
                    let mut peek = tokens.clone();
                    let pattern = (peek.next(), peek.next());

                    match pattern {
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
