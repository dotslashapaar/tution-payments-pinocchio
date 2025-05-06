use bytemuck;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use pinocchio_token::state::{Mint, TokenAccount};

use crate::{student_account::StudentAccount, subject_account::SubjectAccount, uni_account::UniAccount, vire_account::VireAccount};




pub trait PayTutionFeeContext<'a> {
    fn pay_tution_fee(&self) -> ProgramResult;
}


impl <'a> PayTutionFeeContext<'a> for &[AccountInfo] {
    fn pay_tution_fee(&self) -> ProgramResult {
        // all the required accounts for the this instruction
        let [
            student, 
            mint_usdc,
            uni_admin,
            student_account,
            student_ata_usdc,
            subject_account,
            uni_account,
            uni_ata_usdc,
            vire_account, 
            treasury,
            _system_program, 
            _token_program
            ] = self 
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Read-only account data access first
        let vire_data_ref = vire_account.try_borrow_data()?;
        let vire_account_data = bytemuck::try_from_bytes::<VireAccount>(&vire_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;  

        let uni_data_ref = uni_account.try_borrow_data()?;
        let uni_account_data = bytemuck::try_from_bytes::<UniAccount>(&uni_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        
        let subject_data_ref = subject_account.try_borrow_data()?;
        let subject_account_data = bytemuck::try_from_bytes::<SubjectAccount>(&subject_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;  

        // Mutable account data access
        let mut student_data_ref = student_account.try_borrow_mut_data()?;
        let student_account_data = bytemuck::try_from_bytes_mut::<StudentAccount>(&mut student_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;


        let student_semesters = u64::from_le_bytes(student_account_data.semesters);
        let max_semesters = u64::from_le_bytes(subject_account_data.max_semester);

        // Check if student has not exceeded max semesters
        if student_semesters > max_semesters {
            return Err(ProgramError::InvalidArgument);
        }


        // Update first semester time if needed
        if student_semesters == 1 {
            let current_time = Clock::get()?.unix_timestamp.to_le_bytes();
            // let current_time = Clock::get()?.unix_timestamp;
            student_account_data.time_start = current_time; 
        }


        // // // These checks are not COMPULSORY
        // // Doing some checks for accounts
        // // Check if student is a signer
        // if !student.is_signer() {
        //     return Err(ProgramError::MissingRequiredSignature);
        // }

        // // Verify uni_admin key matches the stored key in uni_account
        // if uni_admin.key() != &uni_account_data.uni_key {
        //     return Err(ProgramError::InvalidArgument);
        // }

        // // Verify program-owned accounts
        // if !student_account.is_owned_by(&crate::ID) {//(not compsulion)
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        // if !subject_account.is_owned_by(&crate::ID) {
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        // if !uni_account.is_owned_by(&crate::ID) {
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        // if !vire_account.is_owned_by(&crate::ID) {
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        // // Use from_account_info
        // // // Verify token account ownerships
        // // if !TokenAccount::from_account_info(student_ata_usdc)?;.(student.key()) { //token.
        // //     return Err(ProgramError::IllegalOwner);
        // // }

        // // if !uni_ata_usdc.is_owned_by(uni_admin.key()) {
        // //     return Err(ProgramError::IllegalOwner);
        // // }

        // // if !treasury.is_owned_by(vire_account.key()) {
        // //     return Err(ProgramError::IllegalOwner);
        // // }

        // // {
        // //     let student_ata_usdc = TokenAccount::from_account_info(student_ata_usdc)?.owner();

        // // }

        // Fee calculations
        let transaction_fee = u64::from_le_bytes(vire_account_data.transaction_fee_student);
        let tution_fee = u64::from_le_bytes(subject_account_data.tution_fee);

        // Final Fee Distribution
        let tution_fee_per_sem = tution_fee.checked_div(max_semesters).unwrap();
        let protocol_fee = tution_fee_per_sem.checked_div(100).unwrap() * transaction_fee;

        // student to treasury
        pinocchio_token::instructions::TransferChecked{
            from: student_ata_usdc,
            mint: mint_usdc,
            to: treasury,
            authority: student,
            amount: protocol_fee,
            decimals: Mint::from_account_info(mint_usdc)?.decimals(),
        }.invoke()?;

        // student to uni_ata_usdc
        pinocchio_token::instructions::TransferChecked{
            from: student_ata_usdc,
            mint: mint_usdc,
            to: uni_ata_usdc,
            authority: student,
            amount: tution_fee_per_sem,
            decimals: Mint::from_account_info(mint_usdc)?.decimals(),
        }.invoke()?;

        // Increasing semesters number in student_account pda by 1 (student_account_data.semesters += 1)
        student_account_data.semesters = (u64::from_le_bytes(student_account_data.semesters) + 1).to_le_bytes();

        Ok(())
    }
}