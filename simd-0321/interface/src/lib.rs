//! Interface for the SIMD-0321 test program.

use {solana_instruction::Instruction, solana_program_error::ProgramError, solana_pubkey::Pubkey};

/// Secret easter egg for the program to print structured logs instead of bytes.
#[derive(Debug)]
pub struct EasterEgg {
    code: u64,
    key: [u8; 32],
    pub message: String,
}

const CODE: u64 = 0xffb;
const KEY: &str = "9TKZQ53LMdcAThGFzTH16881BdpsSxSXU7z8jWHbmH6Z";
const KEY_AS_PUBKEY: Pubkey = Pubkey::from_str_const(KEY);

impl EasterEgg {
    pub fn try_decode(input: &[u8]) -> Result<Self, ProgramError> {
        // Must have the proper length.
        if input.len() < 40 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Secrets must match!
        let code = u64::from_le_bytes(input[..8].try_into().unwrap());
        let key: [u8; 32] = input[8..40].try_into().unwrap();
        if code != CODE || key != KEY_AS_PUBKEY.to_bytes() {
            return Err(ProgramError::InvalidInstructionData);
        }

        // Only properly keyed payload can decode.
        let message = String::from_utf8(input[40..].to_vec())
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        Ok(Self { code, key, message })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(40 + self.message.len());
        buf.extend_from_slice(&self.code.to_le_bytes());
        buf.extend_from_slice(&self.key);
        buf.extend_from_slice(self.message.as_bytes());
        buf
    }

    pub fn compose(message: String) -> Self {
        Self {
            code: CODE,
            key: KEY_AS_PUBKEY.to_bytes(),
            message,
        }
    }
}

pub fn build_instruction(program_id: &Pubkey, data: Vec<u8>) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![],
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short() {
        assert_eq!(
            EasterEgg::try_decode(&[0; 39]).unwrap_err(),
            ProgramError::InvalidInstructionData,
        );
        assert_eq!(
            EasterEgg::try_decode(&[]).unwrap_err(),
            ProgramError::InvalidInstructionData,
        );
    }

    #[test]
    fn bad_code() {
        let mut bad = EasterEgg::compose("hello".into()).encode();
        bad[..8].copy_from_slice(&0xdeadu64.to_le_bytes());
        assert_eq!(
            EasterEgg::try_decode(&bad).unwrap_err(),
            ProgramError::InvalidInstructionData,
        );
    }

    #[test]
    fn bad_key() {
        let mut bad = EasterEgg::compose("hello".into()).encode();
        bad[8..40].copy_from_slice(&[0xff; 32]);
        assert_eq!(
            EasterEgg::try_decode(&bad).unwrap_err(),
            ProgramError::InvalidInstructionData,
        );
    }

    #[test]
    fn happy_path() {
        let msg = "You found the easter egg!";
        let encoded = EasterEgg::compose(msg.into()).encode();
        let decoded = EasterEgg::try_decode(&encoded).unwrap();
        assert_eq!(decoded.code, CODE);
        assert_eq!(decoded.key, KEY_AS_PUBKEY.to_bytes());
        assert_eq!(decoded.message, msg);
    }
}
