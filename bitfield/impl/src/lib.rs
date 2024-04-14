use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;

    unimplemented!()
}

#[proc_macro]
pub fn bit_specifiers(_: TokenStream) -> TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    let bits = 1usize..=64;
    let specifiers = bits.map(|bit| {
        let ident = syn::Ident::new(&format!("B{}", bit), proc_macro2::Span::call_site());
        let size_type = size_to_type(bit);

        quote::quote! {
            pub enum #ident {}
            impl Specifier for #ident {
                const BITS: usize = #bit;
                type IntType = #size_type;
                type Interface = #size_type;

                fn to_interface(int_val: Self::IntType) -> Self::Interface {
                    int_val as Self::Interface
                }
            }
        }
    });

    output.extend(specifiers);
    output.into()
}

fn size_to_type(bits: usize) -> proc_macro2::TokenStream {
    match bits {
        1..=8 => quote::quote!(u8),
        9..=16 => quote::quote!(u16),
        17..=32 => quote::quote!(u32),
        33..=64 => quote::quote!(u64),
        65..=128 => quote::quote!(u128),
        _ => unreachable!(),
    }
}
