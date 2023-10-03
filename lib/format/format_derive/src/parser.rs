use proc_macro2::Ident;
use syn::{Expr, Field, Lit, Meta, MetaList, NestedMeta, Type, TypeArray, TypePath};

pub(crate) struct FormatType {
    pub(crate) fields: Vec<FormatField>,
}

#[derive(Clone, Debug)]
pub(crate) struct FormatField {
    pub(crate) name: Option<Ident>,
    pub(crate) field_settings: FieldSettings,
    pub(crate) field_type: FieldType,
}

#[derive(Clone, Debug)]
pub(crate) enum FieldType {
    CustomType(Ident),
    PrimitiveType(Ident, usize),
    Array(Box<FieldType>, usize),
}

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct FieldSettings {
    pub(crate) endianness: Option<Endianness>,
    pub(crate) padding: Option<usize>,
}

#[derive(Copy, Clone, Debug)]
pub enum Endianness {
    Native,
    Little,
    Big,
}

pub(crate) fn parse_fields(fields: &mut dyn Iterator<Item = &Field>) -> FormatType {
    FormatType {
        fields: fields
            .map(|field| FormatField {
                name: field.ident.clone(),
                field_settings: parse_settings(field),
                field_type: parse_type(&field.ty),
            })
            .collect(),
    }
}

fn parse_settings(field: &Field) -> FieldSettings {
    field
        .attrs
        .iter()
        .filter_map(|attribute| match attribute.parse_meta().ok()? {
            Meta::List(meta_list) => Some(meta_list),
            _ => None,
        })
        .fold(
            FieldSettings::default(),
            |mut prev,
             MetaList {
                 path,
                 paren_token: _,
                 nested,
             }| {
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
                            let endianness = meta
                                .path()
                                .get_ident()
                                .expect("Unknown setting for endianness")
                                .to_string();
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
                        NestedMeta::Lit(lit) => match lit {
                            Lit::Int(i) => {
                                prev.padding =
                                    Some(i.base10_parse().expect("unable to parse padding"))
                            }
                            _ => panic!("unsupported lit value; use int"),
                        },
                    }
                }

                prev
            },
        )
}

fn parse_type(field_type: &Type) -> FieldType {
    match field_type {
        Type::Array(TypeArray { elem, len, .. }) => {
            let array_size: usize = if let Expr::Lit(literal) = len {
                if let Lit::Int(int) = &literal.lit {
                    int.base10_parse().expect("unable to parse array size")
                } else {
                    panic!("unknown literal type");
                }
            } else {
                panic!("unknown literal type");
            };

            FieldType::Array(Box::new(parse_type(elem)), array_size)
        }
        Type::Path(TypePath { path, .. }) => {
            let type_ident = path.get_ident().expect("Unsupported type").clone();
            let type_size = match type_ident.to_string().as_str() {
                "u8" => Some(1),
                "u16" => Some(2),
                "u32" => Some(4),
                "u64" => Some(8),
                "u128" => Some(16),
                "i8" => Some(1),
                "i16" => Some(2),
                "i32" => Some(4),
                "i64" => Some(8),
                "i128" => Some(16),
                "f32" | "f64" | "usize" | "isize" => panic!("Attempted use of unsupported type"),
                _ => None,
            };

            match type_size {
                None => FieldType::CustomType(type_ident),
                Some(size) => FieldType::PrimitiveType(type_ident, size),
            }
        }
        _ => panic!("{:?} is not currently supported", field_type),
    }
}
