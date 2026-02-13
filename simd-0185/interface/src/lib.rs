//! Interface for the SIMD-0185 test program.

use {
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    solana_vote_interface::{instruction::VoteInstruction, state::VoteInit},
};

pub enum ProgramInstruction {
    /// Create a v4 vote account via CPI.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[ws]` Payer
    /// 1. `[w]` Uninitialized vote account
    /// 2. `[ ]` Identity PDA (below)
    /// 3. `[ ]` Rent sysvar
    /// 4. `[ ]` Clock sysvar
    /// 5. `[ ]` System program
    /// 6. `[ ]` Vote program
    Create {
        authorized_voter: Pubkey,
        authorized_withdrawer: Pubkey,
        commission: u8,
    },

    /// Read the contents of a v4 vote account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[ ]` Initialized vote account
    View,
}

impl ProgramInstruction {
    const CREATE: u8 = 0;
    const VIEW: u8 = 1;

    pub fn decode(input: &[u8]) -> Self {
        match input.first() {
            Some(&Self::CREATE) => {
                let authorized_voter = Pubkey::new_from_array(input[1..33].try_into().unwrap());
                let authorized_withdrawer =
                    Pubkey::new_from_array(input[33..65].try_into().unwrap());
                let commission = input[65];
                Self::Create {
                    authorized_voter,
                    authorized_withdrawer,
                    commission,
                }
            }
            Some(&Self::VIEW) => Self::View,
            _ => panic!("invalid instruction"),
        }
    }

    pub fn create(
        program_id: &Pubkey,
        payer: &Pubkey,
        vote_account: &Pubkey,
        authorized_voter: &Pubkey,
        authorized_withdrawer: &Pubkey,
        commission: u8,
    ) -> Instruction {
        let mut data = Vec::with_capacity(66);
        data.push(Self::CREATE);
        data.extend_from_slice(authorized_voter.as_ref());
        data.extend_from_slice(authorized_withdrawer.as_ref());
        data.push(commission);

        let (identity_pda, _) = get_identity_pda(program_id);

        Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(*payer, true),
                AccountMeta::new(*vote_account, true),
                AccountMeta::new_readonly(identity_pda, false),
                AccountMeta::new_readonly(solana_sdk_ids::sysvar::rent::ID, false),
                AccountMeta::new_readonly(solana_sdk_ids::sysvar::clock::ID, false),
                AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk_ids::vote::ID, false),
            ],
            data,
        }
    }

    pub fn view(program_id: &Pubkey, vote_account: &Pubkey) -> Instruction {
        Instruction {
            program_id: *program_id,
            accounts: vec![AccountMeta::new_readonly(*vote_account, false)],
            data: vec![Self::VIEW],
        }
    }
}

const PREFIX: &[u8] = b"test_identity";
const BASE: &str = "8x2TcvfbVbBScE5kdJ8sdpJLFY5n84fM1rZBrC1QzFA1";
const BASE_AS_PUBKEY: Pubkey = Pubkey::from_str_const(BASE);

pub fn get_identity_seeds(bump: &u8) -> [&[u8]; 3] {
    [PREFIX, BASE_AS_PUBKEY.as_ref(), core::slice::from_ref(bump)]
}

pub fn get_identity_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PREFIX, BASE_AS_PUBKEY.as_ref()], program_id)
}

pub fn vote_initialize_account(vote_pubkey: &Pubkey, vote_init: &VoteInit) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*vote_pubkey, false),
        AccountMeta::new_readonly(solana_sdk_ids::sysvar::rent::ID, false),
        AccountMeta::new_readonly(solana_sdk_ids::sysvar::clock::ID, false),
        AccountMeta::new_readonly(vote_init.node_pubkey, true),
    ];

    Instruction::new_with_bincode(
        solana_sdk_ids::vote::ID,
        &VoteInstruction::InitializeAccount(*vote_init),
        account_metas,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_roundtrip() {
        let program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let vote_account = Pubkey::new_unique();
        let authorized_voter = Pubkey::new_unique();
        let authorized_withdrawer = Pubkey::new_unique();
        let commission = 7;

        let ix = ProgramInstruction::create(
            &program_id,
            &payer,
            &vote_account,
            &authorized_voter,
            &authorized_withdrawer,
            commission,
        );

        match ProgramInstruction::decode(&ix.data) {
            ProgramInstruction::Create {
                authorized_voter: decoded_voter,
                authorized_withdrawer: decoded_withdrawer,
                commission: decoded_commission,
            } => {
                assert_eq!(decoded_voter, authorized_voter);
                assert_eq!(decoded_withdrawer, authorized_withdrawer);
                assert_eq!(decoded_commission, commission);
            }
            _ => panic!("expected Create"),
        }
    }
}
