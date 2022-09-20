//! XRPL codec primitive types

use crate::traits::BinarySerialize;

pub const ACCOUNT_ID_TYPE_CODE: u16 = 8;

pub struct NotPresentType;
impl BinarySerialize for NotPresentType {
    fn binary_serialize_to(&self, _buf: &mut Vec<u8>, _for_signing: bool) {}
}

pub struct UInt16Type(pub u16);

impl BinarySerialize for UInt16Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        self.0.binary_serialize_to(buf, for_signing)
    }
}

pub struct UInt32Type(pub u32);

impl BinarySerialize for UInt32Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        self.0.binary_serialize_to(buf, for_signing)
    }
}

pub struct UInt64Type(pub u64);

impl BinarySerialize for UInt64Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        self.0.binary_serialize_to(buf, for_signing)
    }
}

pub struct Hash160Type(pub [u8; 20]);
impl BinarySerialize for Hash160Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(self.0.as_slice());
    }
}

pub struct Hash256Type(pub [u8; 32]);
impl BinarySerialize for Hash256Type {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(self.0.as_slice());
    }
}

pub struct AccountIdType(pub [u8; 20]);
impl BinarySerialize for AccountIdType {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(self.0.as_slice());
    }
}

#[derive(Default)]
pub struct BlobType(pub Vec<u8>);

impl BinarySerialize for BlobType {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        buf.extend_from_slice(self.0.as_slice());
    }
}

/// Current
///ly supporting native XRP amounts only
pub struct AmountType(pub u64);
impl BinarySerialize for AmountType {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        // https://xrpl.org/serialization.html#amount-fields
        buf.extend_from_slice((self.0 | 0x4000000000000000).to_be_bytes().as_slice());
    }
}
