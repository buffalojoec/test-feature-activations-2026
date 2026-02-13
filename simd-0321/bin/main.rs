use {
    helpers::{
        CommitmentConfig, OptionSerializer, RpcTransactionConfig, Signer, Transaction,
        UiTransactionEncoding,
    },
    simd_0321_interface::{build_instruction, EasterEgg},
};

fn main() {
    let (client, payer) = helpers::client_from_args();

    // Instruction 1: random bytes — program will log raw bytes.
    let random_ix = build_instruction(vec![0xDE, 0xAD, 0xBE, 0xEF]);

    // Instruction 2: valid EasterEgg payload — program will print ASCII owl +
    // message.
    let egg = EasterEgg::compose("Hoot hoot! You found the secret owl!".into());
    let egg_ix = build_instruction(egg.encode());

    // Build, sign, and send the transaction.
    let blockhash = client
        .get_latest_blockhash()
        .expect("failed to get blockhash");
    let tx = Transaction::new_signed_with_payer(
        &[random_ix, egg_ix],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );

    println!("Sending transaction...");
    let signature = client
        .send_and_confirm_transaction(&tx)
        .expect("transaction failed");
    println!("Success! Signature: {}", signature);
    println!();

    // Fetch and print transaction logs.
    let tx_response = client
        .get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .expect("failed to fetch transaction");

    if let Some(meta) = tx_response.transaction.meta {
        if let OptionSerializer::Some(logs) = meta.log_messages {
            println!("Transaction logs:");
            for log in &logs {
                println!("  {}", log);
            }
        }
    }
}
