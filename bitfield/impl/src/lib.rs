#[proc_macro_attribute]
pub fn bitfield(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = proc_macro2::TokenStream::new();

    let item = syn::parse_macro_input!(input as syn::Item);
    let item_struct = match item {
        syn::Item::Struct(item_struct) => item_struct,
        _ => unimplemented!("#[bitfield] expected struct")
    };

    let ident = &item_struct.ident;
    let fields_tys = item_struct.fields.iter().map(|field| &field.ty);
    let fields_bit_size = quote::quote!(0 #(+ <#fields_tys as Specifier>::BITS)*);

    let quoted = quote::quote! {
        pub struct #ident {
            data: [u8; ( #fields_bit_size) / 8],
        }
    };

    output.extend(quoted);
    output.into()
}

#[proc_macro]
pub fn bit_specifiers(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
