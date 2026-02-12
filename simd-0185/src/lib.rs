use {
    simd_0185_interface::{get_identity_seeds_with_bump, ProgramInstruction},
    solana_account_info::AccountInfo,
    solana_cpi::invoke_signed,
    solana_msg::msg,
    solana_program_error::ProgramResult,
    solana_pubkey::Pubkey,
    solana_rent::Rent,
    solana_sysvar::SysvarSerialize,
    solana_vote_interface::state::{VoteStateV4, VoteStateVersions},
};

solana_program_entrypoint::entrypoint!(process);

fn process_create(
    accounts: &[AccountInfo],
    authorized_voter: Pubkey,
    authorized_withdrawer: Pubkey,
    commission: u8,
) -> ProgramResult {
    let payer = &accounts[0];
    let vote_account = &accounts[1];
    let identity_pda = &accounts[2];
    let rent_sysvar = &accounts[3];
    let clock_sysvar = &accounts[4];

    let pda_signer_seeds = &get_identity_seeds_with_bump();

    let rent = Rent::from_account_info(rent_sysvar)?;
    let lamports = rent.minimum_balance(VoteStateV4::size_of());

    // CPI to System Program: create the vote account.
    let create_ix = solana_system_interface::instruction::create_account(
        payer.key,
        vote_account.key,
        lamports,
        VoteStateV4::size_of() as u64,
        &solana_sdk_ids::vote::ID,
    );
    invoke_signed(
        &create_ix,
        &[payer.clone(), vote_account.clone()],
        &[pda_signer_seeds],
    )?;

    // TODO: CPI to Vote Program to initialize the account.

    Ok(())
}

fn process_view(accounts: &[AccountInfo]) -> ProgramResult {
    let vote_account = &accounts[0];
    let data = vote_account.try_borrow_data()?;

    let vote_state = match VoteStateVersions::deserialize(&data) {
        Ok(VoteStateVersions::V4(state)) => state,
        _ => panic!("expected v4 vote state"),
    };

    msg!("Vote State (v4):");
    msg!("  node_pubkey:                    {}", vote_state.node_pubkey);
    msg!("  authorized_withdrawer:          {}", vote_state.authorized_withdrawer);
    msg!("  inflation_rewards_collector:    {}", vote_state.inflation_rewards_collector);
    msg!("  block_revenue_collector:        {}", vote_state.block_revenue_collector);
    msg!("  inflation_rewards_commission:   {} bps", vote_state.inflation_rewards_commission_bps);
    msg!("  block_revenue_commission:       {} bps", vote_state.block_revenue_commission_bps);

    Ok(())
}

fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    match ProgramInstruction::decode(input) {
        ProgramInstruction::Create {
            authorized_voter,
            authorized_withdrawer,
            commission,
        } => process_create(accounts, authorized_voter, authorized_withdrawer, commission),
        ProgramInstruction::View => process_view(accounts),
    }
}

#[cfg(test)]
mod tests {
    use {
        mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk},
        simd_0185_interface::{get_identity_pda, ProgramInstruction},
        solana_account::Account,
        solana_pubkey::Pubkey,
        solana_vote_interface::state::VoteStateV4,
    };

    const PROGRAM_ID: Pubkey =
        Pubkey::from_str_const("33H7aP44PfN6WhknyrDo6wuipnwusHAQ1kK8b4anLwWj");

    #[test]
    fn test_create() {
        let mollusk = Mollusk::new(&PROGRAM_ID, "simd_0185");

        let payer = Pubkey::new_unique();
        let vote_account = Pubkey::new_unique();
        let identity_pda = get_identity_pda();

        let instruction = ProgramInstruction::create(
            &payer,
            &vote_account,
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            10,
        );

        let lamports = mollusk.sysvars.rent.minimum_balance(VoteStateV4::size_of());

        mollusk.process_and_validate_instruction(
            &instruction,
            &[
                (payer, Account::new(lamports * 2, 0, &solana_sdk_ids::system_program::ID)),
                (vote_account, Account::default()),
                (identity_pda, Account::default()),
                mollusk.sysvars.keyed_account_for_rent_sysvar(),
                mollusk.sysvars.keyed_account_for_clock_sysvar(),
                keyed_account_for_system_program(),
            ],
            &[
                Check::success(),
                Check::account(&vote_account)
                    .owner(&solana_sdk_ids::vote::ID)
                    .build(),
            ],
        );
    }
}
