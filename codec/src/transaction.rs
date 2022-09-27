//! XRPL transaction types
use xrpl_codec_utils::Transaction;

use crate::{
    field::*,
    traits::{BinarySerialize, CodecField, CodecToFields},
    types::{AccountIdType, AmountType, BlobType, UInt32Type},
    Vec,
};

/*
   Tx Common Fields
   https://xrpl.org/transaction-common-fields.html#memos-field
   ```
   Account 	String 	AccountID
   TransactionType 	String 	UInt16
   Fee 	String 	Amount
   Sequence 	Number 	UInt32
   SigningPubKey String  Blob *part of signed data ✅
   TxnSignature String Blob
   ```

   Payment
   https://xrpl.org/payment.html

   ```
   Amount  CurrencyAmount Amount (XRP = string only)
   Destination String 	AccountID
   ```

   SignerList
   https://xrpl.org/signerlistset.html
   ```
   SignerQuorum 	Number 	UInt32
   SignerEntries 	Array 	Array
       Account 	String 	AccountID
       SignerWeight 	Number 	UInt16
   ```
*/

// pub struct SetSignerList {
//     /// common tx fields
//     account: Field::AccountId,
//     transaction_type: Field::TransactionType,
//     fee: Field::Fee,
//     sequence: Field::Sequence,
//     /// payment only
//     amount: Field::Amount,
//     destination: Field::AccountId,
//     /// set when signing
//     /// TODO: set to empty string / omit these fields for multisig txs
//     ///     use 'signers' instead
//     signing_pub_key: Field::SigningPubKey,
//     txn_signature: Field::TxnSignature,
// }

/*
SignersField
https://xrpl.org/transaction-common-fields.html#signers-field

Account 	String 	AccountID 	The address associated with this signature, as it appears in the signer list.
TxnSignature 	String 	Blob 	A signature for this transaction, verifiable using the SigningPubKey.
SigningPubKey 	String 	Blob 	The public key used to create this signature.
*/

/// An XRP payment tx
#[derive(Transaction, Debug)]
pub struct Payment {
    /// common tx fields
    account: Account,
    transaction_type: TransactionType,
    fee: Fee,
    sequence: Sequence,
    flags: Flags,
    /// payment only
    amount: Amount,
    destination: Destination,
    /// set when signing
    signing_pub_key: SigningPubKey,
    txn_signature: TxnSignature,
}

impl Payment {
    /// Create a new XRP payment transaction
    ///
    /// Applies the global signing flags (see https://xrpl.org/transaction-common-fields.html#global-flags)
    ///
    /// - `account` the sender's address
    /// - `destination` the address to receive XRP
    /// - `amount` the amount of XRP to receive in drops
    /// - `nonce` the XRPL 'Sequence' # of `account`
    /// - `fee` the max XRP fee in drops
    pub fn new(
        account: [u8; 20],
        destination: [u8; 20],
        amount: u64,
        nonce: u32,
        fee: u64,
        signing_pub_key: Option<[u8; 33]>,
    ) -> Self {
        Self {
            account: Account(AccountIdType(account)),
            transaction_type: TransactionTypeCode::Payment.into(),
            fee: Fee(AmountType(fee)),
            sequence: Sequence(UInt32Type(nonce)),
            // https://xrpl.org/transaction-common-fields.html#global-flags
            flags: Flags(UInt32Type(0x8000_0000_u32)),
            /// payment only
            amount: Amount(AmountType(amount)),
            destination: Destination(AccountIdType(destination)),
            signing_pub_key: signing_pub_key
                .map(|pk| SigningPubKey(BlobType(pk.to_vec())))
                .unwrap_or_default(),
            txn_signature: Default::default(),
        }
    }
    /// Attach a signature to the transaction
    pub fn attach_signature(&mut self, signature: [u8; 65]) {
        self.txn_signature = TxnSignature(BlobType(signature.to_vec()));
    }
}

/// An XRP SignerListSet tx
#[derive(Transaction, Debug)]
pub struct SignerListSet {
    account: Account,
    transaction_type: TransactionType,
    fee: Fee,
    flags: Flags,
    SignerQuorum: u32,
    SignerEntries: Vec<SignerEntry>,
}

#[cfg(test)]
mod tests {
    use super::{CodecToFields, Payment};

    #[test]
    fn canonical_field_order() {
        let account = [1_u8; 20];
        let destination = [2_u8; 20];
        let amount = 5_000_000_u64; // 5 XRP
        let nonce = 1_u32;
        let fee = 1_000; // 1000 drops
        let signing_pub_key = [1_u8; 33];
        let payment = Payment::new(
            account,
            destination,
            amount,
            nonce,
            fee,
            Some(signing_pub_key),
        );

        for chunk in payment.to_canonical_fields().chunks(2) {
            match chunk {
                &[f1, f2] => {
                    assert!(
                        f1.type_code() < f2.type_code()
                            || f1.type_code() == f2.type_code()
                                && f1.field_code() <= f2.field_code()
                    );
                }
                _ => continue,
            }
        }
    }
}
