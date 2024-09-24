#![cfg(test)]

use std::ops::Mul;
use std::process::Command;

use xrpl_codec::field::Amount;
use xrpl_codec::transaction::{
    NFTokenAcceptOffer, NFTokenCreateOffer, PaymentAltCurrency, PaymentWithDestinationTag,
};
use xrpl_codec::types::{
    AccountIdType, AmountType, CurrencyCodeType, IssuedAmountType, IssuedValueType,
};
use xrpl_codec::{
    traits::BinarySerialize,
    transaction::{Payment, SignerListSet},
};

// Assert `encoded` input decodes to `expected` JSON format (whitespace will be removed)
fn assert_decodes(encoded: &[u8], expected: &str) {
    let js_test = format!(
        "const util = require('util'); const xrpl = require(\"xrpl\"); \
        console.log(util.inspect(xrpl.decode('{}'), false, null, false));",
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
            .replace("\n", "")
            .trim(),
        expected.replace(" ", "").replace("\n", "")
    );
}

#[test]
fn serialize_payment_tx() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    let amount = 5_000_000_u64; // 5 XRP
    let nonce = 1_u32;
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;
    let mut payment = Payment::new(
        account,
        destination,
        amount,
        nonce,
        ticket_number,
        fee,
        source_tag,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        TicketSequence: 1,
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
        SourceTag: 38887387,
        Sequence: 1,
        TicketSequence: 1,
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
fn serialize_payment_alt_tx() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    // AST token info
    let issuer = [3_u8; 20];
    let token_symbol = b"AST";
    let token_amount = 5; // 5 AST
    let issued_amount = IssuedAmountType::from_issued_value(
        IssuedValueType::from_mantissa_exponent(token_amount, 0).unwrap(),
        CurrencyCodeType::Standard(token_symbol.clone()),
        AccountIdType(issuer),
    )
    .unwrap();
    let amount = Amount(AmountType::Issued(issued_amount));
    let nonce = 1_u32;
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;
    let mut payment = PaymentAltCurrency::new(
        account,
        destination,
        amount,
        nonce,
        ticket_number,
        fee,
        source_tag,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        TicketSequence: 1,
        Amount: {
            value: '5',
            currency: 'AST',
            issuer: 'rGvdqXNwMbSwRiubF4PhhVqzhkiaTDPgU'
        },
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
        SourceTag: 38887387,
        Sequence: 1,
        TicketSequence: 1,
        Amount: {
            value: '5',
            currency: 'AST',
            issuer: 'rGvdqXNwMbSwRiubF4PhhVqzhkiaTDPgU'
        },
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
fn serialize_payment_alt_tx_decimal_amount() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    // AST token info
    let issuer = [3_u8; 20];
    let token_symbol = b"AST";
    let token_amount = 3.14; // 3.14 AST
    let issued_amount = IssuedAmountType::from_issued_value(
        IssuedValueType::from_mantissa_exponent(token_amount.mul(100_f64).round() as i64, -2)
            .unwrap(),
        CurrencyCodeType::Standard(token_symbol.clone()),
        AccountIdType(issuer),
    )
    .unwrap();
    let amount = Amount(AmountType::Issued(issued_amount));
    let nonce = 1_u32;
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;
    let mut payment = PaymentAltCurrency::new(
        account,
        destination,
        amount,
        nonce,
        ticket_number,
        fee,
        source_tag,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        TicketSequence: 1,
        Amount: {
            value: '3.14',
            currency: 'AST',
            issuer: 'rGvdqXNwMbSwRiubF4PhhVqzhkiaTDPgU'
        },
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
        SourceTag: 38887387,
        Sequence: 1,
        TicketSequence: 1,
        Amount: {
            value: '3.14',
            currency: 'AST',
            issuer: 'rGvdqXNwMbSwRiubF4PhhVqzhkiaTDPgU'
        },
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
fn serialize_payment_alt_tx_non_standard_currency_code() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    // AST token info
    let issuer = [3_u8; 20];
    let token_symbol = [5_u8; 20];
    let token_amount = 3.14; // 3.14 AST
    let issued_amount = IssuedAmountType::from_issued_value(
        IssuedValueType::from_mantissa_exponent(token_amount.mul(100_f64).round() as i64, -2)
            .unwrap(),
        CurrencyCodeType::NonStandard(token_symbol),
        AccountIdType(issuer),
    )
    .unwrap();
    let amount = Amount(AmountType::Issued(issued_amount));
    let nonce = 1_u32;
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;
    let mut payment = PaymentAltCurrency::new(
        account,
        destination,
        amount,
        nonce,
        ticket_number,
        fee,
        source_tag,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        TicketSequence: 1,
        Amount: {
            value: '3.14',
            currency: '0505050505050505050505050505050505050505',
            issuer: 'rGvdqXNwMbSwRiubF4PhhVqzhkiaTDPgU'
        },
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
        SourceTag: 38887387,
        Sequence: 1,
        TicketSequence: 1,
        Amount: {
            value: '3.14',
            currency: '0505050505050505050505050505050505050505',
            issuer: 'rGvdqXNwMbSwRiubF4PhhVqzhkiaTDPgU'
        },
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
fn serialize_payment_with_destination_tag_tx() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    let amount = 5_000_000_u64; // 5 XRP
    let nonce = 1_u32;
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;
    let destination_tag = 12_124_121_u32;
    let mut payment = PaymentWithDestinationTag::new(
        account,
        destination,
        amount,
        nonce,
        ticket_number,
        fee,
        source_tag,
        destination_tag,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        DestinationTag: 12124121,
        TicketSequence: 1,
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
        SourceTag: 38887387,
        Sequence: 1,
        DestinationTag: 12124121,
        TicketSequence: 1,
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
fn serialize_payment_with_destination_tag_as_zero_tx() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    let amount = 5_000_000_u64; // 5 XRP
    let nonce = 1_u32;
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;
    let destination_tag = 0_u32;
    let mut payment = PaymentWithDestinationTag::new(
        account,
        destination,
        amount,
        nonce,
        ticket_number,
        fee,
        source_tag,
        destination_tag,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        DestinationTag: 0,
        TicketSequence: 1,
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
        SourceTag: 38887387,
        Sequence: 1,
        DestinationTag: 0,
        TicketSequence: 1,
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
    let ticket_number = 0_u32;
    let fee = 0; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;

    let payment = Payment::new(
        account,
        destination,
        amount,
        nonce,
        ticket_number,
        fee,
        source_tag,
        Some(signing_pub_key),
    );

    let expected_payment_json = r"{
        TransactionType: 'Payment',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 0,
        TicketSequence: 0,
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
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key =
        hex_literal::hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1");
    let source_tag = 38_887_387_u32;

    let payment = Payment::new(
        account,
        destination,
        amount,
        nonce,
        ticket_number,
        fee,
        source_tag,
        None,
    );

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
        hex::encode(&xrpl_codec::utils::encode_for_multi_signing(
            &payment,
            signing_pub_key
        )),
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

#[test]
#[allow(non_snake_case)]
fn decode_SignerListSet_tx() {
    let account = [1_u8; 20];
    let fee = 1_000; // 1000 drops
    let nonce = 1_u32;
    let ticket_number = 1_u32;
    let signing_pub_key = [1_u8; 33];
    let signer_quorum = 3_u32;
    let mut signer_entries = Vec::<([u8; 20], u16)>::default();
    signer_entries.push(([1_u8; 20], 1_u16));
    signer_entries.push(([2_u8; 20], 2_u16));
    let source_tag = 38_887_387_u32;

    let mut signer_list_set = SignerListSet::new(
        account,
        fee,
        nonce,
        ticket_number,
        signer_quorum,
        signer_entries.clone(),
        source_tag,
        Some(signing_pub_key),
    );

    let encoded_no_signature = signer_list_set.binary_serialize(true);

    let expected_signer_list_set_json = r"{
        TransactionType: 'SignerListSet',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        SignerQuorum: 3,
        TicketSequence: 1,
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        SignerEntries: [
            {
                SignerEntry: {
                    SignerWeight: 1,
                    Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC'
                }
            },
            {
                SignerEntry: {
                    SignerWeight: 2,
                    Account: 'rBcktgVfNjHmxNAQDEE66ztz4qZkdngdm'
                }
            }
        ]
    }";

    assert_decodes(
        encoded_no_signature.as_slice(),
        expected_signer_list_set_json,
    );
    // with signature
    signer_list_set.attach_signature([7_u8; 65]);
    let expected_signer_list_set_json = r"{
        TransactionType: 'SignerListSet',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        SignerQuorum: 3,
        TicketSequence: 1,
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        TxnSignature: '0707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        SignerEntries: [
            {
                SignerEntry: {
                    SignerWeight: 1,
                    Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC'
                }
            },
            {
                SignerEntry: {
                    SignerWeight: 2,
                    Account: 'rBcktgVfNjHmxNAQDEE66ztz4qZkdngdm'
                }
            }
        ]
    }";
    let encoded_with_signature = signer_list_set.binary_serialize(false);
    assert_decodes(
        encoded_with_signature.as_slice(),
        expected_signer_list_set_json,
    );
}

#[test]
#[allow(non_snake_case)]
fn decode_SignerListSet_tx_empty_signer_entries() {
    let account = [1_u8; 20];
    let fee = 1_000; // 1000 drops
    let nonce = 1_u32;
    let ticket_number = 1_u32;
    let signing_pub_key = [1_u8; 33];
    let signer_quorum = 3_u32;
    let source_tag = 38_887_387_u32;
    // let mut signer_entries = Vec::<([u8; 20], u16)>::default();
    // signer_entries.push(([1_u8; 20], 1_u16));
    // signer_entries.push(([2_u8; 20], 2_u16));

    let mut signer_list_set = SignerListSet::new(
        account,
        fee,
        nonce,
        ticket_number,
        signer_quorum,
        Default::default(),
        source_tag,
        Some(signing_pub_key),
    );

    let encoded_no_signature = signer_list_set.binary_serialize(true);

    let expected_signer_list_set_json = r"{
        TransactionType: 'SignerListSet',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        SignerQuorum: 3,
        TicketSequence: 1,
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        SignerEntries: []
    }";

    assert_decodes(
        encoded_no_signature.as_slice(),
        expected_signer_list_set_json,
    );
    // with signature
    signer_list_set.attach_signature([7_u8; 65]);
    let expected_signer_list_set_json = r"{
        TransactionType: 'SignerListSet',
        Flags: 2147483648,
        SourceTag: 38887387,
        Sequence: 1,
        SignerQuorum: 3,
        TicketSequence: 1,
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        TxnSignature: '0707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        SignerEntries: []
    }";
    let encoded_with_signature = signer_list_set.binary_serialize(false);
    assert_decodes(
        encoded_with_signature.as_slice(),
        expected_signer_list_set_json,
    );
}

#[test]
fn serialize_NFTokenCreateOffer_tx() {
    let account = [1_u8; 20];
    let destination = [2_u8; 20];
    let nftoken_id = [3_u8; 32];
    let amount = 0_u64; // 0 XRP
    let sequence = 0_u32;
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;
    let mut nftoken_create_offer = NFTokenCreateOffer::new(
        account,
        destination,
        nftoken_id,
        amount,
        sequence,
        ticket_number,
        fee,
        source_tag,
        Some(signing_pub_key),
    );

    let expected_offer_json = r"{
        TransactionType: 'NFTokenCreateOffer',
        Flags: 1,
        SourceTag: 38887387,
        Sequence: 0,
        TicketSequence: 1,
        NFTokenID: '0303030303030303030303030303030303030303030303030303030303030303',
        Amount: '0',
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        Destination: 'rBcktgVfNjHmxNAQDEE66ztz4qZkdngdm'
    }";
    let encoded_no_signature = nftoken_create_offer.binary_serialize(true);

    assert_decodes(encoded_no_signature.as_slice(), expected_offer_json);

    // with signature
    nftoken_create_offer.attach_signature([7_u8; 65]);
    let expected_offer_json = r"{
        TransactionType: 'NFTokenCreateOffer',
        Flags: 1,
        SourceTag: 38887387,
        Sequence: 0,
        TicketSequence: 1,
        NFTokenID: '0303030303030303030303030303030303030303030303030303030303030303',
        Amount: '0',
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        TxnSignature: '0707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC',
        Destination: 'rBcktgVfNjHmxNAQDEE66ztz4qZkdngdm'
    }";
    let encoded_with_signature = nftoken_create_offer.binary_serialize(false);
    assert_decodes(encoded_with_signature.as_slice(), expected_offer_json);
}

