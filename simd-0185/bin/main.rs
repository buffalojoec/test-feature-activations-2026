use {
    helpers::{
        read_keypair_file, Keypair,
        Signer, Transaction,
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

    helpers::print_transaction_logs_for_signature(&client, &signature);
}
