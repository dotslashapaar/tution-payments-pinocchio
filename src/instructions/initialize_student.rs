use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey, seeds, sysvars::{clock::Clock, rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::{FreezeAccount, SetAuthority};

use crate::{student_account::StudentAccount, uni_account::UniAccount, vire_account::VireAccount};



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

        // Adding(setting-up(path)) the data to state (Read-Write) 
        let mut uni_account_data = *bytemuck::try_from_bytes_mut::<UniAccount>(&mut uni_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;   

        // Adding(setting-up(path)) the data to state (Read-Write)
        let mut student_account_data = *bytemuck::try_from_bytes_mut::<StudentAccount>(&mut student_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;  

        // Adding(setting-up(path)) the data to state (Read-Write)
        let vire_account_data = *bytemuck::try_from_bytes_mut::<VireAccount>(&mut vire_account.try_borrow_mut_data()?)
        .map_err(|_| ProgramError::InvalidAccountData)?;   

        // doing some checks for accounts
        assert!(student.is_signer());
        assert!(subject_account.is_owned_by(&crate::ID));
        assert!(uni_account.is_owned_by(&crate::ID));
        assert!(vire_account.is_owned_by(&crate::ID)); 

        let student_seeds_with_bump = &[student.key().as_ref(), subject_account.key().as_ref(), &[args.bump]];
        let student_account_derived = pubkey::create_program_address(student_seeds_with_bump, &crate::ID)?;

        // checking both created pda account and input pda accounts are same
        assert!(student_account_derived == student_account.key().as_ref());
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
        .invoke_signed(&[signer])?;

        

        let current_time = ((Clock::get()?.unix_timestamp)).to_le_bytes();

        // Adding(setting-up(adding)) the data to state (Read-Write)
        student_account_data.clone_from(&StudentAccount { 
            student_key: *student.key(), 
            student_id: uni_account_data.student_number, 
            time_start:  unsafe { std::mem::transmute(current_time) }, //<---- how can i make it better
            semesters: (1u64).to_le_bytes(), 
            student_bump: args.bump 
        });

        uni_account_data.student_number = (u64::from_le_bytes(uni_account_data.student_number) + 1).to_le_bytes();

        // <---Minting Card Nft---> (How can I add metadata)

        pinocchio_token::instructions::InitializeMint2{
            mint: card_mint,
            decimals: 0,
            mint_authority: vire_account.key(),
            freeze_authority: Some(vire_account.key()),
        }.invoke()?; //<---- who is the payer

        pinocchio_token::instructions::MintToChecked{
            mint: card_mint,
            account: student_card_ata,
            mint_authority: vire_account,
            amount: 1,
            decimals: 0, 
        }.invoke()?; //<---- who is the payer



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


        SetAuthority{
            account: student_card_ata,
            authority: student,
            authority_type: pinocchio_token::instructions::AuthorityType::FreezeAccount,
            new_authority: Some(vire_account.key()),
        }.invoke()?;

        FreezeAccount{
            account: student_card_ata,
            mint: card_mint,
            freeze_authority: vire_account,
        }
        .invoke_signed(&[signer])?;

    

        Ok(())
    }
}