use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey, seeds, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;


use crate::{uni_account::UniAccount, vire_account::VireAccount};



#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UniArgs {
    bump: u8,
}

impl TryFrom<&[u8]> for UniArgs {
    type Error = ProgramError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            bytemuck::try_from_bytes::<Self>(value)
                .map(|reference| *reference)
                .map_err(|_| ProgramError::InvalidInstructionData)
        
    }
}

pub trait InitializeUniContext<'a> {
    fn initialize_uni(&self, args: &UniArgs) -> ProgramResult;
}

impl <'a> InitializeUniContext <'a> for &[AccountInfo] {
    fn initialize_uni(&self, args: &UniArgs) -> ProgramResult {
        // all the required accounts for the this instruction
        let [
            uni_admin, 
            uni_account,
            vire_account, 
            _system_program, 
            ] = self 
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };


        // Check if uni_admin is a signer
        if !uni_admin.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Verify vire_account is owned by the current program
        if !vire_account.is_owned_by(&crate::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        let uni_seeds_with_bump = &[uni_admin.key().as_ref(), vire_account.key().as_ref(), &[args.bump]];
        let uni_account_derived = pubkey::create_program_address(uni_seeds_with_bump, &crate::ID)?;

        // Ensure derived PDA matches the provided uni_account
        if uni_account_derived != uni_account.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }
        let bump_ref = &[args.bump];
        
        // creating signer seeds vire pda 
        let signer_seeds = seeds!(uni_admin.key().as_ref(), vire_account.key().as_ref(), bump_ref);
        let signer = Signer::from(&signer_seeds);

        CreateAccount{
            from: uni_admin,
            to: uni_account,
            space: UniAccount::LEN as u64,
            owner: &crate::ID,
            lamports: Rent::get()?.minimum_balance(UniAccount::LEN),
        }
        .invoke_signed(&[signer])?;

        let mut vire_data_ref = vire_account.try_borrow_mut_data()?;
        let vire_account_data = bytemuck::try_from_bytes_mut::<VireAccount>(&mut vire_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let mut uni_data_ref = uni_account.try_borrow_mut_data()?;
        let uni_account_data = bytemuck::try_from_bytes_mut::<UniAccount>(&mut uni_data_ref)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        // Direct field assignments (zero-copy approach)
        uni_account_data.uni_key = *uni_account.key();
        uni_account_data.vire_key = *vire_account.key();
        uni_account_data.uni_id = vire_account_data.uni_number;
        uni_account_data.subject_number = (0u64).to_le_bytes();     
        uni_account_data.student_number = (0u64).to_le_bytes();     
        uni_account_data.uni_bump = args.bump;

        

        // increasing uni_number in vire_account by 1 (vire_account_data.uni_number += 1;)
        let mut num = u64::from_le_bytes(vire_account_data.uni_number);
        num += 1;
        vire_account_data.uni_number = num.to_le_bytes();

        Ok(())
    }
}