use pinocchio::{ account_info::AccountInfo, program_error::ProgramError, ProgramResult };
use pinocchio_token::{
    instructions::{ CloseAccount, TransferChecked },
    state::Mint,
    state::TokenAccount,
};
use crate::{
    error::FundraiserError,
    state::Fundraiser,
    utils::{ load_acc_unchecked, Initialized },
};

pub fn process_check_contribution(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [maker, mint_to_raise, fundraiser, vault, maker_ata, _token_program, _system_program] =
        accounts else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let fundraiser_state = unsafe { load_acc_unchecked::<Fundraiser>(fundraiser)? };

    if !fundraiser_state.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }

    if fundraiser_state.current_amount <= fundraiser_state.amount_to_raise {
        return Err(FundraiserError::TargetNotMet.into());
    }

    // Transfer the funds to the maker
    let maker_ata_state = unsafe { load_acc_unchecked::<TokenAccount>(maker_ata)? };
    let mint_state = unsafe { load_acc_unchecked::<Mint>(mint_to_raise)? };

    let fundraiser_signer = Fundraiser::get_signer_seeds(maker.key(), fundraiser_state.bump);
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
        maker.borrow_mut_lamports_unchecked() += fundraiser.borrow_mut_lamports_unchecked();
        fundraiser.close();
    }

    Ok(())
}
