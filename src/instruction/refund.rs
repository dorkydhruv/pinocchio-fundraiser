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
    utils::load_acc_mut,
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

    // / Some checks for authorities
    let vault_acc = TokenAccount::from_account_info(vault)?;
    // The vault should be intialised on client side to save CUs
    assert_eq!(vault_acc.owner(), fundraiser.key());
    // assert_eq!(contributor_ata_acc.owner(), contributor.key());
    // Some checks for authorities
    // Check if the fundraiser is initialized
    let fundraiser_state = unsafe {
        load_acc_mut::<Fundraiser>(fundraiser.borrow_mut_data_unchecked())?
    };

    let contributor_state = unsafe {
        load_acc_mut::<Contributor>(contributor_acc.borrow_mut_data_unchecked())?
    };

    // Check if the fundraising duration has been reached
    let current_time = Clock::get()?.unix_timestamp;
    if
        fundraiser_state.duration >
        (((current_time - fundraiser_state.time_started) / SECONDS_TO_DAYS) as u8)
    {
        return Err(FundraiserError::FundraiserNotEnded.into());
    }
    if vault_acc.amount() >= fundraiser_state.amount_to_raise {
        return Err(FundraiserError::TargetMet.into());
    }

    // Transfer the funds to the contributor
    let mint_state = Mint::from_account_info(mint_to_raise)?;
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
    }
    contributor_acc.close()?;
    Ok(())
}
