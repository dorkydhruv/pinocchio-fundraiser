use pinocchio::pubkey::Pubkey;

use crate::utils::{ DataLen, Initialized };

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fundraiser {
    is_initialized: bool,
    pub maker: Pubkey,
    pub mint_to_raise: Pubkey,
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: u8,
    pub bump: u8,
}

impl DataLen for Fundraiser {
    const LEN: usize = core::mem::size_of::<Fundraiser>();
}

impl Initialized for Fundraiser {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Fundraiser {
    pub const SEED: &'static str = "fundraiser";

    pub fn initialize(
        &mut self,
        maker: Pubkey,
        mint_to_raise: Pubkey,
        amount_to_raise: u64,
        duration: u8,
        bump: u8,
        time_started: i64
    ) {
        self.is_initialized = true;
        self.maker = maker;
        self.mint_to_raise = mint_to_raise;
        self.amount_to_raise = amount_to_raise;
        self.current_amount = 0;
        self.time_started = time_started;
        self.duration = duration;
        self.bump = bump;
    }
}
