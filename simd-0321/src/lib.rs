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

    match simd_0321_interface::EasterEgg::try_decode(instruction_data) {
        Ok(egg) => {
            solana_msg::msg!("A secret has been unlocked");
            solana_msg::msg!("");
            solana_msg::msg!(r"        ,---.");
            solana_msg::msg!(r"       /     \");
            solana_msg::msg!(r"      | () () |");
            solana_msg::msg!(r"       \  ^  /");
            solana_msg::msg!(r"        |||||");
            solana_msg::msg!(r"        |||||");
            solana_msg::msg!("");
            solana_msg::msg!("~ {} ~", egg.message);
        }
        Err(_) => solana_msg::msg!("{:?}", instruction_data),
    }

    solana_program_entrypoint::SUCCESS
}

solana_program_entrypoint::custom_heap_default!();
solana_program_entrypoint::custom_panic_default!();

#[cfg(test)]
mod tests {
    use {
        agave_feature_set::provide_instruction_data_offset_in_vm_r2,
        mollusk_svm::{result::Check, Mollusk},
        simd_0321_interface::EasterEgg,
        solana_instruction::{error::InstructionError, Instruction},
        solana_pubkey::Pubkey,
        solana_svm_log_collector::LogCollector,
        std::{cell::RefCell, rc::Rc},
    };

    fn setup(data: &[u8]) -> (Mollusk, Instruction, Rc<RefCell<LogCollector>>) {
        let program_id = Pubkey::new_unique();
        let mut mollusk = Mollusk::new(&program_id, "simd_0321");
        let log_collector = LogCollector::new_ref();
        mollusk.logger = Some(log_collector.clone());
        let instruction = Instruction::new_with_bytes(program_id, data, vec![]);
        (mollusk, instruction, log_collector)
    }

    #[test]
    fn bytes() {
        let (mollusk, instruction, log_collector) = setup(&[0xDE, 0xAD]);
        mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);

        let logs = log_collector.borrow().get_recorded_content().to_vec();
        assert!(logs.iter().any(|log| log.contains("[222, 173]")),);
    }

    #[test]
    fn easter_egg() {
        let data = EasterEgg::compose("a warrior was here".into()).encode();
        let (mollusk, instruction, log_collector) = setup(&data);

        mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);

        let logs = log_collector.borrow().get_recorded_content().to_vec();
        assert!(logs
            .iter()
            .any(|log| log.contains("A secret has been unlocked")));
        assert!(logs
            .iter()
            .any(|log| log.contains("~ a warrior was here ~")));
    }

    #[test]
    fn fail_feature_disabled() {
        let (mut mollusk, instruction, _) = setup(&[0xDE, 0xAD]);
        mollusk
            .feature_set
            .deactivate(&provide_instruction_data_offset_in_vm_r2::id());
        mollusk.process_and_validate_instruction(
            &instruction,
            &[],
            &[Check::instruction_err(
                InstructionError::ProgramFailedToComplete,
            )],
        );
    }
}
