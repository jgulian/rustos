extern crate proc_macro;

use proc_macro::{Ident, TokenStream};
use std::str::FromStr;

use quote::{format_ident, quote, ToTokens};
use syn::{Data, DeriveInput, Field, Fields, Lit, LitInt, Meta, MetaList, NestedMeta, parse_macro_input, Type};
use synstructure::macros::TokenStream2;

#[proc_macro_derive(Formatted, attributes(endianness, padding, test))]
pub fn derive_formatted(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let impl_formatted = formatted_generator(&derive_input.ident, &derive_input.data);
    proc_macro::TokenStream::from(impl_formatted)
}

#[derive(Debug)]
enum Endianness { Native, Little, Big }

#[derive(Debug)]
struct FieldSetting {
    endianness: Option<Endianness>,
    padding: Option<usize>,
}

impl Default for FieldSetting {
    fn default() -> Self {
        Self { endianness: None, padding: None }
    }
}

fn parse_fields(fields: &mut dyn Iterator<Item=&Field>) -> Vec<(Field, FieldSetting)> {
    fields.map(|field| {
        let settings = field.attrs.iter().filter_map(|attribute| {
            let meta = attribute.parse_meta().ok()?;
            match meta {
                Meta::List(meta_list) => Some(meta_list),
                _ => None,
            }
        }).fold(FieldSetting::default(),
                |mut prev, MetaList { path, paren_token: _, nested }| {
                    if nested.len() != 1 {
                        return prev;
                    }

                    if path.is_ident("endianness") {
                        let nested_meta = match nested.first() {
                            None => panic!("Nested field doesn't have enough metas"),
                            Some(nested_meta) => nested_meta,
                        };
                        if prev.endianness.is_some() {
                            panic!("Field has endianness set twice")
                        }
                        match nested_meta {
                            NestedMeta::Meta(meta) => {
                                let endianness = meta.path().get_ident()
                                    .expect("Unknown setting for endianness").to_string();
                                match endianness.to_ascii_lowercase().as_str() {
                                    "native" => prev.endianness = Some(Endianness::Native),
                                    "little" => prev.endianness = Some(Endianness::Little),
                                    "big" => prev.endianness = Some(Endianness::Big),
                                    _ => panic!("Unknown endianness setting"),
                                }
                            }
                            NestedMeta::Lit(_) => {
                                panic!("Lit is not supported for padding; use Meta")
                            }
                        }
                    } else if path.is_ident("padding") {
                        let nested_meta = match nested.first() {
                            None => panic!("Nested field doesn't have enough metas"),
                            Some(nested_meta) => nested_meta,
                        };
                        if prev.padding.is_some() {
                            panic!("Field has padding set twice")
                        }
                        match nested_meta {
                            NestedMeta::Meta(_) => {
                                panic!("Meta is not supported for padding; use Lit")
                            }
                            NestedMeta::Lit(lit) => {
                                match lit {
                                    Lit::Int(i) =>
                                        prev.padding = Some(i.base10_parse()
                                            .expect("unable to parse padding")),
                                    _ => panic!("unsupported lit value; use int")
                                }
                            }
                        }
                    }

                    prev
                });

        (field.clone(), settings)
    }).collect()
}

fn generate_result_for_reads(
    data_ident: &proc_macro2::Ident,
    fields: Vec<(Field, FieldSetting)>,
) -> proc_macro2::TokenStream {
    let mut result = quote!("{}{", data_ident);
    fields.iter().for_each(|(field, _)|
        result.extend(quote!("{},", field.ident.as_ref().expect("Field should have ident"))));
    result.extend(quote!("}"));
    result
}

