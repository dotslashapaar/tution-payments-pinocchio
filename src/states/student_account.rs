use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)] //(checkout pod u16...)
pub struct StudentAccount{
    pub student_key: Pubkey,
    pub student_id: [u8; 8],//[8,0,0,0,0,0,0,0]
    pub time_start:  [u8; 8],
    // pub time_start:  i64, //<----- for time
    pub semesters: [u8; 8],
    pub student_bump: u8,
    // pub _padding: [u8; 7]
}



impl StudentAccount {
    pub const LEN: usize = core::mem::size_of::<StudentAccount>();
}


// seeds = [student.key().as_ref(), subject_account.key().as_ref()]