#[test]
fn serialize_NFTokenAcceptOffer_tx() {
    let account = [1_u8; 20];
    let nftoken_sell_offer = [3_u8; 32];
    let sequence = 0_u32;
    let ticket_number = 1_u32;
    let fee = 1_000; // 1000 drops
    let signing_pub_key = [1_u8; 33];
    let source_tag = 38_887_387_u32;
    let mut nftoken_accept_offer = NFTokenAcceptOffer::new(
        account,
        nftoken_sell_offer,
        sequence,
        ticket_number,
        fee,
        source_tag,
        Some(signing_pub_key),
    );

    let expected_accept_offer_json = r"{
        TransactionType: 'NFTokenAcceptOffer',
        SourceTag: 38887387,
        Sequence: 0,
        TicketSequence: 1,
        NFTokenSellOffer: '0303030303030303030303030303030303030303030303030303030303030303',
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC'
    }";
    let encoded_no_signature = nftoken_accept_offer.binary_serialize(true);

    assert_decodes(encoded_no_signature.as_slice(), expected_accept_offer_json);

    // with signature
    nftoken_accept_offer.attach_signature([7_u8; 65]);
    let expected_accept_offer_json = r"{
        TransactionType: 'NFTokenAcceptOffer',
        SourceTag: 38887387,
        Sequence: 0,
        TicketSequence: 1,
        NFTokenSellOffer: '0303030303030303030303030303030303030303030303030303030303030303',
        Fee: '1000',
        SigningPubKey: '010101010101010101010101010101010101010101010101010101010101010101',
        TxnSignature: '0707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707',
        Account: 'raJ1Aqkhf19P7cyUc33MMVAzgvHPvtNFC'
    }";
    let encoded_with_signature = nftoken_accept_offer.binary_serialize(false);
    // println!("{:?}", hex::encode(encoded_with_signature.clone()));
    assert_decodes(
        encoded_with_signature.as_slice(),
        expected_accept_offer_json,
    );
}
