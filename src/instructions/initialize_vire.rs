use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey, seeds, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::InitializeAccount;

use crate::vire_account::VireAccount;



#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VireArgs {
    transaction_fee_uni: [u8; 8], 
    transaction_fee_student: [u8; 8],
    bump: u8,
}


impl VireArgs {
    fn transaction_fee_uni(&self) -> u64 {
        u64::from_le_bytes(self.transaction_fee_uni)
    }

    fn transaction_fee_student(&self) -> u64 {
        u64::from_le_bytes(self.transaction_fee_student)
    }
}

impl TryFrom<&[u8]> for VireArgs {
    type Error = ProgramError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            bytemuck::try_from_bytes::<Self>(value)
                .map(|reference| *reference)
                .map_err(|_| ProgramError::InvalidInstructionData)
        
    }
}

pub trait InitializeVireContext<'a> {
    fn initialize_vire(&self, args: &VireArgs) -> ProgramResult;
}

impl <'a> InitializeVireContext <'a> for &[AccountInfo] {
    fn initialize_vire(&self, args: &VireArgs) -> ProgramResult {
        // all the required accounts for the this instruction
        let [
            admin, 
            mint_usdc, 
            vire_account, 
            treasury, 
            _system_program, 
            _token_program
            ] = self 
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

          

        // doing some checks for accounts
        if !admin.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let vire_seeds_with_bump = &[b"vire", admin.key().as_ref(), &[args.bump]];
        let vire_account_derived = pubkey::create_program_address(vire_seeds_with_bump, &crate::ID)?;

        // checking both created pda account and input pda accounts are same
        if vire_account_derived != vire_account.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }
        let bump_ref = &[args.bump];
        
        // creating signer seeds vire pda 
        let signer_seeds = seeds!(b"vire", admin.key().as_ref(), bump_ref);
        let signer = Signer::from(&signer_seeds);

        CreateAccount{
            from: admin,
            to: vire_account,
            space: VireAccount::LEN as u64,
            owner: &crate::ID,
            lamports: Rent::get()?.minimum_balance(VireAccount::LEN),
        }
        .invoke_signed(&[signer])?;


        // Adding(setting-up(path)) the data to state (Read-Write)
        let mut account_data_ref  = vire_account.try_borrow_mut_data()?;
        let vire_account_data = bytemuck::try_from_bytes_mut::<VireAccount>(&mut account_data_ref )
        .map_err(|_| ProgramError::InvalidAccountData)?;  


        // Adding(setting-up(adding)) the data to state (Read-Write)
        // <--References (Zero-Copy)-->
        vire_account_data.admin_key = *admin.key();
        vire_account_data.uni_number = (1u64).to_le_bytes(); //<---------- explain please (any other options)
        vire_account_data.transaction_fee_uni = args.transaction_fee_uni;
        vire_account_data.transaction_fee_student = args.transaction_fee_student;
        vire_account_data.vire_bump = args.bump;

        // Adding(setting-up(adding)) the data to state (Read-Write)
        // <--Dereferencing (Copying)-->
        // vire_account_data.clone_from(&VireAccount { 
        //     admin_key: *admin.key(), 
        //     uni_number: (1u64).to_le_bytes(), //<----------
        //     transaction_fee_uni: args.transaction_fee_uni, 
        //     transaction_fee_student: args.transaction_fee_student, 
        //     vire_bump: args.bump 
        // });

        // pinocchio_token::instructions::InitializeAccount3{ //(Do it in front-end for better CU's)
        //     account: treasury,
        //     mint: mint_usdc,
        //     owner: vire_account.key(),
        // }.invoke()?;

        Ok(())
    }
}