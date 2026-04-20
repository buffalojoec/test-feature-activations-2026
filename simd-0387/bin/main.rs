use {
    helpers::{read_keypair_file, Keypair, Signer, Transaction},
    simd_0387_interface::ProgramInstruction,
    solana_account::ReadableAccount,
    solana_pubkey::Pubkey,
    solana_vote_interface::state::VoteStateVersions,
    solana_vote_program::vote_state::create_bls_pubkey_and_proof_of_possession,
    std::{env, str::FromStr},
};

fn main() {
    let program_id =
        read_keypair_file("simd-0185/keypair.json").expect("failed to read program keypair");
    let program_id = program_id.pubkey();
    let authorized_voter =
        read_keypair_file("simd-0185/authorized-voter.json").expect("failed to read authorized voter keypair");
    let authorized_voter_pubkey = authorized_voter.pubkey();

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

    let vote_pubkey = Pubkey::from_str(&vote_account_arg).expect("Invalid vote account pubkey");

    let (client, payer) = helpers::client_with_network_override(network_override);
    let vote_account = client.get_account(&vote_pubkey).unwrap();
    let VoteStateVersions::V4(vote_state) = VoteStateVersions::deserialize(vote_account.data()).unwrap()
    else {
        panic!("exptected v4 vote state")
    };

    // Set instruction.
    let new_authorized_voter = Pubkey::new_unique();
    let (bls_pubkey_compressed, bls_proof_of_possession) =
        create_bls_pubkey_and_proof_of_possession(&vote_pubkey);
    let set_ix = ProgramInstruction::set(
        &program_id,
        vote_state.authorized_voters.last().unwrap().1,
        &vote_pubkey,
        &new_authorized_voter,
        &bls_pubkey_compressed,
        &bls_proof_of_possession,
    );

    // View instruction.
    let view_ix = ProgramInstruction::view(&program_id, &vote_pubkey);

    // Build, sign, and send the transaction.
    let blockhash = client
        .get_latest_blockhash()
        .expect("failed to get blockhash");
    let tx = Transaction::new_signed_with_payer(
        &[set_ix, view_ix],
        Some(&payer.pubkey()),
        &[&payer, &authorized_voter],
        blockhash,
    );




    let (client, payer) = helpers::client_from_args();

    let program_id =
        read_keypair_file("simd-0387/keypair.json").expect("failed to read program keypair");
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

    helpers::print_transaction_logs_for_signature(&client, &signature);
}
