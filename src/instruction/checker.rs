use pinocchio::{
    account_info::AccountInfo,
    instruction::{ Seed, Signer },
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_token::{
    instructions::{ CloseAccount, TransferChecked },
    state::Mint,
    state::TokenAccount,
};
use crate::{
    error::FundraiserError,
    state::Fundraiser,
    utils::{ load_acc_unchecked, DataLen, Initialized },
};

impl DataLen for Mint {
    const LEN: usize = core::mem::size_of::<Mint>();
}

impl DataLen for TokenAccount {
    const LEN: usize = core::mem::size_of::<TokenAccount>();
}

pub fn process_check_contribution(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [maker, mint_to_raise, fundraiser, vault, maker_ata, _token_program, _system_program] =
        accounts else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let fundraiser_state = unsafe {
        load_acc_unchecked::<Fundraiser>(fundraiser.borrow_data_unchecked())?
    };

    if !fundraiser_state.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }

    if fundraiser_state.current_amount <= fundraiser_state.amount_to_raise {
        return Err(FundraiserError::TargetNotMet.into());
    }

    // Transfer the funds to the maker
    let _maker_ata_state = unsafe {
        load_acc_unchecked::<TokenAccount>(maker_ata.borrow_data_unchecked())?
    };
    let mint_state = unsafe { load_acc_unchecked::<Mint>(mint_to_raise.borrow_data_unchecked())? };

    let bump_seed = [fundraiser_state.bump];
    let fundraiser_seeds = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump_seed[..]),
    ];

    let fundraiser_signer = Signer::from(&fundraiser_seeds[..]);
    (TransferChecked {
        amount: fundraiser_state.current_amount,
        from: vault,
        to: maker_ata,
        authority: fundraiser,
        mint: mint_to_raise,
        decimals: mint_state.decimals(),
    }).invoke_signed(&[fundraiser_signer.clone()])?;

    // Close the vault account
    (CloseAccount {
        account: vault,
        destination: maker,
        authority: fundraiser,
    }).invoke_signed(&[fundraiser_signer.clone()])?;

    // Close the fundraiser account
    unsafe {
        *maker.borrow_mut_lamports_unchecked() += *fundraiser.borrow_lamports_unchecked();
        fundraiser.close()?;
    }

    Ok(())
}
