//! Interface for the SIMD-0185 test program.

use sha2_const_stable::Sha256;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

pub enum ProgramInstruction {
    /// Create a v4 vote account via CPI.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Uninitalized vote account
    /// 1. `[ ]` Identity PDA (below)
    /// 2. `[ ]` Rent sysvar
    /// 3. `[ ]` Clock sysvar
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
                let authorized_voter =
                    Pubkey::new_from_array(input[1..33].try_into().unwrap());
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

        Instruction {
            program_id: PROGRAM_ID_AS_PUBKEY,
            accounts: vec![
                AccountMeta::new(*vote_account, false),
                AccountMeta::new_readonly(get_identity_pda(), false),
                AccountMeta::new_readonly(solana_sdk_ids::sysvar::rent::ID, false),
                AccountMeta::new_readonly(solana_sdk_ids::sysvar::clock::ID, false),
            ],
            data,
        }
    }

    pub fn view(vote_account: &Pubkey) -> Instruction {
        Instruction {
            program_id: PROGRAM_ID_AS_PUBKEY,
            accounts: vec![AccountMeta::new_readonly(*vote_account, false)],
            data: vec![Self::VIEW],
        }
    }
}

const PREFIX: &[u8] = b"test_identity";
const BASE: &str = "8x2TcvfbVbBScE5kdJ8sdpJLFY5n84fM1rZBrC1QzFA1";
const BASE_AS_PUBKEY: Pubkey = Pubkey::from_str_const(BASE);
const BUMP: u8 = 254;
const PROGRAM_ID: &str = "33H7aP44PfN6WhknyrDo6wuipnwusHAQ1kK8b4anLwWj";
const PROGRAM_ID_AS_PUBKEY: Pubkey = Pubkey::from_str_const(PROGRAM_ID);
const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

pub const fn get_identity_pda() -> Pubkey {
    let bytes = Sha256::new()
        .update(PREFIX)
        .update(BASE_AS_PUBKEY.as_array())
        .update(&[BUMP])
        .update(PROGRAM_ID_AS_PUBKEY.as_array())
        .update(PDA_MARKER)
        .finalize();
    Pubkey::new_from_array(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_roundtrip() {
        let vote_account = Pubkey::new_unique();
        let authorized_voter = Pubkey::new_unique();
        let authorized_withdrawer = Pubkey::new_unique();
        let commission = 7;

        let ix = ProgramInstruction::create(
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

    #[test]
    fn test_pda() {
        let (_pda, bump) = Pubkey::find_program_address(
            &[PREFIX, BASE_AS_PUBKEY.as_ref()],
            &PROGRAM_ID_AS_PUBKEY,
        );
        assert_eq!(bump, BUMP);
    }
}