//! XRPL codec primitive types

use crate::error::Error;
use crate::{
    field::{Account, SignerWeight},
    traits::BinarySerialize,
    Vec,
};
use alloc::format;
use alloc::string::ToString;

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

/// Currency code, ref - https://xrpl.org/docs/references/protocol/data-types/currency-formats#currency-codes
#[derive(Debug, Clone)]
pub enum CurrencyCode {
    Standard([u8; 3]),
    NonStandard([u8; 20]),
}

impl CurrencyCode {
    pub fn is_valid(&self) -> bool {
        // https://xrpl.org/docs/references/protocol/data-types/currency-formats#currency-codes
        match self {
            CurrencyCode::Standard(value) => value.ne(b"XRP"),
            CurrencyCode::NonStandard(value) => value[0] != 0x00,
        }
    }
}

impl BinarySerialize for CurrencyCode {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, _for_signing: bool) {
        // https://xrpl.org/docs/references/protocol/binary-format#currency-codes
        match self {
            CurrencyCode::NonStandard(payload) => buf.extend_from_slice(payload),
            CurrencyCode::Standard(payload) => {
                buf.extend_from_slice(&[0u8; 12]);
                buf.extend_from_slice(payload);
                buf.extend_from_slice(&[0u8; 5]);
            }
        }
    }
}

/// The value of Issued amount, ref - https://xrpl.org/docs/references/protocol/data-types/currency-formats#string-numbers
#[derive(Debug, Clone)]
pub struct IssuedValue {
    pub mantissa: i64,
    pub exponent: i8,
}

impl IssuedValue {
    /// Creates value from given mantissa and exponent. The created value will be normalized
    /// according to https://xrpl.org/docs/references/protocol/binary-format#token-amount-format. If the value
    /// cannot be represented, an error is returned.
    pub fn from_mantissa_exponent(mantissa: i64, exponent: i8) -> Result<Self, Error> {
        Self { mantissa, exponent }.normalize()
    }

    pub fn zero() -> Self {
        Self {
            mantissa: 0,
            exponent: 0,
        }
    }

    /// Normalizes value into the ranges specified in https://xrpl.org/docs/references/protocol/binary-format#token-amount-format
    fn normalize(self) -> Result<Self, Error> {
        // rippled implementation: https://github.com/seelabs/rippled/blob/cecc0ad75849a1d50cc573188ad301ca65519a5b/src/ripple/protocol/impl/IOUAmount.cpp#L38
        const MANTISSA_MIN: i64 = 1000000000000000;
        const MANTISSA_MAX: i64 = 9999999999999999;
        const EXPONENT_MIN: i8 = -96;
        const EXPONENT_MAX: i8 = 80;

        let mut exponent = self.exponent;
        let (mut mantissa, negative) = match self.mantissa {
            0 => {
                return Ok(Self::zero());
            }
            1.. => (self.mantissa, false),
            ..=-1 => (
                self.mantissa.checked_neg().ok_or_else(|| {
                    Error::OutOfRange("Specified mantissa cannot be i64::MIN".to_string())
                })?,
                true,
            ),
        };

        while mantissa < MANTISSA_MIN && exponent > EXPONENT_MIN {
            mantissa *= 10;
            exponent -= 1;
        }

        while mantissa > MANTISSA_MAX && exponent < EXPONENT_MAX {
            mantissa /= 10;
            exponent += 1;
        }

        if mantissa > MANTISSA_MAX || exponent > EXPONENT_MAX {
            return Err(Error::OutOfRange(format!(
                "Issued value too big to be normalized: {:?}",
                self
            )));
        }

        if mantissa < MANTISSA_MIN || exponent < EXPONENT_MIN {
            return Ok(Self::zero());
        }

        if negative {
            mantissa = -mantissa;
        }

        Ok(Self { mantissa, exponent })
    }
}

impl BinarySerialize for IssuedValue {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        // https://xrpl.org/docs/references/protocol/binary-format#token-amount-format
        const ISSUED_MASK: u64 = 0x8000000000000000;
        const POSITIVE_MASK: u64 = 0x4000000000000000;

        let (mantissa, positive) = match self.mantissa {
            0 => {
                ISSUED_MASK.binary_serialize_to(buf, for_signing);
                return;
            }
            1.. => (self.mantissa as u64, true),
            ..=-1 => (-self.mantissa as u64, false),
        };
        let exponent = (self.exponent + 97) as u64;
        let payload =
            ISSUED_MASK | (if positive { POSITIVE_MASK } else { 0 }) | mantissa | (exponent << 54);
        payload.binary_serialize_to(buf, for_signing);
    }
}

/// Amount of issued token. ref - https://xrpl.org/docs/references/protocol/data-types/currency-formats#token-amounts,
#[derive(Debug, Clone)]
pub struct IssuedAmount {
    // fields are private since it is validated when the IssuedAmount value is created
    pub value: IssuedValue,
    pub currency: CurrencyCode,
    pub issuer: AccountIdType,
}

impl IssuedAmount {
    pub fn from_issued_value(
        value: IssuedValue,
        currency: CurrencyCode,
        issuer: AccountIdType,
    ) -> Result<Self, Error> {
        if currency.is_valid() {
            return Err(Error::InvalidData(
                "Issued amount cannot have invalid currency code".to_string(),
            ));
        }
        Ok(Self {
            value,
            currency,
            issuer,
        })
    }
}

impl BinarySerialize for IssuedAmount {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        // https://xrpl.org/docs/references/protocol/binary-format#amount-fields
        self.value.binary_serialize_to(buf, for_signing);
        self.currency.binary_serialize_to(buf, for_signing);
        self.issuer.binary_serialize_to(buf, for_signing);
    }
}

/// Amount type, ref - https://xrpl.org/docs/references/protocol/data-types/currency-formats#specifying-currency-amounts
#[derive(Debug, Clone)]
pub enum AmountType {
    Issued(IssuedAmount), // For tokens
    Drops(u64),           // For XRP
}

impl BinarySerialize for AmountType {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        // https://xrpl.org/docs/references/protocol/binary-format#amount-fields
        match self {
            AmountType::Issued(issued_amount) => {
                issued_amount.binary_serialize_to(buf, for_signing)
            }
            AmountType::Drops(drops_amount) => {
                const POSITIVE_MASK: u64 = 0x4000000000000000;
                buf.extend_from_slice((drops_amount | POSITIVE_MASK).to_be_bytes().as_slice());
            }
        }
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
