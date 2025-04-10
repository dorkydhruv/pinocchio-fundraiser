use pinocchio::program_error::ProgramError;

pub mod initialize;
pub use initialize::*;

#[repr(u8)]
pub enum ProgramInstruction {
    Initialize,
    Contribute,
    CheckContribution,
    Refund,
}

impl TryFrom<&u8> for ProgramInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(ProgramInstruction::Initialize),
            1 => Ok(ProgramInstruction::Contribute),
            2 => Ok(ProgramInstruction::CheckContribution),
            3 => Ok(ProgramInstruction::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
