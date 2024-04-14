use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;

#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let mut output = input.clone();
    let item = syn::parse_macro_input!(input as syn::Item);

    if let Err(error) = sorted_impl(&item) {
        output.extend::<proc_macro::TokenStream>(error.to_compile_error().into())
    };

    output
}

fn sorted_impl(item: &syn::Item) -> Result<(), syn::Error> {
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

        Ok(())
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}

#[derive(Default)]
struct MatchOrder {
    errors: Vec<syn::Error>,
}

impl syn::visit_mut::VisitMut for MatchOrder {
    fn visit_expr_match_mut(&mut self, expr_match: &mut syn::ExprMatch) {
        if expr_match
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("sorted"))
        {
            expr_match
                .attrs
                .retain(|attr| !attr.path().is_ident("sorted"));

            let mut names = Vec::new();
            let mut wild_pat = None;
            for arm in expr_match.arms.iter() {
                if let Some(ref wild) = wild_pat {
                    self.errors.push(syn::Error::new_spanned(
                        &wild,
                        "wildcard pattern should come last",
                    ));
                    break;
                }

                if let Some(path) = get_arm_path(&arm.pat) {
                    let name = path_as_string(&path);

                    if names.last().map(|last| &name < last).unwrap_or(false) {
                        let next_index = names.binary_search(&name).unwrap_err();
                        let should_be = &names[next_index];

                        self.errors.push(syn::Error::new_spanned(
                            path,
                            format!("{name} should sort before {should_be}"),
                        ));
                    }

                    names.push(name);
                } else if let syn::Pat::Wild(wild) = &arm.pat {
                    wild_pat = Some(wild);

                    continue;
                } else {
                    self.errors.push(syn::Error::new_spanned(
                        &arm.pat,
                        "unsupported by #[sorted]",
                    ));

                    continue;
                };
            }
        }

        syn::visit_mut::visit_expr_match_mut(self, expr_match)
    }
}

fn path_as_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| format!("{}", quote::quote! {#s}))
        .collect::<Vec<_>>()
        .join("::")
}

fn get_arm_path(arm: &syn::Pat) -> Option<syn::Path> {
    match *arm {
        syn::Pat::Ident(syn::PatIdent { ident: ref id, .. }) => Some(id.clone().into()),
        syn::Pat::Path(ref p) => Some(p.path.clone()),
        syn::Pat::Struct(ref s) => Some(s.path.clone()),
        syn::Pat::TupleStruct(ref s) => Some(s.path.clone()),
        _ => None,
    }
}

#[proc_macro_attribute]
pub fn check(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let mut item_fn = syn::parse_macro_input!(input as syn::ItemFn);

    let mut match_order = MatchOrder::default();
    match_order.visit_item_fn_mut(&mut item_fn);

    let mut output: proc_macro2::TokenStream = quote::quote!(#item_fn).into();
    for err in match_order.errors.iter().take(1) {
        output.extend::<proc_macro2::TokenStream>(err.to_compile_error().into())
    }

    output.into()
}
