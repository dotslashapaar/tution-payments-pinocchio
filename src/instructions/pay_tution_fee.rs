use bytemuck;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use pinocchio_token::state::Mint;

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


        let uni_account_data = *bytemuck::try_from_bytes_mut::<UniAccount>(&mut uni_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;   

        let mut student_account_data = *bytemuck::try_from_bytes_mut::<StudentAccount>(&mut student_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;  

        let vire_account_data = *bytemuck::try_from_bytes::<VireAccount>(&mut vire_account.try_borrow_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;  

        let subject_account_data = *bytemuck::try_from_bytes_mut::<SubjectAccount>(&mut subject_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;    

        assert!(student_account_data.semesters < subject_account_data.max_semester);

        if u64::from_le_bytes(student_account_data.semesters) == 1 {
            let current_time = ((Clock::get()?.unix_timestamp)).to_le_bytes();
            student_account_data.time_start = unsafe { std::mem::transmute(current_time) };

        }

        // doing some checks for accounts
        assert!(student.is_signer());
        assert!(uni_admin.key() == &uni_account_data.uni_key);
        assert!(student_account.is_owned_by(&crate::ID));
        assert!(subject_account.is_owned_by(&crate::ID));
        assert!(uni_account.is_owned_by(&crate::ID));
        assert!(vire_account.is_owned_by(&crate::ID)); 

        assert!(student_ata_usdc.is_owned_by(student.key()));
        assert!(uni_ata_usdc.is_owned_by(uni_admin.key()));
        assert!(treasury.is_owned_by(vire_account.key()));

        // First convert all byte arrays to u64
        let transaction_fee = u64::from_le_bytes(vire_account_data.transaction_fee_student);
        let tution_fee = u64::from_le_bytes(subject_account_data.tution_fee);
        let max_semester = u64::from_le_bytes(subject_account_data.max_semester);

        // Now perform calculations with compatible numeric types
        let tution_fee_per_sem = tution_fee.checked_div(max_semester).unwrap();
        let protocol_fee = tution_fee_per_sem.checked_div(100).unwrap() * transaction_fee;

        // student to treasury
        pinocchio_token::instructions::TransferChecked{
            from: student_ata_usdc,
            mint: mint_usdc,
            to: treasury,
            authority: student,
            amount: protocol_fee,
            decimals: Mint::from_account_info(mint_usdc)?.decimals(),
        }
        .invoke()?;

        // student to uni_ata_usdc
        pinocchio_token::instructions::TransferChecked{
            from: student_ata_usdc,
            mint: mint_usdc,
            to: uni_ata_usdc,
            authority: student,
            amount: tution_fee_per_sem,
            decimals: Mint::from_account_info(mint_usdc)?.decimals(),
        };

        // Increasing semesters number in student_account pda by 1 (student_account_data.semesters += 1)
        student_account_data.semesters = (u64::from_le_bytes(student_account_data.semesters) + 1).to_le_bytes();

        Ok(())
    }
}