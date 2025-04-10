use pinocchio::program_error::ProgramError;

#[derive(Clone, PartialEq)]
pub enum FundraiserError {
    // The amount to raise has not been met
    TargetNotMet,
    // The amount to raise has been achieved
    TargetMet,
    // The contribution is too big
    ContributionTooBig,
    // The contribution is too small
    ContributionTooSmall,
    // The maximum amount to contribute has been reached
    MaximumContributionsReached,
    // The fundraiser has not ended yet
    FundraiserNotEnded,
    // The fundraiser has ended
    FundraiserEnded,
    // Invalid total amount. i should be bigger than 3
    InvalidAmount,
}

impl From<FundraiserError> for ProgramError {
    fn from(e: FundraiserError) -> Self {
        Self::Custom(e as u32)
    }
}
