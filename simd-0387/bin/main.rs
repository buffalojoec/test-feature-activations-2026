use {
    helpers::{read_keypair_file, Signer, Transaction},
    simd_0387_interface::ProgramInstruction,
    solana_account::ReadableAccount,
    solana_pubkey::Pubkey,
    solana_vote_interface::state::VoteStateVersions,
    solana_vote_program::vote_state::create_bls_pubkey_and_proof_of_possession,
    std::{env, str::FromStr},
};

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
        eprintln!("  vote_account: the v4 vote account to set a BLS key on");
        std::process::exit(1);
    };

    let vote_pubkey = Pubkey::from_str(&vote_account_arg).expect("Invalid vote account pubkey");

    let (client, payer) = helpers::client_with_network_override(network_override);

    let program_id =
        read_keypair_file("simd-0387/keypair.json").expect("failed to read program keypair");
    let program_id = program_id.pubkey();

    // The same local keypair used by SIMD-0185 to create the vote account,
    // so we can sign `Authorize::VoterWithBLS` here.
    let authorized_voter = read_keypair_file("simd-0185/authorized-voter.json")
        .expect("failed to read authorized voter keypair");

    // Fetch the vote account and verify the authorized voter matches.
    let vote_account = client
        .get_account(&vote_pubkey)
        .expect("failed to fetch vote account");
    let VoteStateVersions::V4(vote_state) = VoteStateVersions::deserialize(vote_account.data())
        .expect("failed to deserialize vote state")
    else {
        panic!("expected v4 vote state");
    };
    let current_authorized_voter = vote_state
        .authorized_voters
        .last()
        .expect("vote account has no authorized voter")
        .1;
    assert_eq!(
        current_authorized_voter,
        &authorized_voter.pubkey(),
        "on-chain authorized voter does not match local authorized-voter.json",
    );

    // Derive the BLS pubkey + proof of possession for this vote account.
    let (bls_pubkey_compressed, bls_proof_of_possession) =
        create_bls_pubkey_and_proof_of_possession(&vote_pubkey);

    // Rotate to a new authorized voter as part of the Set ix.
    let new_authorized_voter = Pubkey::new_unique();

    println!("Payer:                  {}", payer.pubkey());
    println!("Program:                {}", program_id);
    println!("Vote account:           {}", vote_pubkey);
    println!("Authorized voter:       {}", authorized_voter.pubkey());
    println!("New authorized voter:   {}", new_authorized_voter);
    println!();

    let set_ix = ProgramInstruction::set(
        &program_id,
        &authorized_voter.pubkey(),
        &vote_pubkey,
        &new_authorized_voter,
        &bls_pubkey_compressed,
        &bls_proof_of_possession,
    );

    let view_ix = ProgramInstruction::view(&program_id, &vote_pubkey);

    let blockhash = client
        .get_latest_blockhash()
        .expect("failed to get blockhash");
    let tx = Transaction::new_signed_with_payer(
        &[set_ix, view_ix],
        Some(&payer.pubkey()),
        &[&payer, &authorized_voter],
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
