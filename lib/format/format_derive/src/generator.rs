use proc_macro2::{TokenStream};
use std::str::FromStr;
use proc_macro2::Ident;
use quote::{format_ident, quote};

use crate::parser::{Endianness, FieldSettings, FieldType, FormatField, FormatType};

type TokenStreamTuple = (TokenStream, TokenStream, TokenStream, TokenStream);

pub(crate) struct FormatImplGenerator {
    read: TokenStream,
    read_seek: TokenStream,
    write: TokenStream,
    write_seek: TokenStream,

    field_list: Vec<Ident>,
    pad_count: usize,
}

impl FormatImplGenerator {
    pub(crate) fn new() -> FormatImplGenerator {
        FormatImplGenerator {
            read: quote!(),
            read_seek: quote!(),
            write: quote!(),
            write_seek: quote!(),
            field_list: Vec::new(),
            pad_count: 0,
        }
    }

    pub(crate) fn process_type(&mut self, format_type: FormatType) {
        for FormatField { name, field_settings, field_type } in format_type.fields {
            let unwrapped_name = match name {
                None => panic!("format doesn't yet support unnamed types"),
                Some(unwrapped_name) => unwrapped_name,
            };
            self.field_list.push(unwrapped_name.clone());

            if let Some(pad) = field_settings.padding {
                self.pad(pad);
            }

            let (read, read_seek, write, write_seek) =
                self.add_generic_type(field_type, field_settings, unwrapped_name, true);

            self.read.extend(read);
            self.read_seek.extend(read_seek);
            self.write.extend(write);
            self.write_seek.extend(write_seek);
        }
    }

    fn pad(&mut self, pad: usize) {
        let field_name_pad = format_ident!("pad_{}", self.pad_count);
        self.pad_count += 1;

        self.read.extend(quote! {
                let mut #field_name_pad = [0u8; #pad];
                stream.read_exact(#field_name_pad .as_mut())?;
            });
        self.read_seek.extend(quote! {
                stream.seek(SeekFrom::Current(#pad as i64))?;
            });
        self.write.extend(quote! {
                let #field_name_pad = [0u8; #pad];
                stream.write_all(#field_name_pad .as_ref())?;
            });
        self.write_seek.extend(quote! {
                stream.seek(SeekFrom::Current(#pad as i64))?;
            });
    }

    fn add_generic_type(&mut self, field_type: FieldType, field_settings: FieldSettings, name: Ident, on_self: bool) -> TokenStreamTuple {
        match field_type {
            FieldType::CustomType(type_name) =>
                self.add_custom_type(name, type_name, on_self),
            FieldType::PrimitiveType(type_name, size) =>
                self.add_primitive_type(name, field_settings, type_name, size, on_self),
            FieldType::Array(sub_type, len) => {
                match sub_type.as_ref() {
                    FieldType::PrimitiveType(type_name, 1)
                    if type_name.clone().to_string().as_str() == "u8" =>
                        self.add_u8_array_type(name, len, on_self),
                    _ => self.add_generic_array_type(name, *sub_type, field_settings.endianness, len, on_self),
                }
            }
        }
    }

    fn add_custom_type(&mut self, name: Ident, type_name: Ident, on_self: bool) -> TokenStreamTuple {
        let write_name = Self::get_self_name(name.clone(), on_self);
        (
            quote! {
                    let #name = #type_name::load_readable(stream)?;
                },
            quote! {
                    let #name = #type_name::load_readable_seekable(stream)?;
                },
            quote! {
                    #write_name .save_writable(stream)?;
                },
            quote! {
                    #write_name .save_writable_seekable(stream)?;
                }
        )
    }

    fn add_primitive_type(&mut self, name: Ident, field_settings: FieldSettings, type_name: Ident, size: usize, on_self: bool) -> TokenStreamTuple {
        let name_data = format_ident!("{}_data", name);
        let size_formatted = TokenStream::from_str(format!("{}", size).as_str())
            .expect("should be able to format number");
        let (from_bytes, to_bytes) = endianness_from_to(
            field_settings.endianness.unwrap_or(Endianness::Native));

        let read_tokens = quote! {
                    let mut #name_data = [0u8; #size_formatted];
                    stream.read_exact(#name_data .as_mut())?;
                    let #name = #type_name::#from_bytes ( #name_data );
                };
        let write_name = Self::get_self_name(name, on_self);
        let write_tokens = quote! {
                    let #name_data = #write_name .#to_bytes();
                    stream.write_all(#name_data .as_ref())?;
                };

        (read_tokens.clone(), read_tokens, write_tokens.clone(), write_tokens)
    }

    fn add_u8_array_type(&mut self, name: Ident, len: usize, on_self: bool) -> TokenStreamTuple {
        let read_tokens = quote! {
                    let mut #name = [0u8; #len];
                    stream.read_exact(#name .as_mut())?;
                };
        let write_name = Self::get_self_name(name, on_self);
        let write_tokens = quote! {
                    stream.write_all(#write_name .as_ref())?;
                };

        (read_tokens.clone(), read_tokens, write_tokens.clone(), write_tokens)
    }

    fn add_generic_array_type(&mut self, name: Ident, sub_type: FieldType, endianness: Option<Endianness>, len: usize, on_self: bool) -> TokenStreamTuple {
        let sub_field_settings = FieldSettings {endianness, padding: None};
        let name_element = format_ident!("{}_element", name);

        let (read, read_seek, write, write_seek) =
            self.add_generic_type(sub_type, sub_field_settings, name_element.clone(), false);

        let mut read_list = Vec::new();
        read_list.extend((0..len).map(|_| read.clone()));
        let mut read_seek_list = Vec::new();
        read_seek_list.extend((0..len).map(|_| read_seek.clone()));

        (
            quote! {
                    let #name = [
                        #({#read_list #name_element}),*
                    ];
                },
            quote! {
                    let #name = [
                        #({#read_seek_list #name_element}),*
                    ];
                },
            quote! {
                    for #name_element in &self.#name {
                        #write
                    }
                },
            quote! {
                    for #name_element in &self.#name {
                        #write_seek
                    }
                }
        )
    }

    fn get_self_name(name: Ident, on_self: bool) -> TokenStream {
        if on_self {
            quote!{
                self.#name
            }
        } else {
            quote! {
                #name
            }
        }
    }

    pub(crate) fn generate(&mut self, format_type: Ident) -> TokenStream {
        let read = &self.read;
        let read_seek = &self.read_seek;
        let write = &self.write;
        let write_seek = &self.write_seek;
        let field_list = &self.field_list;

        quote! {
        impl Format for #format_type {
            fn load_readable<T: Read>(stream: &mut T) -> Result<Self> {
                #read
                Ok(Self { #(#field_list),* })
            }

            fn load_readable_seekable<T: Read + Seek>(stream: &mut T) -> Result<Self> {
                #read_seek
                Ok(Self { #(#field_list),* })
            }

            fn save_writable<T: Write>(&self, stream: &mut T) -> Result<()> {
                #write
                Ok(())
            }

            fn save_writable_seekable<T: Write + Seek>(&self, stream: &mut T) -> Result<()> {
                #write_seek
                Ok(())
            }
        }
        }
    }
}

fn endianness_from_to(endianness: Endianness) -> (Ident, Ident) {
    let endianness_ident = match endianness {
        Endianness::Native => format_ident!("ne"),
        Endianness::Little => format_ident!("le"),
        Endianness::Big => format_ident!("be"),
    };

    (
        format_ident!("from_{}_bytes", endianness_ident),
        format_ident!("to_{}_bytes", endianness_ident)
    )
}