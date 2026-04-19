use {
    simd_0387_interface::ProgramInstruction,
    solana_account_info::AccountInfo,
    solana_cpi::invoke,
    solana_msg::msg,
    solana_program_error::ProgramResult,
    solana_pubkey::Pubkey,
    solana_vote_interface::{
        instruction::authorize,
        state::{
            VoteAuthorize, VoteStateVersions, VoterWithBLSArgs,
            BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE, BLS_PUBLIC_KEY_COMPRESSED_SIZE,
        },
    },
};

solana_program_entrypoint::entrypoint!(process);

fn process_set(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_authorized_voter: Pubkey,
    bls_pubkey_compressed: [u8; BLS_PUBLIC_KEY_COMPRESSED_SIZE],
    bls_proof_of_possession: [u8; BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE],
) -> ProgramResult {
    let authorized_voter = &accounts[0];
    let vote_account = &accounts[1];
    let clock_sysvar = &accounts[2];

    // CPI to Vote Program: add the BLS key.
    let authorize_ix = authorize(
        vote_account.key,
        authorized_voter.key,
        &new_authorized_voter,
        VoteAuthorize::VoterWithBLS(VoterWithBLSArgs {
            bls_pubkey: bls_pubkey_compressed,
            bls_proof_of_possession: bls_proof_of_possession,
        }),
    );
    invoke(
        &authorize_ix,
        &[
            vote_account.clone(),
            clock_sysvar.clone(),
            authorized_voter.clone(),
        ],
    )?;

    Ok(())
}

fn process_view(accounts: &[AccountInfo]) -> ProgramResult {
    let vote_account = &accounts[0];
    let data = vote_account.try_borrow_data()?;

    let vote_state = match VoteStateVersions::deserialize(&data) {
        Ok(VoteStateVersions::V4(state)) => state,
        _ => panic!("expected v4 vote state"),
    };

    let bls_pubkey = match vote_state.bls_pubkey_compressed {
        Some(pubkey) => pubkey,
        _ => panic!("expected some bls key"),
    };

    msg!("What's the matter, you've never seen a compressed BLS pubkey before?");
    msg!("  BLS PUBKEY (COMPRESSED):  {:?}", bls_pubkey,);

    Ok(())
}

fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    match ProgramInstruction::decode(input) {
        ProgramInstruction::Set {
            new_authorized_voter,
            bls_pubkey_compressed,
            bls_proof_of_possession,
        } => process_set(
            program_id,
            accounts,
            new_authorized_voter,
            bls_pubkey_compressed,
            bls_proof_of_possession,
        ),
        ProgramInstruction::View => process_view(accounts),
    }
}

#[cfg(test)]
mod tests {
    use {
        mollusk_svm::{program::create_keyed_account_for_builtin_program, result::Check, Mollusk},
        simd_0387_interface::ProgramInstruction,
        solana_account::Account,
        solana_instruction::Instruction,
        solana_pubkey::Pubkey,
        solana_sdk_ids::system_program,
        solana_vote_interface::{
            instruction::{create_account_with_config, CreateVoteAccountConfig},
            state::{
                VoteInit, VoteStateV4, VoteStateVersions, BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE,
                BLS_PUBLIC_KEY_COMPRESSED_SIZE,
            },
        },
        solana_vote_program::vote_state::create_bls_pubkey_and_proof_of_possession,
    };

