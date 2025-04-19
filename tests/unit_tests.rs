use mollusk_svm::result::{ Check, ProgramResult };
use mollusk_svm::{ program, Mollusk };
use solana_sdk::account::{ Account, WritableAccount };
use solana_sdk::instruction::{ AccountMeta, Instruction };
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::program_option::COption;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
extern crate alloc;
use alloc::vec;

use pinocchio_fundraiser::instruction::{ InitializeIxData, ContributeIxData };
use pinocchio_fundraiser::state::{ Contributor, Fundraiser };
use pinocchio_fundraiser::utils::to_bytes;
use solana_sdk::rent::Rent;
use solana_sdk::sysvar::Sysvar;

pub const PROGRAM: Pubkey = Pubkey::new_from_array(pinocchio_fundraiser::ID);
pub const RENT: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");

// 1 -> 0x01 -> Maker
// 2 -> 0x02 -> Contributor
// 3 -> 0x03 -> Mint to raise
// 4 -> 0x04 -> Vault ATA
// 5 -> 0x05 -> Contributor ATA

pub fn mollusk() -> Mollusk {
    let mut mollusk = Mollusk::new(&PROGRAM, "target/deploy/pinocchio_fundraiser");
    mollusk.add_program(
        &spl_token::ID,
        "tests/elfs/spl_token",
        &mollusk_svm::program::loader_keys::LOADER_V3
    );
    mollusk
}

pub trait AccountExt {
    fn refresh(
        &mut self,
        account_pubkey: &Pubkey,
        result: mollusk_svm::result::InstructionResult
    ) -> &mut Self;
}

impl AccountExt for Account {
    fn refresh(
        &mut self,
        account_pubkey: &Pubkey,
        result: mollusk_svm::result::InstructionResult
    ) -> &mut Self {
        *self = result.get_account(account_pubkey).unwrap().clone();
        self
    }
}

pub fn get_spl_token_program() -> (Pubkey, Account) {
    (spl_token::ID, program::create_program_account_loader_v3(&spl_token::ID))
}

pub fn get_rent_data() -> Vec<u8> {
    let rent = Rent::default();
    unsafe {
        core::slice::from_raw_parts(&rent as *const Rent as *const u8, Rent::size_of()).to_vec()
    }
}

/// Setup common fundraiser accounts
pub fn setup_fundraiser(mollusk: &Mollusk) -> (
    // Pubkeys
    Pubkey,
    Pubkey,
    Pubkey,
    u8,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    // Accounts
    Account,
    Account,
    Account,
    Account,
    Account,
    Account,
    Account,
) {
    // Setup system and token programs
    let (system_program, system_account) = program::keyed_account_for_system_program();
    let (token_program, token_account) = get_spl_token_program();

    // Setup maker and contributor accounts
    let maker = Pubkey::new_from_array([0x01; 32]);
    let maker_account = Account::new(2 * LAMPORTS_PER_SOL, 0, &system_program);
    let contributor = Pubkey::new_unique();
    let contributor_account = Account::new(2 * LAMPORTS_PER_SOL, 0, &system_program);

    // Derive PDAs
    let (fundraiser, fundraiser_bump) = Pubkey::find_program_address(
        &[Fundraiser::SEED.as_bytes(), &maker.to_bytes()],
        &PROGRAM
    );

    // Create empty fundraiser account (will be initialized later)
    let fundraiser_account = Account::new(0, 0, &system_program);

    // Create mint account
    let mint_to_raise = Pubkey::new_from_array([0x03; 32]);
    let mut mint_to_raise_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program
    );
    solana_sdk::program_pack::Pack
        ::pack(
            spl_token::state::Mint {
                decimals: 6,
                supply: 100_000,
                is_initialized: true,
                freeze_authority: COption::None,
                mint_authority: COption::None,
            },
            mint_to_raise_account.data_as_mut_slice()
        )
        .unwrap();

    // Create vault account
    let vault = Pubkey::new_from_array([0x04; 32]);
    let mut vault_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program
    );
    solana_sdk::program_pack::Pack
        ::pack(
            spl_token::state::Account {
                amount: 0,
                mint: mint_to_raise,
                owner: fundraiser,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                close_authority: COption::None,
                is_native: COption::None,
                delegated_amount: 0,
            },
            vault_account.data_as_mut_slice()
        )
        .unwrap();

    (
        // Return Pubkeys
        maker,
        contributor,
        fundraiser,
        fundraiser_bump,
        mint_to_raise,
        vault,
        system_program,
        token_program,
        // Return Accounts
        maker_account,
        contributor_account,
        fundraiser_account,
        mint_to_raise_account,
        vault_account,
        system_account,
        token_account,
    )
}

