use {
    simd_0185_interface::ProgramInstruction,
    solana_cli_config::{Config, CONFIG_FILE},
    solana_client::{
        rpc_client::RpcClient,
        rpc_config::RpcTransactionConfig,
        rpc_response::OptionSerializer,
    },
    solana_commitment_config::CommitmentConfig,
    solana_keypair::{read_keypair_file, Keypair, Signer},
    solana_transaction::Transaction,
};

fn rpc_url_from_network(network: &str) -> String {
    match network {
        "mainnet" => "https://api.mainnet-beta.solana.com".to_string(),
        "devnet" => "https://api.devnet.solana.com".to_string(),
        "testnet" => "https://api.testnet.solana.com".to_string(),
        "localnet" => "http://localhost:8899".to_string(),
        other => {
            eprintln!("Unknown network: {other}");
            eprintln!("Usage: simd-0185 [mainnet|devnet|testnet|localnet]");
            std::process::exit(1);
        }
    }
}

fn main() {
    // Load the local signer from Solana CLI config.
    let config_file = CONFIG_FILE.as_ref().expect("unable to get config file path");
    let config = Config::load(config_file).expect("failed to load Solana CLI config");
    let payer = read_keypair_file(&config.keypair_path).expect("failed to read payer keypair");

    let rpc_url = match std::env::args().nth(1) {
        Some(network) => rpc_url_from_network(&network),
        None => config.json_rpc_url.clone(),
    };
    println!("RPC URL: {}", rpc_url);

    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

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
        &payer.pubkey(),
        &vote_account.pubkey(),
        &authorized_voter.pubkey(),
        &authorized_withdrawer.pubkey(),
        commission,
    );

    // View instruction.
    let view_ix = ProgramInstruction::view(&vote_account.pubkey());

    // Build, sign, and send the transaction.
    let blockhash = client.get_latest_blockhash().expect("failed to get blockhash");
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
                encoding: Some(solana_client::rpc_config::UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .expect("failed to fetch transaction");

    if let Some(meta) = tx_response.transaction.meta {
        if let OptionSerializer::Some(logs) = meta.log_messages
        {
            println!("Transaction logs:");
            for log in &logs {
                println!("  {}", log);
            }
        }
    }
}