    fn setup(
        program_id: &Pubkey,
        authorized_voter: &Pubkey,
        vote_pubkey: &Pubkey,
        new_authorized_voter: &Pubkey,
        bls_pubkey_compressed: &[u8; BLS_PUBLIC_KEY_COMPRESSED_SIZE],
        bls_proof_of_possession: &[u8; BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE],
        mollusk: &Mollusk,
    ) -> (Instruction, Instruction, Vec<(Pubkey, Account)>) {
        // First set up the v4 vote account.
        let payer = Pubkey::new_unique();
        let space = VoteStateV4::size_of();
        let lamports = mollusk.sysvars.rent.minimum_balance(space);
        let pre_result = mollusk.process_instruction_chain(
            &create_account_with_config(
                &payer,
                &vote_pubkey,
                &VoteInit {
                    node_pubkey: *authorized_voter,
                    authorized_voter: *authorized_voter,
                    authorized_withdrawer: *authorized_voter,
                    commission: 0,
                },
                lamports,
                CreateVoteAccountConfig {
                    space: space as u64,
                    with_seed: None,
                },
            ),
            &[
                (payer, Account::new(100_000_000, 0, &system_program::id())),
                (*vote_pubkey, Account::default()),
                mollusk.sysvars.keyed_account_for_rent_sysvar(),
                mollusk.sysvars.keyed_account_for_clock_sysvar(),
                (*authorized_voter, Account::default()),
            ],
        );

        let vote_account = &pre_result.resulting_accounts[1].1;

        let set_ix = ProgramInstruction::set(
            program_id,
            &authorized_voter,
            vote_pubkey,
            &new_authorized_voter,
            &bls_pubkey_compressed,
            &bls_proof_of_possession,
        );

        let view_ix = ProgramInstruction::view(program_id, vote_pubkey);

        let accounts = vec![
            (*authorized_voter, Account::default()),
            (*vote_pubkey, vote_account.clone()),
            mollusk.sysvars.keyed_account_for_clock_sysvar(),
            create_keyed_account_for_builtin_program(&solana_sdk_ids::vote::ID, "vote_program"),
        ];

        (set_ix, view_ix, accounts)
    }

    #[test]
    fn test_set() {
        let program_id = Pubkey::new_unique();
        let authorized_voter = Pubkey::new_unique();
        let vote_pubkey = Pubkey::new_unique();
        let mollusk = Mollusk::new(&program_id, "simd_0387");

        let new_authorized_voter = Pubkey::new_unique();
        let (bls_pubkey_compressed, bls_proof_of_possession) =
            create_bls_pubkey_and_proof_of_possession(&vote_pubkey);

        let (set_ix, _, accounts) = setup(
            &program_id,
            &authorized_voter,
            &vote_pubkey,
            &new_authorized_voter,
            &bls_pubkey_compressed,
            &bls_proof_of_possession,
            &mollusk,
        );

        let vote_account = accounts[1].0;

        let result = mollusk.process_and_validate_instruction(
            &set_ix,
            &accounts,
            &[
                Check::success(),
                Check::account(&vote_account)
                    .owner(&solana_sdk_ids::vote::ID)
                    .build(),
            ],
        );

        let vote_account_data = &result.get_account(&vote_account).unwrap().data;
        let vote_state = match VoteStateVersions::deserialize(vote_account_data) {
            Ok(VoteStateVersions::V4(state)) => state,
            _ => panic!("expected v4 vote state"),
        };

        assert_eq!(
            vote_state.authorized_voters.last().unwrap().1,
            &new_authorized_voter
        );
        assert_eq!(
            vote_state.bls_pubkey_compressed.unwrap(),
            bls_pubkey_compressed
        );
    }

    // #[test]
    // fn fail_feature_disabled() {
    //     let program_id = Pubkey::new_unique();
    //     let authorized_voter = Pubkey::new_unique();
    //     let vote_pubkey = Pubkey::new_unique();
    //     let mut mollusk = Mollusk::new(&program_id, "simd_0387");
    //     mollusk
    //         .feature_set
    //         .deactivate(&agave_feature_set::bls_key_management::id());

    //     let (set_ix, _, accounts) = setup(
    //         &program_id,
    //         &authorized_voter,
    //         &vote_pubkey,
    //         &Pubkey::new_unique(),
    //         &[4u8; BLS_PUBLIC_KEY_COMPRESSED_SIZE],
    //         &[4u8; BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE],
    //         &mollusk,
    //     );

    //     // Should fail - VoterWithBls is disabled.
    //     mollusk.process_and_validate_instruction(
    //         &set_ix,
    //         &accounts,
    //         &[Check::instruction_err(InstructionError::InvalidInstructionData)],
    //     );
    // }
}
