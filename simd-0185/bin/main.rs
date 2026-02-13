use {
    helpers::{
        read_keypair_file, CommitmentConfig, Keypair, OptionSerializer, RpcTransactionConfig,
        Signer, Transaction, UiTransactionEncoding,
    },
    simd_0185_interface::ProgramInstruction,
};

fn main() {
    let (client, payer) = helpers::client_from_args();

    let program_id =
        read_keypair_file("simd-0185/keypair.json").expect("failed to read program keypair");
    let program_id = program_id.pubkey();

    // Generate a fresh keypair for the vote account.
    let vote_account = Keypair::new();
    let authorized_voter = Keypair::new();
    let authorized_withdrawer = Keypair::new();
    let commission = 10;

    println!("Payer:                  {}", payer.pubkey());
    println!("Vote account:           {}", vote_account.pubkey());
    println!("Authorized voter:       {}", authorized_voter.pubkey());
    println!("Authorized withdrawer:  {}", authorized_withdrawer.pubkey());
    println!("Commission:             {}%", commission);
    println!();

    // Create instruction.
    let create_ix = ProgramInstruction::create(
        &program_id,
        &payer.pubkey(),
        &vote_account.pubkey(),
        &authorized_voter.pubkey(),
        &authorized_withdrawer.pubkey(),
        commission,
    );

    // View instruction.
    let view_ix = ProgramInstruction::view(&program_id, &vote_account.pubkey());

    // Build, sign, and send the transaction.
    let blockhash = client
        .get_latest_blockhash()
        .expect("failed to get blockhash");
    let tx = Transaction::new_signed_with_payer(
        &[create_ix, view_ix],
        Some(&payer.pubkey()),
        &[&payer, &vote_account],
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
