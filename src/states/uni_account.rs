use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UniAccount{
    pub uni_key: Pubkey,
    pub vire_key: Pubkey,
    pub uni_id: [u8; 8],
    pub subject_number: [u8; 8],
    pub student_number: [u8; 8],
    pub uni_bump: u8,
}

impl UniAccount {
    pub const LEN: usize = core::mem::size_of::<UniAccount>();
}

// seeds = [uni_admin.key().as_ref(), vire_account.key().as_ref()]