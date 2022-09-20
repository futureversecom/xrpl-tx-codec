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

pub struct AccountIdType(pub [u8; 32]);
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
        buf.extend_from_slice((self.0 & 0x4000000000000000).to_be_bytes().as_slice());
    }
}

// Primitive types of the XRPL codec
// fn type_code_to_type(type_code: u16) -> Box<dyn BinarySerialize> {
//     let codec_type = match type_code {
//         0 => NotPresentType,
//         1 => UInt16Type,
//         2 => UInt32Type,
//         3 => UInt64Type,
//         5 => Hash256Type,
//         6 => AmountType,
//         7 => BlobType,
//         8 => AccountIdType,
//         // 14 => STObjectType,
//         // 15 => STArrayType,
//         // 16 => UInt8Type,
//         17 => Hash160Type,
//         // 18 => PathSetType,
//         // 19 => Vector256Type,
//         // 20 => UInt96Type,
//         // 21 => UInt192Type,
//         // 22 => UInt384Type,
//         // 23 => UInt512Type,
//         // 10001 => TransactionType,
//         // 10002 => LedgerEntryType,
//         // 10003 => ValidationType,
//         // 10004 => MetadataType,
//         _ => return None,
//     };

//     Some(codec_type)
// }
