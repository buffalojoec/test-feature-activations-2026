use {
    helpers::{
        Keypair, Signer, Transaction,
    },
    solana_pubkey::Pubkey,
    solana_stake_interface::{
        instruction::{delegate_stake, initialize},
        state::{Authorized, Lockup},
    },
    solana_system_interface::instruction::create_account,
    std::{env, str::FromStr},
};

const STAKE_STATE_SIZE: usize = std::mem::size_of::<solana_stake_interface::state::StakeStateV2>();

fn main() {
    let args: Vec<String> = env::args().collect();

    // program_name <network> <vote_account>
    // - or -
    // program_name <vote_account>
    let (vote_account_arg, network_override) = if args.len() >= 3 {
        (args[2].clone(), Some(args[1].clone()))
    } else if args.len() == 2 {
        (args[1].clone(), None)
    } else {
        eprintln!("Usage: {} [network] <vote_account>", args[0]);
        eprintln!("  network: localnet, devnet, testnet, or mainnet (optional)");
        eprintln!("  vote_account: the vote account to delegate stake to");
        std::process::exit(1);
    };

    let vote_account = Pubkey::from_str(&vote_account_arg)
        .expect("Invalid vote account pubkey");

    let (client, payer) = helpers::client_with_network_override(network_override);

    // Generate a fresh keypair for the stake account
    let stake_account = Keypair::new();

    // Stake amount: 5,000 lamports
    let stake_amount = 5_000u64;

    // Get rent for stake account
    let rent = client
        .get_minimum_balance_for_rent_exemption(STAKE_STATE_SIZE)
        .expect("failed to get rent exemption");

    let total_lamports = rent + stake_amount;

    println!("Payer:           {}", payer.pubkey());
    println!("Stake account:   {}", stake_account.pubkey());
    println!("Vote account:    {}", vote_account);
    println!("Stake amount:    {} lamports", stake_amount);
    println!("Rent exemption:  {} lamports", rent);
    println!("Total lamports:  {} lamports", total_lamports);
    println!();

    // Step 1: Create stake account.
    let create_account_ix = create_account(
        &payer.pubkey(),
        &stake_account.pubkey(),
        total_lamports,
        STAKE_STATE_SIZE as u64,
        &solana_sdk_ids::stake::ID,
    );

    // Step 2: Initialize stake account.
    let authorized = Authorized {
        staker: payer.pubkey(),
        withdrawer: payer.pubkey(),
    };
    let lockup = Lockup::default();

    let initialize_ix = initialize(
        &stake_account.pubkey(),
        &authorized,
        &lockup,
    );

    // Step 3: Delegate stake.
    let delegate_ix = delegate_stake(
        &stake_account.pubkey(),
        &payer.pubkey(),
        &vote_account,
    );

    // Build, sign, and send the transaction.
    let blockhash = client
        .get_latest_blockhash()
        .expect("failed to get blockhash");
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, initialize_ix, delegate_ix],
        Some(&payer.pubkey()),
        &[&payer, &stake_account],
        blockhash,
    );

    println!("Sending transaction...");
    let signature = client
        .send_and_confirm_transaction(&tx)
        .expect("transaction failed");
    println!("Success! Signature: {}", signature);
    println!();

    helpers::print_transaction_logs_for_signature(&client, &signature);
}
