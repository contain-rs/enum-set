extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::iter;

use proc_macro::TokenStream;
use syn::{Body, Ident, Variant, VariantData};
use quote::Tokens;

fn generate_enum_code(name: &Ident, variants: &[Variant]) -> Tokens {
    for (count,
         &Variant {
             ref data,
             ref discriminant,
             ..
         }) in variants.iter().enumerate()
    {
        if count == 32 {
            panic!("#[derive(CLike)] supports at most 32 variants");
        }
        if data != &VariantData::Unit {
            panic!("#[derive(CLike)] requires C style style enum");
        }
        if discriminant.is_some() {
            panic!("#[derive(CLike)] doesn't currently support discriminants");
        }
    }

    let variant = variants.iter().map(|variant| &variant.ident);
    let counter = 0..variants.len() as u32;
    let names = iter::repeat(name);

    let to_u32 = if variants.len() == 0 {
        quote! { unreachable!() }
    } else {
        quote! { *self as u32 }
    };

    quote! {
        impl ::enum_set::CLike for #name {
            unsafe fn from_u32(value: u32) -> Self {
                match value {
                    #(
                        #counter => #names::#variant,
                    )*
                    _ => unreachable!()
                }
            }
            fn to_u32(&self) -> u32 {
                #to_u32
            }
        }
    }
}

#[proc_macro_derive(CLike)]
pub fn derive_clike(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input(&input.to_string()).unwrap();
    match input.body {
        Body::Enum(ref variants) => generate_enum_code(&input.ident, variants),
        _ => panic!("#[derive(CLike)] is only defined for enums"),
    }.parse()
        .unwrap()
}
