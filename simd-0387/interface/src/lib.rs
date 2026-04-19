//! Interface for the SIMD-0387 test program.

use {
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    solana_vote_interface::state::{
        BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE, BLS_PUBLIC_KEY_COMPRESSED_SIZE,
    },
};

pub enum ProgramInstruction {
    /// Set a v4 vote account's BLS public key using the
    /// `Authorize::VoterWithBls` instruction.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[s]` Authorized voter
    /// 1. `[w]` Vote account
    /// 4. `[ ]` Clock sysvar
    /// 6. `[ ]` Vote program
    Set {
        new_authorized_voter: Pubkey,
        bls_pubkey_compressed: [u8; BLS_PUBLIC_KEY_COMPRESSED_SIZE],
        bls_proof_of_possession: [u8; BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE],
    },

    /// Read the contents of a v4 vote account, with emphasis on the BLS
    /// public key.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[ ]` Vote account
    View,
}

impl ProgramInstruction {
    const SET: u8 = 0;
    const VIEW: u8 = 1;

    pub fn decode(input: &[u8]) -> Self {
        match input.first() {
            Some(&Self::SET) => {
                const OFFSET: usize = 1;
                const VOTER_END: usize = OFFSET + size_of::<Pubkey>();
                const BLS_PUBKEY_END: usize = VOTER_END + BLS_PUBLIC_KEY_COMPRESSED_SIZE;
                const BLS_POP_END: usize = BLS_PUBKEY_END + BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE;

                let new_authorized_voter =
                    Pubkey::new_from_array(input[OFFSET..VOTER_END].try_into().unwrap());
                let bls_pubkey_compressed = input[VOTER_END..BLS_PUBKEY_END].try_into().unwrap();
                let bls_proof_of_possession =
                    input[BLS_PUBKEY_END..BLS_POP_END].try_into().unwrap();

                Self::Set {
                    new_authorized_voter,
                    bls_pubkey_compressed,
                    bls_proof_of_possession,
                }
            }
            Some(&Self::VIEW) => Self::View,
            _ => panic!("invalid instruction"),
        }
    }

    pub fn set(
        program_id: &Pubkey,
        authorized_voter: &Pubkey,
        vote_account: &Pubkey,
        new_authorized_voter: &Pubkey,
        bls_pubkey_compressed: &[u8; BLS_PUBLIC_KEY_COMPRESSED_SIZE],
        bls_proof_of_possession: &[u8; BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE],
    ) -> Instruction {
        const CAPACITY: usize = 1
            + size_of::<Pubkey>()
            + BLS_PUBLIC_KEY_COMPRESSED_SIZE
            + BLS_PROOF_OF_POSSESSION_COMPRESSED_SIZE;
        let mut data = Vec::with_capacity(CAPACITY);
        data.push(Self::SET);
        data.extend_from_slice(new_authorized_voter.as_ref());
        data.extend_from_slice(bls_pubkey_compressed);
        data.extend_from_slice(bls_proof_of_possession);

        Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(*authorized_voter, true),
                AccountMeta::new(*vote_account, false),
                AccountMeta::new_readonly(solana_sdk_ids::sysvar::clock::ID, false),
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

#[cfg(test)]
mod tests {
    use {super::*, solana_vote_program::vote_state::create_bls_pubkey_and_proof_of_possession};

    #[test]
    fn test_set_roundtrip() {
        let program_id = Pubkey::new_unique();
        let authorized_voter = Pubkey::new_unique();
        let vote_account = Pubkey::new_unique();
        let new_authorized_voter = Pubkey::new_unique();
        let (bls_pubkey_compressed, bls_proof_of_possession) =
            create_bls_pubkey_and_proof_of_possession(&vote_account);

        let ix = ProgramInstruction::set(
            &program_id,
            &authorized_voter,
            &vote_account,
            &new_authorized_voter,
            &bls_pubkey_compressed,
            &bls_proof_of_possession,
        );

        match ProgramInstruction::decode(&ix.data) {
            ProgramInstruction::Set {
                new_authorized_voter,
                bls_pubkey_compressed,
                bls_proof_of_possession,
            } => {
                assert_eq!(new_authorized_voter, new_authorized_voter);
                assert_eq!(bls_pubkey_compressed, bls_pubkey_compressed);
                assert_eq!(bls_proof_of_possession, bls_proof_of_possession);
            }
            _ => panic!("expected Set"),
        }
    }
}
