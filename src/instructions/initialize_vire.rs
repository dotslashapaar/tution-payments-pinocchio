use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey, seeds, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;

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

         // Adding(setting-up(path)) the data to state (Read-Write)
         let mut vire_account_data = *bytemuck::try_from_bytes_mut::<VireAccount>(&mut vire_account.try_borrow_mut_data()?)
         .map_err(|_| ProgramError::InvalidAccountData)?;   

        // doing some checks for accounts
        assert!(admin.is_signer());

        let vire_seeds_with_bump = &[b"vire", admin.key().as_ref(), &[args.bump]];
        let vire_account_derived = pubkey::create_program_address(vire_seeds_with_bump, &crate::ID)?;

        // checking both created pda account and input pda accounts are same
        assert!(vire_account_derived == vire_account.key().as_ref());
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

       

        // Adding(setting-up(adding)) the data to state (Read-Write)
        vire_account_data.clone_from(&VireAccount { 
            admin_key: *admin.key(), 
            uni_number: (1u64).to_le_bytes(), //<----------
            transaction_fee_uni: args.transaction_fee_uni, 
            transaction_fee_student: args.transaction_fee_student, 
            vire_bump: args.bump 
        });

        pinocchio_token::instructions::InitializeAccount3{
            account: treasury,
            mint: mint_usdc,
            owner: vire_account.key(),
        };
        
        // InitializeAccount{
        //     account: todo!(),
        //     mint: todo!(),
        //     owner: todo!(),
        //     rent_sysvar: todo!(),  //<-------
        // };

        Ok(())
    }
}