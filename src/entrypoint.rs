use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint_deprecated::ProgramResult, msg,
    pubkey::Pubkey,
};

use crate::processor;

entrypoint!(process_instruction);

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!(
        "process_instruction: {}: {} accounts, data{:?}",
        program_id,
        accounts.len(),
        instruction_data
    );

    processor::process_instruction(program_id, accounts, instruction_data)
}
