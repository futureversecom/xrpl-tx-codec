//! XRPL codec fields
//! These are higher-level types which typically wrap a primitive value

use xrpl_codec_utils::Field;

use crate::types::Hash256Type;
use crate::{
    traits::{BinarySerialize, CodecField},
    types::{
        AccountIdType, AmountType, BlobType, STArrayType, SignerEntryType, UInt16Type, UInt32Type,
        ACCOUNT_ID_TYPE_CODE,
    },
    Vec,
};

// TODO: auto-generate the structs from definitions.json

#[derive(Field, Debug, Clone)]
pub struct Account(pub AccountIdType);

#[derive(Field, Debug, Clone)]
pub struct Destination(pub AccountIdType);

#[derive(Field, Debug, Clone)]
pub struct TransactionType(pub UInt16Type);
impl From<TransactionTypeCode> for TransactionType {
    fn from(v: TransactionTypeCode) -> Self {
        TransactionType(UInt16Type(v.code()))
    }
}

#[derive(Field, Debug, Clone)]
pub struct Fee(pub AmountType);

#[derive(Field, Debug, Clone)]
pub struct Flags(pub UInt32Type);

#[derive(Field, Debug, Clone)]
pub struct Sequence(pub UInt32Type);

#[derive(Field, Debug, Clone)]
pub struct SourceTag(pub UInt32Type);

#[derive(Field, Debug, Clone)]
pub struct DestinationTag(pub UInt32Type);

#[derive(Field, Debug, Clone)]
pub struct TicketSequence(pub UInt32Type);

#[derive(Field, Debug, Default)]
pub struct SigningPubKey(pub BlobType);

#[derive(Field, Debug, Clone)]
pub struct Amount(pub AmountType);

#[derive(Field, Debug, Default)]
pub struct TxnSignature(pub BlobType);

#[derive(Field, Debug, Clone)]
pub struct SignerQuorum(pub UInt32Type);

#[derive(Field, Debug, Clone)]
pub struct SignerWeight(pub UInt16Type);

#[derive(Field, Debug, Clone)]
pub struct SignerEntry(pub SignerEntryType);

#[derive(Field, Debug, Clone)]
pub struct SignerEntries(pub STArrayType<SignerEntry>);

#[derive(Field, Debug, Clone)]
pub struct NFTokenID(pub Hash256Type);

#[derive(Field, Debug, Clone)]
pub struct NFTokenSellOffer(pub Hash256Type);

impl<T: CodecField> BinarySerialize for T {
    fn binary_serialize_to(&self, buf: &mut Vec<u8>, for_signing: bool) {
        if !self.is_serialized() {
            return;
        }

        if for_signing && !self.is_signing_field() {
            return;
        }

        // header
        if self.type_code() < 16 {
            if self.field_code() < 16 {
                buf.push(((self.type_code() << 4) | self.field_code()) as u8);
            } else {
                buf.push((self.type_code() << 4) as u8);
                buf.push(self.field_code() as u8);
            }
        } else if self.type_code() >= 16 && self.field_code() < 16 {
            buf.push(self.field_code() as u8);
            buf.push(self.type_code() as u8);
        } else {
            // self.type_code() >= 16 && self.field_code() >= 16
            buf.push(0_u8);
            buf.push(self.type_code() as u8);
            buf.push(self.field_code() as u8);
        }

        if !self.is_variable_length() {
            self.inner().binary_serialize_to(buf, for_signing);
            return;
        }

        // variable length prefixed type
        let mut data = self.inner().binary_serialize(for_signing);
        // AccountID length prefix is always 0x14
        // https://xrpl.org/serialization.html#accountid-fields
        if self.type_code() == ACCOUNT_ID_TYPE_CODE {
            buf.push(0x14);
        } else {
            // https://github.com/XRPLF/xrpl.js/blob/8a9a9bcc28ace65cde46eed5010eb8927374a736/packages/ripple-binary-codec/src/serdes/binary-serializer.ts#L103
            // https://xrpl.org/serialization.html#length-prefixing
            // length prefix
            match data.len() {
                0..=192 => buf.push(data.len() as u8),
                193..=12_480 => {
                    // 193 + ((a - 193) * 256) + b
                    let [length_a, length_b] = ((data.len() - 193) as u16).to_be_bytes();
                    buf.push(length_a + 193_u8);
                    buf.push(length_b as u8);
                }
                12_481..=918_744 => {
                    // 12_481 + ((a - 241) * 65_536) + (b * 256) + c
                    let [length_a, length_b, length_c, _length_d] =
                        ((data.len() - 12_841) as u32).to_be_bytes();
                    buf.push(241 + length_a);
                    buf.push(length_b);
                    buf.push(length_c);
                }
                _ => panic!("data too long"),
            }
        }
        buf.append(&mut data);
    }
}

/// XRPL TransactionTypes
pub enum TransactionTypeCode {
    // Invalid = -1,
    Payment = 0,
    EscrowCreate = 1,
    EscrowFinish = 2,
    AccountSet = 3,
    EscrowCancel = 4,
    SetRegularKey = 5,
    NickNameSet = 6,
    OfferCreate = 7,
    OfferCancel = 8,
    Contract = 9,
    TicketCreate = 10,
    TicketCancel = 11,
    SignerListSet = 12,
    PaymentChannelCreate = 13,
    PaymentChannelFund = 14,
    PaymentChannelClaim = 15,
    CheckCreate = 16,
    CheckCash = 17,
    CheckCancel = 18,
    DepositPreauth = 19,
    TrustSet = 20,
    AccountDelete = 21,
    SetHook = 22,
    NFTokenMint = 25,
    NFTokenBurn = 26,
    NFTokenCreateOffer = 27,
    NFTokenCancelOffer = 28,
    NFTokenAcceptOffer = 29,
    EnableAmendment = 100,
    SetFee = 101,
    UNLModify = 102,
}

