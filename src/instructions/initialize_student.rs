use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey, seeds, sysvars::{clock::Clock, rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::{FreezeAccount, SetAuthority};

use crate::{student_account::StudentAccount, uni_account::UniAccount};



#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct StudentArgs {
    bump: u8,
}

impl TryFrom<&[u8]> for StudentArgs {
    type Error = ProgramError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            bytemuck::try_from_bytes::<Self>(value)
                .map(|reference| *reference)
                .map_err(|_| ProgramError::InvalidInstructionData)
        
    }
}

pub trait InitializeStudentContext<'a> {
    fn initialize_student(&self, args: &StudentArgs) -> ProgramResult;
}

impl <'a> InitializeStudentContext <'a> for &[AccountInfo] {
    fn initialize_student(&self, args: &StudentArgs) -> ProgramResult {
        // all the required accounts for the this instruction
        let [
            student, 
            student_account,
            subject_account,
            uni_account,
            vire_account, 
            card_mint,
            student_card_ata,
            _system_program, 
            _token_program,
            ] = self 
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
 

        // These checks are not compulsory
        // Doing some checks for accounts
        // Check if student is a signer
        // if !student.is_signer() {
        //     return Err(ProgramError::MissingRequiredSignature);
        // }

        // // Verify subject_account is owned by the current program
        // if !subject_account.is_owned_by(&crate::ID) {
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        // // Verify uni_account is owned by the current program
        // if !uni_account.is_owned_by(&crate::ID) {
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        // // Verify vire_account is owned by the current program
        // if !vire_account.is_owned_by(&crate::ID) {
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        
        let student_seeds_with_bump = &[student.key().as_ref(), subject_account.key().as_ref(), &[args.bump]];
        let student_account_derived = pubkey::create_program_address(student_seeds_with_bump, &crate::ID)?;

        // Ensure derived PDA matches the provided student_account
        if student_account_derived != student_account.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }
        let bump_ref = &[args.bump];
        
        // creating signer seeds vire pda 
        let signer_seeds = seeds!(student.key().as_ref(), subject_account.key().as_ref(), bump_ref);
        let signer = Signer::from(&signer_seeds);

        CreateAccount{
            from: student,
            to: student_account,
            space: StudentAccount::LEN as u64,
            owner: &crate::ID,
            lamports: Rent::get()?.minimum_balance(StudentAccount::LEN),
        }
        .invoke_signed(&[signer.clone()])?;


        let mut uni_data_ref = uni_account.try_borrow_mut_data()?;
        let uni_account_data = bytemuck::try_from_bytes_mut::<UniAccount>(&mut uni_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let mut student_data_ref = student_account.try_borrow_mut_data()?;
        let student_account_data = bytemuck::try_from_bytes_mut::<StudentAccount>(&mut student_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        

        let current_time = ((Clock::get()?.unix_timestamp)).to_le_bytes();
        // let current_time: i64 = Clock::get()?.unix_timestamp;

        student_account_data.student_key = *student.key();
        student_account_data.student_id = uni_account_data.student_number;
        student_account_data.time_start = current_time; 
        student_account_data.semesters = (1u64).to_le_bytes();
        student_account_data.student_bump = args.bump;

        

        uni_account_data.student_number = (u64::from_le_bytes(uni_account_data.student_number) + 1).to_le_bytes();


        // <---Minting Card Nft---> (How can I add metadata (In FrontEnd))


        // // Do the Initialine part in frontend for better CU's
        // pinocchio_token::instructions::InitializeMint2{   //(Thinking to be optional (doing this in frontend))
        //     mint: card_mint,
        //     decimals: 0,
        //     mint_authority: student_account.key(),
        //     freeze_authority: Some(student_account.key()),
        // }.invoke()?; 

        pinocchio_token::instructions::MintToChecked{
            mint: card_mint,
            account: student_card_ata,
            mint_authority: student_account,
            amount: 1,
            decimals: 0, 
        }
        .invoke_signed(&[signer.clone()])?; 


        // <---Staking(Freezing)---> 

        SetAuthority{
            account: student_card_ata,
            authority: student, 
            authority_type: pinocchio_token::instructions::AuthorityType::FreezeAccount,
            new_authority: Some(student_account.key()),
        }.invoke()?;

        FreezeAccount{
            account: student_card_ata,
            mint: card_mint,
            freeze_authority: student_account,
        }
        .invoke_signed(&[signer])?;

    

        Ok(())
    }
}