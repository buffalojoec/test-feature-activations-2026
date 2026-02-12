use {
    simd_0185_interface::ProgramInstruction,
    solana_account_info::AccountInfo,
    solana_msg::msg,
    solana_program_error::ProgramResult,
    solana_pubkey::Pubkey,
    solana_vote_interface::state::VoteStateVersions,
};

solana_program_entrypoint::entrypoint!(process);

fn process_create(
    accounts: &[AccountInfo],
    authorized_voter: Pubkey,
    authorized_withdrawer: Pubkey,
    commission: u8,
) -> ProgramResult {
    let vote_account = &accounts[0];
    let identity_pda = &accounts[1];
    let rent_sysvar = &accounts[2];
    let clock_sysvar = &accounts[3];

    // TODO: Build VoteInit and CPI to the vote program to initialize the
    //       account, signing with the identity PDA seeds.
    todo!()
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
