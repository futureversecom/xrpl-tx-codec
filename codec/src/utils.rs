use ripemd::{Digest as _, Ripemd160};
use sha2::Sha256;

/// Convert a 33 byte Secp256k1 pub key to an XRPL account ID
///
/// `public_key` The secp256k1 public key
///
/// Returns the XRPL Account ID
pub fn secp256k1_public_key_to_account_id(public_key: [u8; 33]) -> [u8; 20] {
    let pubkey_inner_hash = Sha256::digest(&public_key);
    Ripemd160::digest(pubkey_inner_hash).into()
}
