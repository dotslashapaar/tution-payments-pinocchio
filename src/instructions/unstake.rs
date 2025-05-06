use bytemuck;
use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey, seeds, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use pinocchio_token::instructions::{SetAuthority, ThawAccount};

use crate::{student_account::StudentAccount, subject_account::SubjectAccount};




pub trait Unstake<'a> {
    fn unstake(&self) -> ProgramResult;
}


impl <'a> Unstake<'a> for &[AccountInfo] {
    fn unstake(&self) -> ProgramResult {
        // all the required accounts for the this instruction
        let [
            student, 
            student_account,
            subject_account,
            card_mint,
            student_card_ata,
            _system_program, 
            _token_program,
            ] = self 
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };


        // Use read-only access for accounts we don't modify
        let student_data_ref = student_account.try_borrow_data()?;
        let student_account_data = bytemuck::try_from_bytes::<StudentAccount>(&student_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        
   
        
        
        let subject_data_ref = subject_account.try_borrow_data()?;
        let subject_account_data = bytemuck::try_from_bytes::<SubjectAccount>(&subject_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;


        // Check if student has completed all required semesters
        if student_account_data.semesters != subject_account_data.max_semester {
            return Err(ProgramError::InvalidArgument);
        }

        // Time calculations 
        const SECONDS_IN_A_MONTH: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
        
        let max_semester = i64::from_le_bytes(subject_account_data.max_semester);
        let semester_months = i64::from_le_bytes(subject_account_data.semester_months);
        let required_wait_time = SECONDS_IN_A_MONTH * max_semester * semester_months;
        
        let current_timestamp = Clock::get()?.unix_timestamp;
        let start_timestamp = i64::from_le_bytes(student_account_data.time_start);
        // let start_timestamp = student_account_data.time_start;


        // Main check (Verifies whether the degree duration has been completed)
        // Check that degree duration is not yet completed
        if current_timestamp - start_timestamp < required_wait_time {
            return Err(ProgramError::InvalidArgument);
        }

        
        // // These checks are not COMPULSORY
        // // Doing some checks for accounts
        // // Check if student is a signer
        // if !student.is_signer() {
        //     return Err(ProgramError::MissingRequiredSignature);
        // }

        // // // Verify student_account is owned by the current program
        // // if !student_account.is_owned_by(&crate::ID) {
        // //     return Err(ProgramError::IncorrectProgramId);
        // // }

        // // // Verify subject_account is owned by the current program
        // // if !subject_account.is_owned_by(&crate::ID) {
        // //     return Err(ProgramError::IncorrectProgramId);
        // // } //(super extra)
        

        let student_account_seeds = &[student.key().as_ref(), subject_account.key().as_ref()];
        let (student_account_derived, student_account_bump) = 
            pubkey::try_find_program_address(student_account_seeds, &crate::ID )
            .ok_or(ProgramError::InvalidSeeds)?;

        // Ensure derived PDA matches the provided student_account
        if student_account_derived != student_account.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }
        let bump_ref = &[student_account_bump];
        
        // creating signer seeds vire pda 
        let signer_seeds = seeds!(student.key().as_ref(), subject_account.key().as_ref(), bump_ref);
        let signer = Signer::from(&signer_seeds);

        ThawAccount{
            account: student_card_ata,
            mint: card_mint,
            freeze_authority: student_account,
        }
        .invoke_signed(&[signer.clone()])?;
        
        SetAuthority{
            account: student_card_ata,
            authority: student_account,
            authority_type: pinocchio_token::instructions::AuthorityType::FreezeAccount,
            new_authority: Some(student.key()),
        }
        .invoke_signed(&[signer])?;


       

        Ok(())
    }
}