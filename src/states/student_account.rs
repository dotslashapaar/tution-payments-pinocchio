use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct StudentAccount{
    pub student_key: Pubkey,
    pub student_id: [u8; 8],
    pub time_start:  [u8; 8], //<----- for time
    pub semesters: [u8; 8],
    pub student_bump: u8,
}

impl StudentAccount {
    pub const LEN: usize = core::mem::size_of::<StudentAccount>();
}


// seeds = [student.key().as_ref(), subject_account.key().as_ref()]