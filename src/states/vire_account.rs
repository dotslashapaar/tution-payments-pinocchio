use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VireAccount{
    pub admin_key: Pubkey,
    pub uni_number: [u8; 8],
    pub transaction_fee_uni: [u8; 8],
    pub transaction_fee_student: [u8; 8],
    pub vire_bump: u8,
}

impl VireAccount {
    pub const LEN: usize = core::mem::size_of::<VireAccount>();
}

// seeds = [b"vire", admin.key().as_ref()]