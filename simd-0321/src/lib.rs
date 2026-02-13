//! # SIMD-0321
//!
//! Test program that reads instruction data using the r2 register pointer.

#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::missing_safety_doc)]

#[no_mangle]
pub unsafe extern "C" fn entrypoint(_input: *mut u8, instruction_data_addr: *const u8) -> u64 {
    let instruction_data_len = *((instruction_data_addr as u64 - 8) as *const u64);
    let instruction_data =
        core::slice::from_raw_parts(instruction_data_addr, instruction_data_len as usize);

    solana_msg::msg!("{:?}", instruction_data);

    solana_program_entrypoint::SUCCESS
}

solana_program_entrypoint::custom_heap_default!();
solana_program_entrypoint::custom_panic_default!();

#[cfg(test)]
mod tests {
    use {
        mollusk_svm::{result::Check, Mollusk},
        simd_0321_interface::EasterEgg,
        solana_instruction::Instruction,
        solana_pubkey::Pubkey,
    };

    fn run(instruction_data: &[u8], checks: &[Check]) {
        let program_id = Pubkey::new_unique();
        let mollusk = Mollusk::new(&program_id, "simd_0321");
        let instruction =
            Instruction::new_with_bytes(program_id, instruction_data, vec![]);
        mollusk.process_and_validate_instruction(&instruction, &[], checks);
    }

    #[test]
    fn bad_payload() {
        run(&[0xDE, 0xAD], &[Check::success()]);
    }

    #[test]
    fn easter_egg_payload() {
        let egg = EasterEgg::compose("You found the easter egg!".into());
        let data = egg.encode();
        run(&data, &[Check::success()]);
    }
}
