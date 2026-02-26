use {
    helpers::{
        read_keypair_file, Signer,
        Transaction,
    },
    simd_0321_interface::{build_instruction, EasterEgg},
};

fn main() {
    let (client, payer) = helpers::client_from_args();

    let program_id =
        read_keypair_file("simd-0321/keypair.json").expect("failed to read program keypair");
    let program_id = program_id.pubkey();

    // Instruction 1: random bytes — program will log raw bytes.
    let random_ix = build_instruction(&program_id, vec![0xDE, 0xAD, 0xBE, 0xEF]);

    // Instruction 2: valid EasterEgg payload — program will print ASCII owl +
    // message.
    let egg = EasterEgg::compose("Hoot hoot! You found the secret owl!".into());
    let egg_ix = build_instruction(&program_id, egg.encode());

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

    helpers::print_transaction_logs_for_signature(&client, &signature);
}
