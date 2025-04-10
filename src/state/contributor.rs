use crate::utils::{ DataLen, Initialized };

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contributor {
    pub is_initialized: bool,
    pub amount: u64,
}

impl DataLen for Contributor {
    const LEN: usize = core::mem::size_of::<Contributor>();
}

impl Initialized for Contributor {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Contributor {
    pub const SEED: &'static str = "contributor";

    pub fn initialize(&mut self, amount: u64) {
        self.is_initialized = true;
        self.amount = amount;
    }
}
