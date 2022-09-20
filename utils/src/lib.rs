//! Utilities for XRPL codec
//!
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use serde_json::Value;
use std::collections::HashMap;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// XRPL codec definitions file
/// https://github.com/XRPLF/xrpl.js/blob/8a9a9bcc28ace65cde46eed5010eb8927374a736/packages/ripple-binary-codec/src/enums/definitions.json
static DEFINITIONS: &str = include_str!("../res/definitions.json");

lazy_static::lazy_static! {
    /// XRPL codec definitions parsed
    static ref DEFINITIONS_JSON: serde_json::Value = serde_json::from_str(DEFINITIONS).expect("JSON was not well-formatted");
}

lazy_static::lazy_static! {
    /// XRPL codec fields
    static ref FIELDS: HashMap<&'static str, FieldMetadata> = {
        let mut f = HashMap::new();

        let fields = if let Value::Array(fields) = &DEFINITIONS_JSON["FIELDS"] {
            fields
        } else {
            panic!("invalid fields in definitions.json");
        };

        for field in fields {
            if let [
                Value::String(field_name),
                Value::Object(field_metadata)
            ] = field.as_array().expect("field is a kv tuple").as_slice() {
                let field_type_key =
                if let Value::String(field_type) = &field_metadata["type"] {
                    field_type
                } else {
                    panic!("invalid field type in definitions.json");
                };

                let field_code: u16 = if let Value::Number(n) = &field_metadata["nth"] {
                    n.as_u64().unwrap_or(0) as u16
                } else {
                    panic!("invalid field code in definitions.json");
                };

                let type_code: u16 = if let Value::Number(n) = &DEFINITIONS_JSON["TYPES"][field_type_key] {
                    n.as_u64().unwrap_or(0) as u16
                } else {
                    panic!("invalid type code in definitions.json");
                };

                let is_vl_encoded = if let Value::Bool(b) = field_metadata["isVLEncoded"] {
                    b
                } else {
                    panic!("invalid bool value in definitions.json");
                };
                let is_serialized = if let Value::Bool(b) = field_metadata["isSerialized"] {
                    b
                } else {
                    panic!("invalid bool value in definitions.json");
                };
                let is_signing_field =
                    if let Value::Bool(b) = field_metadata["isSigningField"] {
                        b
                    } else {
                        panic!("invalid bool value in definitions.json");
                    };

                let m = FieldMetadata {
                    type_code,
                    field_code,
                    is_vl_encoded,
                    is_serialized,
                    is_signing_field,
                };

               f.insert(field_name.as_str(), m);
            } else {
                panic!("invalid field in definitions.json");
            }
        }
        f
    };
}

#[derive(Debug)]
/// Metadata about codec field
struct FieldMetadata {
    pub field_code: u16,
    pub type_code: u16,
    pub is_serialized: bool,
    pub is_vl_encoded: bool,
    pub is_signing_field: bool,
}

#[proc_macro_derive(Field)]
pub fn derive_macro_field(input: TokenStream) -> TokenStream {
    self::derive_proc_macro_impl(input)
}

fn derive_proc_macro_impl(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input as DeriveInput);
    let where_clause = &generics.where_clause;
    let field_key = ident.to_string();

    let FieldMetadata {
        type_code,
        field_code,
        is_vl_encoded,
        is_serialized,
        is_signing_field,
    } = FIELDS[field_key.as_str()];

    quote! {
        impl #generics CodecField for #ident #generics #where_clause {
          fn field_code(&self) -> u16 {
            #field_code
          }
          fn type_code(&self) -> u16 {
            #type_code
          }
          fn is_variable_length(&self) -> bool {
            #is_vl_encoded
          }
          fn is_serialized(&self) -> bool {
            #is_serialized
          }
          fn is_signing_field(&self) -> bool {
            #is_signing_field
          }
          fn inner(&self) -> &dyn BinarySerialize {
            &self.0 as &dyn BinarySerialize
          }
      }
    }
    .into()
}

#[proc_macro_derive(Transaction)]
pub fn derive_macro_transaction(input: TokenStream) -> TokenStream {
    self::derive_proc_macro_impl_transaction(input)
}

fn derive_proc_macro_impl_transaction(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let where_clause = &generics.where_clause;

    let mut fields = TokenStream2::new();
    if let Data::Struct(struct_data) = data {
        // normal struct Struct{ a, b, c }
        if let Fields::Named(fields_named) = struct_data.fields {
            for field in fields_named.named {
                let field_name = field.ident.expect("field has an ident");
                fields.extend::<TokenStream2>(quote! { &self.#field_name as &dyn CodecField, });
            }
        // tuple struct Struct(a,b,c)
        } else if let Fields::Unnamed(unnamed_fields) = struct_data.fields {
            for idx in 0..unnamed_fields.unnamed.len() {
                fields.extend::<TokenStream2>(quote! { &self.#idx as &dyn CodecField, });
            }
        }
    }

    quote! {
        impl #generics CodecToFields for #ident #generics #where_clause {
            fn to_canonical_fields(&self) -> Vec<&dyn CodecField> {
                let mut fields_ = [#fields];
                // Sort in canonical order
                fields_.sort_by(|a, b| {
                    let field_order = a.field_code().cmp(&b.field_code());
                    if let std::cmp::Ordering::Equal = field_order {
                        a.type_code().cmp(&b.type_code())
                    } else {
                        field_order
                    }
                });
                fields_.to_vec()
            }
        }

        impl BinarySerialize for #ident {
            fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
                for f in self.to_canonical_fields().iter_mut() {
                    f.binary_serialize_to(buf, for_signing);
                }
            }
        }
    }
    .into()
}