/// Execute the initialize fundraiser instruction
pub fn execute_initialize(
    mollusk: &Mollusk,
    maker: Pubkey,
    maker_account: Account,
    mint_to_raise: Pubkey,
    mint_to_raise_account: Account,
    fundraiser: Pubkey,
    fundraiser_account: Account,
    vault: Pubkey,
    vault_account: Account,
    fundraiser_bump: u8,
    system_program: Pubkey,
    system_account: Account,
    token_program: Pubkey,
    token_account: Account,
    amount: u64,
    duration: u8
) -> mollusk_svm::result::InstructionResult {
    // Create instruction accounts
    let ix_accounts = vec![
        AccountMeta::new(maker, true),
        AccountMeta::new(mint_to_raise, false),
        AccountMeta::new(fundraiser, true),
        AccountMeta::new(vault, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(token_program, false)
    ];

    // Create instruction data
    let ix_data = InitializeIxData {
        amount,
        duration,
        bump: fundraiser_bump,
    };

    // Serialize instruction with discriminator
    let mut ser_ix_data = vec![0]; // Ix discriminator = 0
    ser_ix_data.extend_from_slice(unsafe { to_bytes(&ix_data) });

    // Create instruction
    let instruction = Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts);

    // Create transaction accounts
    let tx_accounts = &vec![
        (maker, maker_account),
        (mint_to_raise, mint_to_raise_account),
        (fundraiser, fundraiser_account),
        (vault, vault_account),
        (system_program, system_account),
        (token_program, token_account)
    ];

    // Process instruction
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        tx_accounts,
        &[Check::success()]
    );
    result
}

/// Get new contributor account
pub fn get_new_contributor_account(_mollusk: &Mollusk, system_program: Pubkey) -> (Pubkey, Account) {
    let contributor = Pubkey::new_unique();
    let contributor_account = Account::new(2 * LAMPORTS_PER_SOL, 0, &system_program);
    return (contributor, contributor_account);
}

/// Setup contributor accounts
pub fn setup_contributor(
    mollusk: &Mollusk,
    contributor: Pubkey,
    fundraiser: Pubkey,
    mint_to_raise: Pubkey,
    system_program: Pubkey,
    token_program: Pubkey
) -> (Pubkey, u8, Pubkey, Account, Account) {
    // Derive contributor PDA
    let (contributor_acc, contributor_bump) = Pubkey::find_program_address(
        &[Contributor::SEED.as_bytes(), &fundraiser.to_bytes(), &contributor.to_bytes()],
        &PROGRAM
    );

    // Create empty contributor account (will be initialized during contribute)
    let contributor_acc_account = Account::new(0, 0, &system_program);

    // Create contributor ATA
    let contributor_ata = Pubkey::new_unique();
    let mut contributor_ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program
    );
    solana_sdk::program_pack::Pack
        ::pack(
            spl_token::state::Account {
                amount: 1_000_000,
                mint: mint_to_raise,
                owner: contributor,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                close_authority: COption::None,
                is_native: COption::None,
                delegated_amount: 0,
            },
            contributor_ata_account.data_as_mut_slice()
        )
        .unwrap();

    (
        contributor_acc,
        contributor_bump,
        contributor_ata,
        contributor_acc_account,
        contributor_ata_account,
    )
}

