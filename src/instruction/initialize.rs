use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{ clock::Clock, rent::Rent, Sysvar },
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{ state::Fundraiser, utils::{ load_acc_mut_unchecked, load_ix_data, DataLen } };

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InitializeIxData {
    amount: u64,
    duration: u64,
    bump: u8,
}

impl DataLen for InitializeIxData {
    const LEN: usize = core::mem::size_of::<InitializeIxData>();
}

pub fn process_initialize(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        sysvar_rent_acc,
        _system_program,
        _token_program,
    ] = accounts else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !fundraiser.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Some more checks
    unsafe {
        // The vault should be intialised on client side to save CUs
        assert_eq!(vault.owner(), fundraiser.key());
    }

    let rent = Rent::from_account_info(sysvar_rent_acc)?;
    let ix_data = unsafe { load_ix_data::<InitializeIxData>(data)? };

    let pda_signer = Fundraiser::get_signer_seeds(maker.key(), ix_data.bump);

    // Create the fundraiser account
    (CreateAccount {
        from: maker,
        to: fundraiser,
        lamports: rent.minimum_balance(Fundraiser::LEN),
        space: Fundraiser::LEN as u64,
        owner: &crate::ID,
    }).invoke_signed(&[pda_signer[..]])?;

    let fundraiser_state = (unsafe {
        load_acc_mut_unchecked::<Fundraiser>(fundraiser.borrow_mut_data_unchecked())
    })?;

    fundraiser_state.initialize(
        maker.key(),
        mint_to_raise.key(),
        ix_data.amount,
        ix_data.duration,
        ix_data.bump,
        Clock::get()?.unix_timestamp
    );

    Ok(())
}
