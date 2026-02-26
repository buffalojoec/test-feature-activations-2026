use solana_cli_config::CONFIG_FILE;
pub use {
    solana_cli_config::Config,
    solana_client::{
        rpc_client::RpcClient,
        rpc_config::{RpcTransactionConfig, UiTransactionEncoding},
        rpc_response::OptionSerializer,
    },
    solana_commitment_config::CommitmentConfig,
    solana_keypair::{read_keypair_file, Keypair, Signer},
    solana_transaction::{Signature, Transaction},
};

pub fn rpc_url_from_network(network: &str) -> String {
    match network {
        "mainnet" => "https://api.mainnet-beta.solana.com".to_string(),
        "devnet" => "https://api.devnet.solana.com".to_string(),
        "testnet" => "https://api.testnet.solana.com".to_string(),
        "localnet" => "http://localhost:8899".to_string(),
        other => {
            eprintln!("Unknown network: {other}");
            eprintln!("Expected: mainnet, devnet, testnet, or localnet");
            std::process::exit(1);
        }
    }
}

pub fn load_config() -> Config {
    let config_file = CONFIG_FILE
        .as_ref()
        .expect("unable to get config file path");
    Config::load(config_file).expect("failed to load Solana CLI config")
}

pub fn load_payer() -> Keypair {
    let config = load_config();
    read_keypair_file(&config.keypair_path).expect("failed to read payer keypair")
}

/// Parse optional network arg from `env::args().nth(1)`, resolve the RPC URL,
/// create an `RpcClient`, and load the payer keypair from the Solana CLI
/// config.
pub fn client_from_args() -> (RpcClient, Keypair) {
    let config = load_config();
    let payer = read_keypair_file(&config.keypair_path).expect("failed to read payer keypair");

    let rpc_url = match std::env::args().nth(1) {
        Some(network) => rpc_url_from_network(&network),
        None => config.json_rpc_url.clone(),
    };
    println!("RPC URL: {}", rpc_url);

    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    (client, payer)
}

/// Create an `RpcClient` and load the payer keypair. If `network_override` is
/// provided, use that network; otherwise, use the default from the Solana CLI
/// config.
pub fn client_with_network_override(network_override: Option<String>) -> (RpcClient, Keypair) {
    let payer = load_payer();

    if let Some(network) = network_override {
        let rpc_url = rpc_url_from_network(&network);
        println!("RPC URL: {}", rpc_url);
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        (client, payer)
    } else {
        let config = load_config();
        println!("RPC URL: {}", config.json_rpc_url);
        let client = RpcClient::new_with_commitment(config.json_rpc_url, CommitmentConfig::confirmed());
        (client, payer)
    }
}

/// Fetch and print transaction logs for a given signature.
pub fn print_transaction_logs_for_signature(
    client: &RpcClient,
    signature: &Signature,
) {
    let tx_response = client
        .get_transaction_with_config(
            signature,
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
