//! SIMD-0431: Loader V3 minimum extend program size.
//!
//! Extends the deployed SIMD-0387 program's ProgramData account, confirming
//! that an extension of less than 10 KiB fails with `InvalidArgument` and an
//! extension of exactly 10 KiB succeeds.

use {
    helpers::{read_keypair_file, Signer, Transaction, TransactionError},
    solana_instruction::error::InstructionError,
    solana_loader_v3_interface::{
        get_program_data_address,
        instruction::{extend_program, MINIMUM_EXTEND_PROGRAM_BYTES},
    },
};

fn main() {
    // program_name [network]
    let (client, payer) = helpers::client_from_args();

    let program_id =
        read_keypair_file("simd-0387/keypair.json").expect("failed to read program keypair");
    let program_id = program_id.pubkey();
    let programdata_address = get_program_data_address(&program_id);

    let programdata_len = |label: &str| {
        let len = client
            .get_account(&programdata_address)
            .expect("failed to fetch ProgramData account")
            .data
            .len();
        println!("ProgramData length ({label}): {len} bytes");
        len
    };

    println!("Payer:        {}", payer.pubkey());
    println!("Program:      {}", program_id);
    println!("ProgramData:  {}", programdata_address);
    let starting_len = programdata_len("starting");
    println!();

    let send_extend_transaction = |additional_bytes: u32| {
        let extend_ix = extend_program(&program_id, Some(&payer.pubkey()), additional_bytes);
        let blockhash = client
            .get_latest_blockhash()
            .expect("failed to get blockhash");
        let tx = Transaction::new_signed_with_payer(
            &[extend_ix],
            Some(&payer.pubkey()),
            &[&payer],
            blockhash,
        );
        client.send_and_confirm_transaction(&tx)
    };

    // Fail - SIMD-0431: extensions below 10 KiB are rejected.
    let below_minimum = MINIMUM_EXTEND_PROGRAM_BYTES - 1;
    println!("Extending by {below_minimum} bytes (expecting failure)...");
    let error = send_extend_transaction(below_minimum)
        .expect_err("extend below the minimum unexpectedly succeeded");
    helpers::print_preflight_failure_logs(&error);
    let transaction_error = error
        .get_transaction_error()
        .expect("expected a transaction error");
    assert_eq!(
        transaction_error,
        TransactionError::InstructionError(0, InstructionError::InvalidArgument),
    );
    println!("Failed as expected: {transaction_error}");
    println!();

    // Success - extending by exactly 10 KiB.
    println!("Extending by {MINIMUM_EXTEND_PROGRAM_BYTES} bytes (expecting success)...");
    let signature =
        send_extend_transaction(MINIMUM_EXTEND_PROGRAM_BYTES).expect("transaction failed");
    println!("Success! Signature: {signature}");
    println!();

    helpers::print_transaction_logs_for_signature(&client, &signature);
    println!();

    let extended_len = programdata_len("extended");
    assert_eq!(
        extended_len,
        starting_len + MINIMUM_EXTEND_PROGRAM_BYTES as usize,
    );
}
