

use instructions::{add_subjects::AddSubjectContext, initialize_student::InitializeStudentContext, initialize_uni::InitializeUniContext, initialize_vire::InitializeVireContext, pay_tution_fee::PayTutionFeeContext, unstake::Unstake, vire_instructions::VireInstruction};
use pinocchio::{account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult};


mod instructions;
mod states;

pub use states::*;


entrypoint!(process_instruction);

pinocchio_pubkey::declare_id!("Hh6AGqBdAeXJF64MmkLrV5yD3citghoEh4MDyh4rHy9j");

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult{
    let (instruction, data) = instruction_data
    .split_first()
    .ok_or(ProgramError::InvalidInstructionData)?;

    match VireInstruction::try_from(instruction)? {
        VireInstruction::InitializeVire => accounts.initialize_vire(&data.try_into()?),
        VireInstruction::InitializeUni => accounts.initialize_uni(&data.try_into()?),
        VireInstruction::AddSubjects => accounts.add_subject(&data.try_into()?),
        VireInstruction::InitializeStudent => accounts.initialize_student(&data.try_into()?),
        VireInstruction::PayTutionFee => accounts.pay_tution_fee(),
        VireInstruction::UnStake => accounts.unstake(),
    }

    
}
