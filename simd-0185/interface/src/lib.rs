//! Interface for the SIMD-0185 test program.

use sha2_const_stable::Sha256;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

#[repr(u8)]
pub enum ProgramInstruction {
    /// Create a v4 vote account via CPI.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Uninitalized vote account
    /// 1. `[ ]` Identity PDA (below)
    /// 2. `[ ]` Rent sysvar
    /// 3. `[ ]` Clock sysvar
    Create,

    /// Read the contents of a v4 vote account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[ ]` Initialized vote account
    View,
}

impl ProgramInstruction {
    pub fn decode(input: &[u8]) -> Self {
        match input.first() {
            Some(0) => Self::Create,
            Some(1) => Self::View,
            _ => panic!("invalid instruction"),
        }
    }

    pub fn create(vote_account: &Pubkey) -> Instruction {
        Instruction {
            program_id: PROGRAM_ID_AS_PUBKEY,
            accounts: vec![
                AccountMeta::new(*vote_account, false),
                AccountMeta::new_readonly(get_identity_pda(), false),
                AccountMeta::new_readonly(solana_sdk_ids::sysvar::rent::ID, false),
                AccountMeta::new_readonly(solana_sdk_ids::sysvar::clock::ID, false),
            ],
            data: vec![ProgramInstruction::Create as u8],
        }
    }

    pub fn view(vote_account: &Pubkey) -> Instruction {
        Instruction {
            program_id: PROGRAM_ID_AS_PUBKEY,
            accounts: vec![AccountMeta::new_readonly(*vote_account, false)],
            data: vec![ProgramInstruction::View as u8],
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
    fn test_pda() {
        let (_pda, bump) = Pubkey::find_program_address(
            &[PREFIX, BASE_AS_PUBKEY.as_ref()],
            &PROGRAM_ID_AS_PUBKEY,
        );
        assert_eq!(bump, BUMP);
    }
}