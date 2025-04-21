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

        // doing some checks for accounts
        assert!(uni_admin.is_signer());
        assert!(uni_account.is_owned_by(&crate::ID));
        assert!(vire_account.is_owned_by(&crate::ID));       


        // Adding(setting-up(path)) the data to state (Read-Write)
        let mut uni_account_data = *bytemuck::try_from_bytes_mut::<UniAccount>(&mut uni_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;   

        // Adding(setting-up(path)) the data to state (Read-Write)
        let mut subject_account_data = *bytemuck::try_from_bytes_mut::<SubjectAccount>(&mut subject_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;    

        // Adding(setting-up(path)) the data to state (Read-Only)
        let vire_account_data = *bytemuck::try_from_bytes::<VireAccount>(&mut vire_account.try_borrow_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?; 


        let subject_seeds_with_bump = &[
            uni_account.key().as_ref(), 
            (&[u64::from_le_bytes((uni_account_data.subject_number).clone()).try_into().unwrap()]), 
            &[args.bump]
            ];
        let subject_account_derived = pubkey::create_program_address(subject_seeds_with_bump, &crate::ID)?;

        // checking both created pda account and input pda accounts are same
        assert!(subject_account_derived == subject_account.key().as_ref());
        let bump_ref = &[args.bump];  


        // <---Help--->
        // // creating signer seeds vire pda 
        // let signer_seeds = seeds!(uni_account.key().as_ref(), &[u64::from_le_bytes(uni_account_data.subject_number).try_into().unwrap()], bump_ref);
        // let signer = Signer::from(&signer_seeds);
        
        

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
        .invoke_signed(&[signer])?;

        

        // <---Filling subect_account pda--->
        // Adding(setting-up(adding)) the data to state (Read-Write)
        subject_account_data.clone_from(&SubjectAccount { 
            uni_key: *uni_account.key(), 
            subject_code: uni_account_data.subject_number, 
            tution_fee: args.tution_fee, 
            max_semester: args.max_semester, 
            semester_months: args.semester_months, 
            subject_bump: args.bump 
        }); 

    
        // Increasing subject number in uni_account pda by 1 (uni_account_data.subject_number += 1)
        uni_account_data.subject_number = (u64::from_le_bytes(uni_account_data.subject_number) + 1).to_le_bytes();



        // <---Uni Paying Fee to Protocal---> 


        // Calculating Tution Fee in Percent
        // Convert the byte array to a u64
        let transaction_fee = u64::from_ne_bytes(vire_account_data.transaction_fee_uni);
        // Now perform the multiplication with compatible types
        let fee = ((args.tution_fee()).checked_div(100).unwrap()) * transaction_fee;

        // sending mint_usdc token (uni_ata_usdc --mint_usdc--> treasury)
        pinocchio_token::instructions::TransferChecked{
            from: uni_ata_usdc,
            to: treasury,
            mint: mint_usdc,
            amount: fee,
            authority: uni_admin,
            decimals: Mint::from_account_info(mint_usdc)?.decimals()
        }
        .invoke()?;



        // <---Making Collection---> (How can I add metadata)

        // InitializeMint{
        //     mint: todo!(),
        //     rent_sysvar: todo!(), //<--------- Help
        //     decimals: todo!(),
        //     mint_authority: todo!(),
        //     freeze_authority: todo!(),
        // };
        
        InitializeMint2{
            mint: collection_mint,
            decimals: 0,
            mint_authority: vire_account.key(),
            freeze_authority: Some(vire_account.key()),
        }.invoke()?; //<---- who is the payer

        MintToChecked{
            mint: collection_mint,
            account: uni_collection_ata,
            mint_authority: vire_account,
            amount: 1,
            decimals: 0, 
        }.invoke()?; //<---- who is the payer


        Ok(())
    }
}








