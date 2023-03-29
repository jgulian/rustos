#![feature(proc_macro_quote)]

extern crate proc_macro;

use proc_macro::TokenStream;

use syn::ext::IdentExt;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use crate::generator::FormatImplGenerator;
use crate::parser::parse_fields;

mod generator;
mod parser;

#[proc_macro_derive(Format, attributes(endianness, padding))]
pub fn derive_format(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let impl_formatted = format_generator(&derive_input.ident, &derive_input.data);
    proc_macro::TokenStream::from(impl_formatted)
}

fn format_generator(
    format_type_ident: &proc_macro2::Ident,
    data: &Data,
) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let format_type = parse_fields(&mut fields.named.iter());
                let mut generator = FormatImplGenerator::new();
                generator.process_type(format_type);
                generator.generate(format_type_ident.unraw())
            }
            Fields::Unnamed(_) => panic!("unnamed structs are not supported for format derivation"),
            Fields::Unit => panic!("Unit structs are not supported for format derivation"),
        },
        Data::Enum(_) => {
            panic!("Unions are not supported for format derivation");
        }
        Data::Union(_) => {
            panic!("Unions are not supported for format derivation");
        }
    }
}
