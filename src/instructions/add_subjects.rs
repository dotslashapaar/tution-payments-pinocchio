use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey::{self}, seeds, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::{InitializeMint2, MintToChecked}, state::Mint};

use crate::{subject_account::SubjectAccount, uni_account::UniAccount, vire_account::VireAccount};



#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SubjectArgs {
    tution_fee: [u8; 8],
    max_semester: [u8; 8], 
    semester_months: [u8; 8],
    bump: u8,
}

impl SubjectArgs {
    fn tution_fee(&self) -> u64 {
        u64::from_le_bytes(self.tution_fee)
    }

    fn max_semester(&self) -> u64 {
        u64::from_le_bytes(self.max_semester)
    }

    fn semester_months(&self) -> u64 {
        u64::from_le_bytes(self.semester_months)
    }
}

impl TryFrom<&[u8]> for SubjectArgs {
    type Error = ProgramError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            bytemuck::try_from_bytes::<Self>(value)
                .map(|reference| *reference)
                .map_err(|_| ProgramError::InvalidInstructionData)
    }
}

pub trait AddSubjectContext<'a> {
    fn add_subject(&self, args: &SubjectArgs) -> ProgramResult;
}

impl <'a> AddSubjectContext <'a> for &[AccountInfo] {
    fn add_subject(&self, args: &SubjectArgs) -> ProgramResult {
        // all the required accounts for the this instruction
        let [
            uni_admin, 
            mint_usdc, 
            subject_account,
            uni_account, 
            uni_ata_usdc, 
            vire_account,  
            treasury,
            collection_mint,
            uni_collection_ata,
            _system_program,  
            _token_program 
            ] = self 
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // // These checks are not Compulsory
        // // Doing some checks for accounts
        // // Check if uni_admin is a signer
        // if !uni_admin.is_signer() {
        //     return Err(ProgramError::MissingRequiredSignature);
        // }

        // // Verify uni_account is owned by the current program
        // if !uni_account.is_owned_by(&crate::ID) {
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        // // Verify vire_account is owned by the current program
        // if !vire_account.is_owned_by(&crate::ID) {
        //     return Err(ProgramError::IncorrectProgramId);
        // }

        let mut uni_data_ref_mut = uni_account.try_borrow_mut_data()?;
        let uni_account_data = bytemuck::try_from_bytes_mut::<UniAccount>(&mut uni_data_ref_mut)
            .map_err(|_| ProgramError::InvalidAccountData)?; 

        let vire_data_ref = vire_account.try_borrow_data()?;
        let vire_account_data = bytemuck::try_from_bytes::<VireAccount>(&vire_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let subject_seeds_with_bump = &[
            uni_account.key().as_ref(), 
            (&[u64::from_le_bytes((uni_account_data.subject_number).clone()).try_into().unwrap()]), 
            &[args.bump]
            ];
        let subject_account_derived = pubkey::create_program_address(subject_seeds_with_bump, &crate::ID)?;

        // Ensure derived PDA matches the provided subject_account
        if subject_account_derived != subject_account.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }
        let bump_ref = &[args.bump];  
        

        // creating signer seeds vire pda (subject_acccount)
        let binding = [u64::from_le_bytes(uni_account_data.subject_number).try_into().unwrap()];
        let signer_seeds = seeds!(uni_account.key().as_ref(), &binding, bump_ref);
        let signer = Signer::from(&signer_seeds);


        CreateAccount{
            from: uni_admin,
            to: subject_account,
            space: SubjectAccount::LEN as u64,
            owner: &crate::ID,
            lamports: Rent::get()?.minimum_balance(SubjectAccount::LEN),
        }
        .invoke_signed(&[signer.clone()])?;

        let mut subject_data_ref = subject_account.try_borrow_mut_data()?;
        let subject_account_data = bytemuck::try_from_bytes_mut::<SubjectAccount>(&mut subject_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        

        // <---Filling subect_account pda--->
        // Direct field assignments for zero-copy
        subject_account_data.uni_key = *uni_account.key();
        subject_account_data.subject_code = uni_account_data.subject_number;
        subject_account_data.tution_fee = args.tution_fee;
        subject_account_data.max_semester = args.max_semester;
        subject_account_data.semester_months = args.semester_months;
        subject_account_data.subject_bump = args.bump;

    
        // Increasing subject number in uni_account pda by 1 (uni_account_data.subject_number += 1)
        uni_account_data.subject_number = (u64::from_le_bytes(uni_account_data.subject_number) + 1).to_le_bytes();



        // <---Uni Paying Fee to Protocal---> 

        // Calculating Tution Fee in Percent
        let transaction_fee = u64::from_le_bytes(vire_account_data.transaction_fee_uni);
        let fee = ((args.tution_fee()).checked_div(100).unwrap()) * transaction_fee;

        // sending mint_usdc token (uni_ata_usdc --mint_usdc--> treasury)
        pinocchio_token::instructions::TransferChecked{
            from: uni_ata_usdc,
            to: treasury,
            mint: mint_usdc,
            amount: fee,
            authority: uni_admin,
            decimals: Mint::from_account_info(mint_usdc)?.decimals()
        }.invoke()?;



        // <---Making Collection For Subject---> (How can I add metadata (In FrontEnd))


        // // Do the Initialine part in frontend for better CU's
        // InitializeMint2{    //(Thinking to be optional (doing this in frontend))
        //     mint: collection_mint,
        //     decimals: 0,
        //     mint_authority: subject_account.key(),
        //     freeze_authority: Some(subject_account.key()),
        // }.invoke()?; 

        MintToChecked{
            mint: collection_mint,
            account: uni_collection_ata,
            mint_authority: subject_account,
            amount: 1,
            decimals: 0, 
        }.invoke_signed(&[signer])?; 


        Ok(())
    }
}







// use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

// fn main() -> ProgramResult {
//     assert_with_error!(2==1, MyError::CustomError1);
//     assert_with_error!(2==1, ProgramError::ArithmeticOverflow);
//     Ok(())

// }

// #[macro_export]
// macro_rules! assert_with_error {
//     ($invariant:expr, $error:expr $(,)?) => {
//         if !($invariant) {
//             return Err(ProgramError::from($error));
//         }
//     };
//     ($invariant:expr, $error:path $(,)?) => {
//         if !($invariant) {
//             return Err($error);
//         }
//     };
// }

// pub enum MyError {
//     CustomError1,
//     CustomError2,
// }

// impl From<MyError> for ProgramError {
//     fn from(e: MyError) -> Self {
//         match e {
//             MyError::CustomError1 => ProgramError::Custom(1),
//             MyError::CustomError2 => ProgramError::Custom(2),
//         }
//     }
// }
