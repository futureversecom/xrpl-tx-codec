#![cfg(test)]

use std::process::Command;

use xrpl_codec::{traits::BinarySerialize, transaction::Payment};

// Assert `encoded` input decodes to `expected` JSON format (whitespace will be removed)
fn assert_decodes(encoded: &[u8], expected: &str) {
    let js_test = format!(
        "const xrpl = require(\"xrpl\"); console.log(xrpl.decode('{}'));",
        hex::encode(&encoded)
    );
    let result = Command::new("node")
        .env("NODE_PATH", "./tests/node_modules")
        .arg(format!("--eval={}", js_test))
        .output()
        .expect("node command failed to start");

    // strip whitespace
    assert_eq!(
        core::str::from_utf8(&result.stdout)
            .expect("valid utf8 only")
            .replace(" ", "")
            .trim(),
        expected.replace(" ", "")
    );
}

#[test]
fn serialize_payment_tx() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    let amount = 5_000_000_u64; // 5 XRP
    let nonce = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let mut payment = Payment::new(
        account,
        destination,
        amount,
        nonce,
        fee,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        Sequence: 1,
        Amount: '5000000',
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        Destination: 'rBcktgVfNjHmxNAQDEE66ztz4qZkdngdm'
    }";
    let encoded_no_signature = payment.binary_serialize(true);

    assert_decodes(encoded_no_signature.as_slice(), expected_payment_json);

    // with signature
    payment.attach_signature([7_u8; 65]);
    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        Sequence: 1,
        Amount: '5000000',
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        TxnSignature: '0707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        Destination: 'rBcktgVfNjHmxNAQDEE66ztz4qZkdngdm'
    }";
    let encoded_with_signature = payment.binary_serialize(false);
    assert_decodes(encoded_with_signature.as_slice(), expected_payment_json);
}

#[test]
fn serialize_payment_zero_values() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    let amount = 0; // 5 XRP
    let nonce = 0_u32;
    let fee = 0; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let payment = Payment::new(
        account,
        destination,
        amount,
        nonce,
        fee,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        Sequence: 0,
        Amount: '0',
        Fee: '0',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        Destination: 'rBcktgVfNjHmxNAQDEE66ztz4qZkdngdm'
    }";
    let encoded_no_signature = payment.binary_serialize(true);

    assert_decodes(encoded_no_signature.as_slice(), expected_payment_json);
}

#[test]
fn encode_for_multi_signing() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    let amount = 5_000_000_u64; // 5 XRP
    let nonce = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key =
        hex_literal::hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1");

    let payment = Payment::new(account, destination, amount, nonce, fee, None);

    let js_test = format!(
        r#"
          const xrpl = require("xrpl");
          let tx = xrpl.decode("{}");
          let accountId = xrpl.encodeAccountID(Buffer.from("{}", "hex"));
          console.log(xrpl.encodeForMultiSigning(tx, accountId));
        "#,
        hex::encode(&payment.binary_serialize(true)),
        hex::encode(xrpl_codec::utils::secp256k1_public_key_to_account_id(
            signing_pub_key
        )),
    );

    let result = Command::new("node")
        .env("NODE_PATH", "./tests/node_modules")
        .arg(format!("--eval={}", js_test))
        .output()
        .expect("node command failed to start");

    // sanitize the JS output
    let xrpl_js_output = core::str::from_utf8(&result.stdout)
        .expect("valid utf8 only")
        .replace(" ", "")
        .trim()
        .to_string()
        .to_lowercase();

    assert_eq!(
        xrpl_js_output,
        hex::encode(&payment.encode_for_multi_signing(signing_pub_key)),
    );
}

#[test]
fn public_key_to_account_id() {
    let pub_key: [u8; 33] =
        hex_literal::hex!("EDDF4ECB8F34A168143B928D48EFE625501FB8552403BBBD3FC038A5788951D770");
    let account_id_bytes = hex::encode(xrpl_codec::utils::secp256k1_public_key_to_account_id(
        pub_key,
    ));

    let js_test = format!(
        r#"
          const xrpl = require("xrpl");
          let accountId = xrpl.encodeAccountID(Buffer.from("{}", "hex"));
          console.log(accountId);
        "#,
        account_id_bytes,
    );

    let result = Command::new("node")
        .env("NODE_PATH", "./tests/node_modules")
        .arg(format!("--eval={}", js_test))
        .output()
        .expect("node command failed to start");

    // sanitize the JS output
    let xrpl_js_output = core::str::from_utf8(&result.stdout)
        .expect("valid utf8 only")
        .replace(" ", "")
        .trim()
        .to_string();

    assert_eq!(xrpl_js_output, "rLFd1FzHMScFhLsXeaxStzv3UC97QHGAbM");
}