/// Execute the contribute instruction
pub fn execute_contribute(
    mollusk: &Mollusk,
    contributor: Pubkey,
    contributor_account: Account,
    mint_to_raise: Pubkey,
    mint_to_raise_account: Account,
    fundraiser: Pubkey,
    fundraiser_account: Account,
    contributor_acc: Pubkey,
    contributor_acc_account: Account,
    contributor_ata: Pubkey,
    contributor_ata_account: Account,
    vault: Pubkey,
    vault_account: Account,
    contributor_bump: u8,
    fundraiser_bump: u8,
    system_program: Pubkey,
    system_account: Account,
    token_program: Pubkey,
    token_account: Account,
    amount: u64
) -> mollusk_svm::result::InstructionResult {
    // Create instruction accounts
    let ix_accounts = vec![
        AccountMeta::new(contributor, true),
        AccountMeta::new(mint_to_raise, false),
        AccountMeta::new(fundraiser, false),
        AccountMeta::new(contributor_acc, true),
        AccountMeta::new(contributor_ata, false),
        AccountMeta::new(vault, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(token_program, false)
    ];

    // Create instruction data
    let ix_data = ContributeIxData {
        amount,
        contributor_bump,
        fundraiser_bump,
    };

    // Serialize instruction with discriminator
    let mut ser_ix_data = vec![1]; // Ix discriminator = 1
    ser_ix_data.extend_from_slice(unsafe { to_bytes(&ix_data) });

    // Create instruction
    let instruction = Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts);

    // Create transaction accounts
    let tx_accounts = &vec![
        (contributor, contributor_account),
        (mint_to_raise, mint_to_raise_account),
        (fundraiser, fundraiser_account),
        (contributor_acc, contributor_acc_account),
        (contributor_ata, contributor_ata_account),
        (vault, vault_account),
        (system_program, system_account),
        (token_program, token_account)
    ];

    // Process instruction
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        tx_accounts,
        &[Check::success()]
    );

    result
}

pub fn execute_checker(
    mollusk: &Mollusk,
    maker: Pubkey,
    maker_account: Account,
    mint_to_raise: Pubkey,
    mint_to_raise_account: Account,
    fundraiser: Pubkey,
    fundraiser_account: Account,
    vault: Pubkey,
    vault_account: Account,
    system_program: Pubkey,
    system_account: Account,
    token_program: Pubkey,
    token_account: Account
) -> mollusk_svm::result::InstructionResult {
    // Create maker ATA
    let maker_ata = Pubkey::new_unique();
    let mut maker_ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program
    );
    solana_sdk::program_pack::Pack
        ::pack(
            spl_token::state::Account {
                amount: 0,
                mint: mint_to_raise,
                owner: maker,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                close_authority: COption::None,
                is_native: COption::None,
                delegated_amount: 0,
            },
            maker_ata_account.data_as_mut_slice()
        )
        .unwrap();

    // Create instruction accounts
    let ix_accounts = vec![
        AccountMeta::new(maker, true),
        AccountMeta::new(mint_to_raise, false),
        AccountMeta::new(fundraiser, true),
        AccountMeta::new(vault, false),
        AccountMeta::new(maker_ata, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(token_program, false)
    ];

    // Create instruction data
    let ix_data = InitializeIxData {
        amount: 0,
        duration: 0,
        bump: 0,
    };

    // Serialize instruction with discriminator
    let mut ser_ix_data = vec![2]; // Ix discriminator = 2
    ser_ix_data.extend_from_slice(unsafe { to_bytes(&ix_data) });

    // Create instruction
    let instruction = Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts);

    // Create transaction accounts
    let tx_accounts = &vec![
        (maker, maker_account),
        (mint_to_raise, mint_to_raise_account),
        (fundraiser, fundraiser_account),
        (vault, vault_account),
        (maker_ata, maker_ata_account),
        (system_program, system_account),
        (token_program, token_account)
    ];

    // Process instruction
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        tx_accounts,
        &[Check::success()]
    );

    result
}

#[test]
fn test_initialize_fundraiser() {
    let mollusk = mollusk();
    // get states
    let (
        maker,
        _contributor,
        fundraiser,
        fundraiser_bump,
        mint_to_raise,
        vault,
        system_program,
        token_program,
        mut maker_account,
         _contributor_account,
        mut fundraiser_account,
        mut mint_to_raise_account,
        mut vault_account,
        system_account,
        token_account,
    ) = setup_fundraiser(&mollusk);
    // execute fundraiser instruction
    let init_result = execute_initialize(
        &mollusk,
        maker.clone(),
        maker_account.clone(),
        mint_to_raise.clone(),
        mint_to_raise_account.clone(),
        fundraiser.clone(),
        fundraiser_account.clone(),
        vault.clone(),
        vault_account.clone(),
        fundraiser_bump.clone(),
        system_program.clone(),
        system_account.clone(),
        token_program.clone(),
        token_account.clone(),
        100_000_000, // amount to raise is 100 tokens
        1 // duration 1 day
    );
    maker_account.refresh(&maker, init_result.clone());
    mint_to_raise_account.refresh(&mint_to_raise, init_result.clone());
    vault_account.refresh(&vault, init_result.clone());
    fundraiser_account.refresh(&fundraiser, init_result.clone());
    // check result
    assert_eq!(init_result.program_result, ProgramResult::Success);
}

