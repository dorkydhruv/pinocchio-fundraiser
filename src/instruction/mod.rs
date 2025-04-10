use pinocchio::program_error::ProgramError;

pub mod initialize;
pub mod contribute;
pub mod checker;
pub mod refund;
pub use initialize::*;
pub use contribute::*;
pub use checker::*;
pub use refund::*;

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
