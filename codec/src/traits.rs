//! Codec traits

use crate::Vec;

/// A self-descriptive field type, wraps a primitive typed value for specific context
/// e.g. Destination vs. Account are different fields but both AccountIds types
pub trait CodecField: BinarySerialize {
    /// The XRPL field code (aka 'nth' in 'definitions.json')
    fn field_code(&self) -> u16;
    /// The XRPL type code of the field's underlying (primitive) type
    fn type_code(&self) -> u16;
    /// matches `isVLEncoded` in 'definitions.json', whether to prefix the serialized field with length
    fn is_variable_length(&self) -> bool;
    /// matches `isSerialized` in 'definitions.json', whether the type is included in serialized payloads or not
    /// if `false` the field will not be not serialized
    fn is_serialized(&self) -> bool;
    /// matches `isSigningField` in 'definitions.json', whether to include in signature or not
    fn is_signing_field(&self) -> bool;
    /// Return the inner value of the field
    fn inner(&self) -> &dyn BinarySerialize;
}

/// Converts a codec type into its constituent fields
pub trait CodecToFields {
    /// Convert `self` into canonical field order
    fn to_canonical_fields(&self) -> Vec<&dyn CodecField>;
}

pub trait BinarySerialize {
    /// Binary serialize `self` according to the XRPL codec spec into the given buffer.
    /// `for_signing` indicates whether the result is for signing or not
    fn binary_serialize_to(&self, _buf: &mut Vec<u8>, for_signing: bool);
    /// Binary serialize `self` according to the XRPL codec spec.
    /// `for_signing` indicates whether the result is for signing or not
    fn binary_serialize(&self, for_signing: bool) -> Vec<u8> {
        let mut buf = Vec::<u8>::default();
        self.binary_serialize_to(&mut buf, for_signing);

        buf
    }
}

impl BinarySerialize for u16 {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl BinarySerialize for u32 {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl BinarySerialize for u64 {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}