#[test]
fn test_contribute() {
    let mollusk = mollusk();
    // get states
    let (
        maker,
        contributor,
        fundraiser,
        fundraiser_bump,
        mint_to_raise,
        vault,
        system_program,
        token_program,
        mut maker_account,
         contributor_account,
        mut fundraiser_account,
        mut mint_to_raise_account,
        mut vault_account,
        system_account,
        token_account,
    ) = setup_fundraiser(&mollusk);
    // execute fundraiser instruction
    let init_result = execute_initialize(
        &mollusk,
        maker.clone(),
        maker_account.clone(),
        mint_to_raise.clone(),
        mint_to_raise_account.clone(),
        fundraiser.clone(),
        fundraiser_account.clone(),
        vault.clone(),
        vault_account.clone(),
        fundraiser_bump.clone(),
        system_program.clone(),
        system_account.clone(),
        token_program.clone(),
        token_account.clone(),
        10_000_000, // amount to raise is 10 tokens
        1 // duration 1 day
    );
    maker_account.refresh(&maker, init_result.clone());
    mint_to_raise_account.refresh(&mint_to_raise, init_result.clone());
    vault_account.refresh(&vault, init_result.clone());
    fundraiser_account.refresh(&fundraiser, init_result.clone());
    // check result
    assert_eq!(init_result.program_result, ProgramResult::Success);

    // setup contributor
    let (
        contributor_acc,
        contributor_bump,
        contributor_ata,
        mut contributor_acc_account,
        mut contributor_ata_account,
    ) = setup_contributor(
        &mollusk,
        contributor.clone(),
        fundraiser.clone(),
        mint_to_raise.clone(),
        system_program.clone(),
        token_program.clone()
    );

    // execute contribute instruction
    let contribute_result = execute_contribute(
        &mollusk,
        contributor.clone(),
        contributor_account.clone(),
        mint_to_raise.clone(),
        mint_to_raise_account.clone(),
        fundraiser.clone(),
        fundraiser_account.clone(),
        contributor_acc.clone(),
        contributor_acc_account.clone(),
        contributor_ata.clone(),
        contributor_ata_account.clone(),
        vault.clone(),
        vault_account.clone().clone(),
        contributor_bump.clone(),
        fundraiser_bump.clone(),
        system_program.clone(),
        system_account.clone(),
        token_program.clone(),
        token_account.clone(),

        1_000_000 // amount to contribute is 1 token
    );
    contributor_acc_account.refresh(&contributor, contribute_result.clone());
    mint_to_raise_account.refresh(&mint_to_raise, contribute_result.clone());
    vault_account.refresh(&vault, contribute_result.clone());
    contributor_ata_account.refresh(&contributor_ata, contribute_result.clone());
    fundraiser_account.refresh(&fundraiser, contribute_result.clone());

    assert_eq!(contribute_result.program_result, ProgramResult::Success);
}

