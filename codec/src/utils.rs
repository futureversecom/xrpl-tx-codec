use ripemd::{Digest as _, Ripemd160};
use sha2::Sha256;

use crate::{traits::BinarySerialize, Vec};

/// Convert a 33 byte Secp256k1 pub key to an XRPL account ID
///
/// `public_key` The secp256k1 public key
///
/// Returns the XRPL Account ID
pub fn secp256k1_public_key_to_account_id(public_key: [u8; 33]) -> [u8; 20] {
    let pubkey_inner_hash = Sha256::digest(&public_key);
    Ripemd160::digest(pubkey_inner_hash).into()
}

/// Calculate the tx digest ready for multi signing
///
/// `tx` an XRPL tx type
/// `public_key` the secp256k1 public key that will sign the digest
///
/// Returns the tx digest ready for signing
pub fn digest_for_multi_signing(tx: &impl BinarySerialize, public_key: [u8; 33]) -> [u8; 32] {
    let tx_data = encode_for_multi_signing(tx, public_key);
    let digest: [u8; 64] = sha2::Sha512::digest(tx_data).into();
    digest[..32].try_into().expect("it is a 32 byte digest")
}

/// Encode a tx ready for multi-signing
pub fn encode_for_multi_signing(tx: &impl BinarySerialize, public_key: [u8; 33]) -> Vec<u8> {
    [
        &[0x53, 0x4d, 0x54, 0x00],
        tx.binary_serialize(true).as_slice(),
        secp256k1_public_key_to_account_id(public_key).as_slice(),
    ]
    .concat()
    .to_vec()
}

/// Prepare a pre-encoded tx for multi-signing by some `public_key`
///
/// `tx_data` RBC encoded tx data (in 'for signing' mode)
/// `public_key` the secp256k1 public key that will sign the digest
///
/// Returns the tx digest ready for signing
pub fn digest_for_multi_signing_pre(tx_data: &[u8], public_key: [u8; 33]) -> Vec<u8> {
    let tx_data = [
        &[0x53, 0x4d, 0x54, 0x00],
        tx_data,
        secp256k1_public_key_to_account_id(public_key).as_slice(),
    ]
    .concat();
    let digest: [u8; 64] = sha2::Sha512::digest(tx_data).into();
    digest[..32].try_into().expect("it is a 32 byte digest")
}
