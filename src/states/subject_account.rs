use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SubjectAccount{
    pub uni_key: Pubkey,
    pub subject_code: [u8; 8],
    pub tution_fee: [u8; 8],
    pub max_semester: [u8; 8],
    pub semester_months: [u8; 8],
    pub subject_bump: u8,
}

impl SubjectAccount {
    pub const LEN: usize = core::mem::size_of::<SubjectAccount>();
}

// seeds = [uni_account.key().as_ref(), &[uni_account.subject_number]]