impl TransactionTypeCode {
    pub fn code(self) -> u16 {
        self as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BlobType, CurrencyCodeType, IssuedAmountType, IssuedValueType};

    #[test]
    fn serialize_signing_pub_key() {
        let buf = SigningPubKey(BlobType(vec![1_u8; 65])).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
    }
    #[test]
    fn serialize_transaction_type() {
        let tt: TransactionType = TransactionTypeCode::Payment.into();
        let buf = tt.binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
    }
    #[test]
    fn serialize_account() {
        let account = [1_u8; 20];
        let buf = Account(AccountIdType(account)).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
    }
    #[test]
    fn serialize_destination() {
        let dest = [1_u8; 20];
        let buf = Destination(AccountIdType(dest)).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
    }
    #[test]
    fn serialize_signer_entry() {
        let signer_entry = SignerEntry(SignerEntryType(
            Account(AccountIdType([1_u8; 20])),
            SignerWeight(UInt16Type(1_u16)),
        ));
        let buf = signer_entry.binary_serialize(true);
        // construct the expected buffer manually
        let signer_entry_field_id: u8 = 0xEB; // Typecode(14) | FieldCode(11) = 0xEB
        let account_field_id: u8 = 0x81; // Typecode(8) | Fieldcode(1) = 0x81(129)
        let signer_weight_field_id: u8 = 0x13; // Typecode(1) | Fieldcode(3) = 0x13(19)
        let account_field_vl: u8 = 0x14; // https://xrpl.org/serialization.html#accountid-fields
        let st_object_end: u8 = 0xe1; // https://xrpl.org/serialization.html#object-fields

        let mut expected_buf = Vec::<u8>::default();
        expected_buf.extend_from_slice(&[signer_entry_field_id]);
        expected_buf.extend_from_slice(&[signer_weight_field_id]); // SignerWeight comes first in the canonical order
        expected_buf.extend_from_slice(&1_u16.to_be_bytes());
        expected_buf.extend_from_slice(&[account_field_id]);
        expected_buf.extend_from_slice(&[account_field_vl]);
        expected_buf.extend_from_slice(&[1_u8; 20]);
        expected_buf.extend_from_slice(&[st_object_end]);

        assert_eq!(buf, expected_buf);
    }
    #[test]
    fn serialize_signer_entries() {
        let mut signer_entries_vec = Vec::<SignerEntry>::default();
        for i in 1..=2 {
            signer_entries_vec.push(SignerEntry(SignerEntryType(
                Account(AccountIdType([i as u8; 20])),
                SignerWeight(UInt16Type(i as u16)),
            )));
        }
        let signer_entries = SignerEntries(STArrayType(signer_entries_vec));

        let buf = signer_entries.binary_serialize(true);
        let signer_entries_field_id: u8 = 0xF4; // Typecode(15) | FieldCode(4) = 0xF4
        let signer_entry_field_id: u8 = 0xEB; // Typecode(14) | FieldCode(11) = 0xEB
        let account_field_id: u8 = 0x81; // Typecode(8) | Fieldcode(1) = 0x81(129)
        let signer_weight_field_id: u8 = 0x13; // Typecode(1) | Fieldcode(3) = 0x13(19)
        let account_field_vl: u8 = 0x14; // https://xrpl.org/serialization.html#accountid-fields
        let st_object_end: u8 = 0xe1; // https://xrpl.org/serialization.html#object-fields

        // let's construct the expected buffer -> https://xrpl.org/serialization.html#array-fields
        let mut expected_buf = Vec::<u8>::default();
        expected_buf.extend_from_slice(&[signer_entries_field_id]);
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
    #[test]
    fn serialize_sequence() {
        let nonce = 17_u32;
        let buf = Sequence(UInt32Type(nonce)).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
    }
    #[test]
    fn serialize_ticket_sequence() {
        let ticket_number = 1_u32;
        let buf = TicketSequence(UInt32Type(ticket_number)).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
    }
    #[test]
    fn serialize_source_tag() {
        let source_tag = 38_887_387_u32;
        let buf = SourceTag(UInt32Type(source_tag)).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
    }
    #[test]
    fn serialize_destinatiom_tag() {
        let destination_tag = 18_887_987_u32;
        let buf = DestinationTag(UInt32Type(destination_tag)).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
    }
    #[test]
    fn serialize_amount_drops() {
        let amount = 10;
        let buf = Amount(AmountType::Drops(amount)).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
        assert_eq!(buf.as_slice(), hex::decode("61400000000000000a").unwrap());
    }

    #[test]
    fn serialize_amount_issued() {
        let issuer = [3_u8; 20];
        let token_symbol = b"ASA";
        let token_amount = 5; // 5 ASA
        let issued_amount = IssuedAmountType::from_issued_value(
            IssuedValueType::from_mantissa_exponent(token_amount, 0).unwrap(),
            CurrencyCodeType::Standard(token_symbol.clone()),
            AccountIdType(issuer),
        )
        .unwrap();
        let buf = Amount(AmountType::Issued(issued_amount)).binary_serialize(true);
        println!("{:?}", hex::encode(&buf));
        assert_eq!(buf.as_slice(), hex::decode("61d491c37937e0800000000000000000000000000041534100000000000303030303030303030303030303030303030303").unwrap());
    }
}
