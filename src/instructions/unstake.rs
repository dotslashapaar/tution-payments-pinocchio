use bytemuck;
use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey, seeds, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use pinocchio_token::instructions::{SetAuthority, ThawAccount};

use crate::{student_account::StudentAccount, subject_account::SubjectAccount, vire_account::VireAccount};




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
            vire_account, 
            card_mint,
            student_card_ata,
            _system_program, 
            _token_program,
            ] = self 
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };


        let student_account_data = *bytemuck::try_from_bytes_mut::<StudentAccount>(&mut student_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;  

        let vire_account_data = *bytemuck::try_from_bytes::<VireAccount>(&mut vire_account.try_borrow_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;  

        let subject_account_data = *bytemuck::try_from_bytes_mut::<SubjectAccount>(&mut subject_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;    


        assert!(student_account_data.semesters == subject_account_data.max_semester);

        const SECONDS_IN_A_MONTH: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
        
        let max_semester = i64::from_le_bytes(subject_account_data.max_semester);
        let semester_months = i64::from_le_bytes(subject_account_data.semester_months);

        let required_wait_time: i64 = SECONDS_IN_A_MONTH * max_semester * semester_months;

        let current_time = ((Clock::get()?.unix_timestamp)).to_le_bytes();

        let current_timestamp = i64::from_le_bytes(current_time);
        let start_timestamp = i64::from_le_bytes(student_account_data.time_start);

        // Main check (Verifies whether the degree duration has been completed)
        assert!(current_timestamp - start_timestamp < required_wait_time);

        // doing some checks for accounts
        assert!(student.is_signer());
        assert!(student_account.is_owned_by(&crate::ID));
        assert!(subject_account.is_owned_by(&crate::ID));
        assert!(vire_account.is_owned_by(&crate::ID)); 

        let vire_account_seeds = &[b"vire", vire_account_data.admin_key.as_ref()];
        let (vire_account_derived, vire_account_bump) = 
            pubkey::try_find_program_address(vire_account_seeds, &crate::ID )
            .ok_or(ProgramError::InvalidSeeds)?;

        // checking both created pda account and input pda accounts are same
        assert!(vire_account_derived == vire_account.key().as_ref());
        let bump_ref = &[vire_account_bump];
        
        // creating signer seeds vire pda 
        let signer_seeds = seeds!(b"vire", vire_account_data.admin_key.as_ref(), bump_ref);
        let signer = Signer::from(&signer_seeds);
        let signer1 = Signer::from(&signer_seeds);

        ThawAccount{
            account: student_card_ata,
            mint: card_mint,
            freeze_authority: vire_account,
        }
        .invoke_signed(&[signer])?;
        
        SetAuthority{
            account: student_card_ata,
            authority: vire_account,
            authority_type: pinocchio_token::instructions::AuthorityType::FreezeAccount,
            new_authority: Some(student.key()),
        }
        .invoke_signed(&[signer1])?;


       

        Ok(())
    }
}