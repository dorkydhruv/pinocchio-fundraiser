use pinocchio::{
    account_info::AccountInfo,
    instruction::{ Seed, Signer },
    program_error::ProgramError,
    sysvars::{ clock::Clock, Sysvar },
    ProgramResult,
};
use pinocchio_token::{ instructions::TransferChecked, state::{ Mint, TokenAccount } };

use crate::{
    constants::SECONDS_TO_DAYS,
    error::FundraiserError,
    state::{ Contributor, Fundraiser },
    utils::{ load_acc_mut_unchecked, Initialized },
};

pub fn process_refund(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [
        contributer,
        maker,
        mint_to_raise,
        fundraiser,
        contributor_acc,
        contributor_ata,
        vault,
        _system_program,
        _token_program,
        _rest @ ..,
    ] = accounts else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !contributer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Some checks for authorities
    unsafe {
        // The vault should be intialised on client side to save CUs
        assert_eq!(vault.owner(), fundraiser.key());
        assert_eq!(contributor_ata.owner(), contributer.key());
        assert_eq!(fundraiser.owner(), &crate::ID);
    }

    // Check if the fundraiser is initialized
    let fundraiser_state = unsafe {
        load_acc_mut_unchecked::<Fundraiser>(fundraiser.borrow_mut_data_unchecked())?
    };
    if !fundraiser_state.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }

    let contributor_state = unsafe {
        load_acc_mut_unchecked::<Contributor>(contributor_acc.borrow_mut_data_unchecked())?
    };
    if !contributor_state.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }

    // Check if the fundraising duration has been reached
    let current_time = Clock::get()?.unix_timestamp;
    if
        fundraiser_state.duration <
        (((current_time - fundraiser_state.time_started) / SECONDS_TO_DAYS) as u8)
    {
        return Err(FundraiserError::FundraiserNotEnded.into());
    }

    // Check if the target amount has been met
    let vault_state = unsafe {
        load_acc_mut_unchecked::<TokenAccount>(vault.borrow_mut_data_unchecked())?
    };

    if vault_state.amount() >= fundraiser_state.amount_to_raise {
        return Err(FundraiserError::TargetMet.into());
    }

    // Transfer the funds to the contributor
    let mint_state = unsafe {
        load_acc_mut_unchecked::<Mint>(mint_to_raise.borrow_mut_data_unchecked())?
    };

    let bump_seed = [fundraiser_state.bump];
    let fundraiser_seeds = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump_seed[..]),
    ];

    let fundraiser_signer = Signer::from(&fundraiser_seeds[..]);
    (TransferChecked {
        amount: contributor_state.amount,
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        mint: mint_to_raise,
        decimals: mint_state.decimals(),
    }).invoke_signed(&[fundraiser_signer.clone()])?;

    // Close the contributor account
    unsafe {
        *contributer.borrow_mut_lamports_unchecked() +=
            *contributor_acc.borrow_mut_lamports_unchecked();
        contributor_acc.close()?;
    }
    Ok(())
}
