use pinocchio::program_error::ProgramError;


#[repr(u8)]
pub enum VireInstruction {
    InitializeVire,
    InitializeUni,
    AddSubjects,
    InitializeStudent,
    PayTutionFee,
    UnStake
}


impl TryFrom<&u8> for VireInstruction{
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::InitializeVire),
            1 => Ok(Self::InitializeUni),
            2 => Ok(Self::AddSubjects),
            3 => Ok(Self::InitializeStudent),
            4 => Ok(Self::PayTutionFee),
            5 => Ok(Self::UnStake),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

}




