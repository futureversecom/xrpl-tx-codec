//! XRPL codec primitive types

use crate::{
    field::{Account, SignerWeight},
    traits::BinarySerialize,
    Vec,
};

pub const ACCOUNT_ID_TYPE_CODE: u16 = 8;

#[derive(Debug, Clone)]
pub struct NotPresentType;
impl BinarySerialize for NotPresentType {
    fn binary_serialize_to(&self, _buf: &mut Vec<u8>, _for_signing: bool) {}
}

#[derive(Debug, Clone)]
pub struct UInt16Type(pub u16);

impl BinarySerialize for UInt16Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        self.0.binary_serialize_to(buf, for_signing)
    }
}

#[derive(Debug, Clone)]
pub struct UInt32Type(pub u32);

impl BinarySerialize for UInt32Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        self.0.binary_serialize_to(buf, for_signing)
    }
}

#[derive(Debug, Clone)]
pub struct UInt64Type(pub u64);

impl BinarySerialize for UInt64Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        self.0.binary_serialize_to(buf, for_signing)
    }
}

#[derive(Debug, Clone)]
pub struct Hash160Type(pub [u8; 20]);
impl BinarySerialize for Hash160Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(self.0.as_slice());
    }
}

#[derive(Debug, Clone)]
pub struct Hash256Type(pub [u8; 32]);
impl BinarySerialize for Hash256Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(self.0.as_slice());
    }
}

#[derive(Debug, Clone)]
pub struct AccountIdType(pub [u8; 20]);
impl BinarySerialize for AccountIdType {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(self.0.as_slice());
    }
}

#[derive(Default, Debug, Clone)]
pub struct BlobType(pub Vec<u8>);

impl BinarySerialize for BlobType {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(self.0.as_slice());
    }
}

/// Current
///ly supporting native XRP amounts only
#[derive(Debug, Clone)]
pub struct AmountType(pub u64);
impl BinarySerialize for AmountType {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        // https://xrpl.org/serialization.html#amount-fields
        buf.extend_from_slice((self.0 | 0x4000000000000000).to_be_bytes().as_slice());
    }
}

// TODO(surangap) - https://github.com/futureversecom/xrpl-tx-codec/issues/7
#[derive(Debug, Clone)]
pub struct SignerEntryType(pub Account, pub SignerWeight);
impl BinarySerialize for SignerEntryType {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        // call in canonical order
        self.1.binary_serialize_to(buf, _for_signing);
        self.0.binary_serialize_to(buf, _for_signing);

        // Append the Object end here. Ref -> https://xrpl.org/serialization.html#object-fields
        buf.push(0xe1);
    }
}

#[derive(Debug, Clone)]
pub struct STArrayType<T>(pub Vec<T>);
impl<T: BinarySerialize> BinarySerialize for STArrayType<T> {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        // no order, serialize the way it is. Ref -> https://xrpl.org/serialization.html#array-fields
        for item in &self.0 {
            item.binary_serialize_to(buf, _for_signing);
        }
        // Append the array end here. Ref -> https://xrpl.org/serialization.html#array-fields
        buf.push(0xf1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{Account, SignerEntry, SignerWeight};

    #[test]
    #[allow(non_snake_case)]
    fn test_SignerEntryType() {
        let signer_entry_type = SignerEntryType(
            Account(AccountIdType([1_u8; 20])),
            SignerWeight(UInt16Type(1_u16)),
        );
        let buf = signer_entry_type.binary_serialize(true);
        // let signer_entry_field_id: u8 = 0xEB; // Typecode(14) | FieldCode(11) = 0xEB
        let account_field_id: u8 = 0x81; // Typecode(8) | Fieldcode(1) = 0x81(129)
        let signer_weight_field_id: u8 = 0x13; // Typecode(1) | Fieldcode(3) = 0x13(19)
        let account_field_vl: u8 = 0x14; // https://xrpl.org/serialization.html#accountid-fields
        let st_object_end: u8 = 0xe1; // https://xrpl.org/serialization.html#object-fields
                                      // construct expected buffer
        let mut expected_buf = Vec::<u8>::default();
        expected_buf.extend_from_slice(&[signer_weight_field_id]); // SignerWeight comes first in the canonical order
        expected_buf.extend_from_slice(&1_u16.to_be_bytes());
        expected_buf.extend_from_slice(&[account_field_id]);
        expected_buf.extend_from_slice(&[account_field_vl]);
        expected_buf.extend_from_slice(&[1_u8; 20]);
        expected_buf.extend_from_slice(&[st_object_end]);

        assert_eq!(buf, expected_buf);
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_STArrayType() {
        // use SignerEntry
        let mut signer_entries = Vec::<SignerEntry>::default();
        for i in 1..=2 {
            signer_entries.push(SignerEntry(SignerEntryType(
                Account(AccountIdType([i as u8; 20])),
                SignerWeight(UInt16Type(i as u16)),
            )));
        }
        let st_array_type = STArrayType(signer_entries);

        let buf = st_array_type.binary_serialize(true);
        let signer_entry_field_id: u8 = 0xEB; // Typecode(14) | FieldCode(11) = 0xEB
        let account_field_id: u8 = 0x81; // Typecode(8) | Fieldcode(1) = 0x81(129)
        let signer_weight_field_id: u8 = 0x13; // Typecode(1) | Fieldcode(3) = 0x13(19)
        let account_field_vl: u8 = 0x14; // https://xrpl.org/serialization.html#accountid-fields
        let st_object_end: u8 = 0xe1; // https://xrpl.org/serialization.html#object-fields

        // let's construct the expected buffer -> https://xrpl.org/serialization.html#array-fields
        let mut expected_buf = Vec::<u8>::default();
        for i in 1..=2 {
            expected_buf.extend_from_slice(&[signer_entry_field_id]); // SignerEntry field ID
            expected_buf.extend_from_slice(&[signer_weight_field_id]); // SignerWeight comes first in the canonical order
            expected_buf.extend_from_slice(&(i as u16).to_be_bytes());
            expected_buf.extend_from_slice(&[account_field_id]);
            expected_buf.extend_from_slice(&[account_field_vl]);
            expected_buf.extend_from_slice(&[i as u8; 20]);
            expected_buf.extend_from_slice(&[st_object_end]);
        }
        expected_buf.extend_from_slice(&[0xf1]); // STArray end 0xf1

        assert_eq!(buf, expected_buf);
    }
}