fn generate_impls(
    data_ident: &proc_macro2::Ident,
    fields: Vec<(Field, FieldSetting)>,
) -> proc_macro2::TokenStream {
    let mut read_only = quote!(use shim::io::Read;);
    let mut read_seek = quote!(use shim::io::{Read, Seek, SeekFrom};);
    let mut write_only = quote!(use shim::io::Write;);
    let mut write_seek = quote!(use shim::io::{Seek, SeekFrom, Write});

    fields.iter().for_each(|(field, settings)| {
        let field_ident = field.ident.as_ref().expect("field has no ident").clone();

        if let Some(padding) = settings.padding {
            let field_name_pad = format_ident!("{}_pad", field_ident);
            read_only.extend(quote! {
                let #field_name_pad = [0u8; #padding];
                stream.read_all(&#field_name_pad)?;
            });
            read_seek.extend(quote! {
                stream.seek(SeekFrom::Current(#padding))?;
            });
            write_only.extend(quote! {
                let #field_name_pad = [0u8; #padding];
                stream.write_all(&#field_name_pad)?;
            });
            write_seek.extend(quote! {
                stream.seek(SeekFrom::Current(#padding))?;
            });
        }

        let field_name_data = format_ident!("{}_data", field_ident);
        let endianness_tokens = match settings.endianness.as_ref()
            .unwrap_or(&Endianness::Native) {
            Endianness::Native => format_ident!("ne"),
            Endianness::Little => format_ident!("le"),
            Endianness::Big => format_ident!("be"),
        };

        let (type_size_wrapped, type_name) = match &field.ty {
            Type::Path(path) => {
                let type_name = path.path.get_ident().expect("Unspported type").to_string();
                match type_name.as_str() {
                    "u8" => (Some(1), format_ident!("u8")),
                    "u16" => (Some(2), format_ident!("u16")),
                    "u32" => (Some(4), format_ident!("u32")),
                    "u64" => (Some(8), format_ident!("u64")),
                    "u128" => (Some(16), format_ident!("u128")),
                    "i8" => (Some(1), format_ident!("i8")),
                    "i16" => (Some(2), format_ident!("i16")),
                    "i32" => (Some(4), format_ident!("i32")),
                    "i64" => (Some(8), format_ident!("i64")),
                    "i128" => (Some(16), format_ident!("i128")),
                    "f32" | "f64" | "usize" | "isize" => panic!("Attempted use of unsupported type"),
                    _ => (None, path.path.get_ident().expect("path should have ident").clone()),
                }
            }
            Type::Array(_) => panic!("Array type is not yet supported"),
            Type::Tuple(_) => panic!("Tuple type is not yet supported"),
            _ => panic!("Attempted use of unsupported type class"),
        };

        match type_size_wrapped {
            None => {
                read_only.extend(quote! {
                    let #field_ident = #type_name::load_readable(stream)?;

                });
                read_seek.extend(quote! {
                    let #field_ident = #type_name::load_readable_seekable(stream)?;

                });
                write_only.extend(quote! {
                    let #field_ident = #type_name::save_writable(stream)?;

                });
                write_seek.extend(quote! {
                    let #field_ident = #type_name::save_writable_seekable(stream)?;

                });
            }
            Some(size) => {
                let size_formatted = TokenStream2::from_str(format!("{}", size).as_str())
                    .expect("should be able to format number");
                let from_bytes = format_ident!("from_{}_bytes", endianness_tokens);
                let read_tokens = quote! {
                    let #field_name_data = [0u8; #size_formatted];
                    stream.read_all(& #field_name_data );
                    let #field_ident = #type_name::#from_bytes (& #field_name_data );
                };
                read_only.extend(read_tokens.clone());
                read_seek.extend(read_tokens);
                //write_only.extend(quote! {
                //    let #field_ident = #type_name::save_writable(stream)?;
                //});
                //write_seek.extend(quote! {
                //    let #field_ident = #type_name::save_writable_seekable(stream)?;
                //});
                //read_only.extend(read_tokens);
                //read_seek.extend(read_tokens);
                //write_only.extend(write_tokens);
                //write_seek.extend(write_tokens);
            }
        }
    });

    let read_result = generate_result_for_reads(data_ident, fields);

    //let read_result = quote!("Ok( {} )", generate_result_for_reads(data_ident, fields));

    quote! {
        impl formatted::Formatted for #data_ident {
            fn load_readable<T: shim::io::Read>(stream: &mut T) -> Result<Self> {
                #read_only
                //Ok( #read_result )
            }

            fn load_readable_seekable<T: shim::io::Read + shim::io::Seek>(stream: &mut T) -> Result<Self> {
                #read_seek
   //             #read_result
            }

            fn save_writable<T: shim::io::Write>(&self, stream: &mut T) -> io::Result<()> {
                #write_only
                Ok(())
            }

            fn save_writable_seekable<T: shim::io::Write + shim::io::Seek>(&self, stream: &mut T) -> io::Result<()> {
                #write_seek
                Ok(())
            }
        }
    }
}

fn formatted_generator(data_ident: &proc_macro2::Ident, data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    generate_impls(data_ident, parse_fields(&mut fields.named.iter()))
                }
                Fields::Unnamed(_) => panic!("unnamed structs are not supported for formatted derivation"),
                Fields::Unit => panic!("Unit structs are not supported for formatted derivation"),
            }
        }
        Data::Enum(_) => {
            panic!("Unions are not supported for formatted derivation");
        }
        Data::Union(_) => {
            panic!("Unions are not supported for formatted derivation");
        }
    }
}