#[test]
fn test_checker() {
    let mollusk = mollusk();
    // get states
    let (
        maker,
        contributor,
        fundraiser,
        fundraiser_bump,
        mint_to_raise,
        vault,
        system_program,
        token_program,
        mut maker_account,
        mut contributor_account,
        mut fundraiser_account,
        mut mint_to_raise_account,
        mut vault_account,
        system_account,
        token_account,
    ) = setup_fundraiser(&mollusk);
    // execute fundraiser instruction
    let init_result = execute_initialize(
        &mollusk,
        maker.clone(),
        maker_account.clone(),
        mint_to_raise.clone(),
        mint_to_raise_account.clone(),
        fundraiser.clone(),
        fundraiser_account.clone(),
        vault.clone(),
        vault_account.clone(),
        fundraiser_bump.clone(),
        system_program.clone(),
        system_account.clone(),
        token_program.clone(),
        token_account.clone(),
        10_000_000, // amount to raise is 10 tokens
        1 // duration 1 day
    );
    maker_account.refresh(&maker, init_result.clone());
    mint_to_raise_account.refresh(&mint_to_raise, init_result.clone());
    vault_account.refresh(&vault, init_result.clone());
    fundraiser_account.refresh(&fundraiser, init_result.clone());
    // check result
    assert_eq!(init_result.program_result, ProgramResult::Success);

    // setup the original contributor
    let (
        contributor_acc,
        contributor_bump,
        contributor_ata,
        mut contributor_acc_account,
        mut contributor_ata_account,
    ) = setup_contributor(
        &mollusk,
        contributor.clone(),
        fundraiser.clone(),
        mint_to_raise.clone(),
        system_program.clone(),
        token_program.clone()
    );

    // execute contribute instruction
    let contribute_result = execute_contribute(
        &mollusk,
        contributor.clone(),
        contributor_account.clone(),
        mint_to_raise.clone(),
        mint_to_raise_account.clone(),
        fundraiser.clone(),
        fundraiser_account.clone(),
        contributor_acc.clone(),
        contributor_acc_account.clone(),
        contributor_ata.clone(),
        contributor_ata_account.clone(),
        vault.clone(),
        vault_account.clone().clone(),
        contributor_bump.clone(),
        fundraiser_bump.clone(),
        system_program.clone(),
        system_account.clone(),
        token_program.clone(),
        token_account.clone(),

        1_000_000 // amount to contribute is 1 token
    );
    contributor_account.refresh(&contributor, contribute_result.clone());
    contributor_acc_account.refresh(&contributor_acc, contribute_result.clone());
    mint_to_raise_account.refresh(&mint_to_raise, contribute_result.clone());
    vault_account.refresh(&vault, contribute_result.clone());
    contributor_ata_account.refresh(&contributor_ata, contribute_result.clone());
    fundraiser_account.refresh(&fundraiser, contribute_result.clone());
    assert_eq!(contribute_result.program_result, ProgramResult::Success);

    // lets do this 9 more times
    for _ in 0..9 {
        let (new_contributor, mut new_contributor_account) = get_new_contributor_account(
            &mollusk,
            system_program.clone()
        );
        let (
            new_contributor_acc,
            new_contributor_bump,
            new_contributor_ata,
            mut new_contributor_acc_account,
            mut new_contributor_ata_account,
        ) = setup_contributor(
            &mollusk,
            new_contributor.clone(),
            fundraiser.clone(),
            mint_to_raise.clone(),
            system_program.clone(),
            token_program.clone()
        );
        let contribute_result = execute_contribute(
            &mollusk,
            new_contributor.clone(),
            new_contributor_account.clone(),
            mint_to_raise.clone(),
            mint_to_raise_account.clone(),
            fundraiser.clone(),
            fundraiser_account.clone(),
            new_contributor_acc.clone(),
            new_contributor_acc_account.clone(),
            new_contributor_ata.clone(),
            new_contributor_ata_account.clone(),
            vault.clone(),
            vault_account.clone().clone(),
            new_contributor_bump.clone(),
            fundraiser_bump.clone(),
            system_program.clone(),
            system_account.clone(),
            token_program.clone(),
            token_account.clone(),

            1_000_000 // amount to contribute is 1 token
        );
        new_contributor_account.refresh(&new_contributor, contribute_result.clone());
        new_contributor_acc_account.refresh(&new_contributor_acc, contribute_result.clone());
        mint_to_raise_account.refresh(&mint_to_raise, contribute_result.clone());
        vault_account.refresh(&vault, contribute_result.clone());
        new_contributor_ata_account.refresh(&new_contributor_ata, contribute_result.clone());
        fundraiser_account.refresh(&fundraiser, contribute_result.clone());
        assert_eq!(contribute_result.program_result, ProgramResult::Success);
    }

    // lets now check the fundraiser account
    let checker_result = execute_checker(
        &mollusk,
        maker.clone(),
        maker_account.clone(),
        mint_to_raise.clone(),
        mint_to_raise_account.clone(),
        fundraiser.clone(),
        fundraiser_account.clone(),
        vault.clone(),
        vault_account.clone(),
        system_program.clone(),
        system_account.clone(),
        token_program.clone(),
        token_account.clone()
    );
    maker_account.refresh(&maker, checker_result.clone());
    mint_to_raise_account.refresh(&mint_to_raise, checker_result.clone());
    vault_account.refresh(&vault, checker_result.clone());
    fundraiser_account.refresh(&fundraiser, checker_result.clone());
    assert_eq!(checker_result.program_result, ProgramResult::Success);
}
