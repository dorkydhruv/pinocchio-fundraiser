use mollusk_svm::result::Check;
use mollusk_svm::{ program, Mollusk };
use mollusk_svm_bencher::MolluskComputeUnitBencher;
use solana_sdk::account::{ Account, WritableAccount };
use solana_sdk::instruction::{ AccountMeta, Instruction };
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::program_option::COption;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
extern crate alloc;
use alloc::vec;

use pinocchio_fundraiser::instruction::{ InitializeIxData, ContributeIxData };
use pinocchio_fundraiser::state::{ Contributor, Fundraiser };
use pinocchio_fundraiser::utils::to_bytes;

pub const PROGRAM: Pubkey = Pubkey::new_from_array(pinocchio_fundraiser::ID);

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

fn main() {
    let mut mollusk = Mollusk::new(&PROGRAM, "target/deploy/pinocchio_fundraiser");
    mollusk.add_program(
        &spl_token::ID,
        "tests/elfs/spl_token",
        &mollusk_svm::program::loader_keys::LOADER_V3
    );

    // Setup all accounts first
    let (system_program, _system_account) = program::keyed_account_for_system_program();
    let (token_program, _token_account) = get_spl_token_program();

    // Setup common accounts using the proven test code
    let (
        maker,
        _contributor, // We'll use a new one
        fundraiser,
        fundraiser_bump,
        mint_to_raise,
        vault,
        _system_program, // Already defined
        _token_program, // Already defined
        mut maker_account,
        _contributor_account, // We'll use a new one
        mut fundraiser_account,
        mut mint_to_raise_account,
        mut vault_account,
        mut system_account,
        mut token_account,
    ) = setup_fundraiser(&mollusk);

    // 1. Initialize instruction
    let init_instruction = create_initialize_instruction(
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        fundraiser_bump,
        system_program,
        token_program,
        10_000_000, // Amount to raise (10 tokens)
        1 // Duration (1 day)
    );
    let init_accounts = vec![
        (maker, maker_account.clone()),
        (mint_to_raise, mint_to_raise_account.clone()),
        (fundraiser, fundraiser_account.clone()),
        (vault, vault_account.clone()),
        (system_program, system_account.clone()),
        (token_program, token_account.clone())
    ];

    // Execute initialize (need to apply changes before contribute)
    let init_result = mollusk.process_and_validate_instruction(
        &init_instruction,
        &init_accounts,
        &[Check::success()]
    );

    // Update account states after initialization
    maker_account.refresh(&maker, init_result.clone());
    fundraiser_account.refresh(&fundraiser, init_result.clone());
    mint_to_raise_account.refresh(&mint_to_raise, init_result.clone());
    vault_account.refresh(&vault, init_result.clone());
    system_account.refresh(&system_program, init_result.clone());
    token_account.refresh(&token_program, init_result.clone());

    // 2. First contributor setup and contribution
    let contributor = Pubkey::new_unique();
    let mut contributor_account = Account::new(2 * LAMPORTS_PER_SOL, 0, &system_program);

    let (
        contributor_acc,
        contributor_bump,
        contributor_ata,
        mut contributor_acc_account,
        mut contributor_ata_account,
    ) = setup_contributor(
        &mollusk,
        contributor,
        fundraiser,
        mint_to_raise,
        system_program,
        token_program
    );

    let contribute_instruction = create_contribute_instruction(
        contributor,
        mint_to_raise,
        fundraiser,
        contributor_acc,
        contributor_ata,
        vault,
        contributor_bump,
        fundraiser_bump,
        system_program,
        token_program,
        1_000_000 // 1 token
    );
    let contribute_accounts = vec![
        (contributor, contributor_account.clone()),
        (mint_to_raise, mint_to_raise_account.clone()),
        (fundraiser, fundraiser_account.clone()),
        (contributor_acc, contributor_acc_account.clone()),
        (contributor_ata, contributor_ata_account.clone()),
        (vault, vault_account.clone()),
        (system_program, system_account.clone()),
        (token_program, token_account.clone())
    ];

    // Execute first contribution
    let contribute_result = mollusk.process_and_validate_instruction(
        &contribute_instruction,
        &contribute_accounts,
        &[Check::success()]
    );

    // Update account states after contribution
    contributor_account.refresh(&contributor, contribute_result.clone());
    contributor_acc_account.refresh(&contributor_acc, contribute_result.clone());
    mint_to_raise_account.refresh(&mint_to_raise, contribute_result.clone());
    fundraiser_account.refresh(&fundraiser, contribute_result.clone());
    vault_account.refresh(&vault, contribute_result.clone());
    contributor_ata_account.refresh(&contributor_ata, contribute_result.clone());
    system_account.refresh(&system_program, contribute_result.clone());
    token_account.refresh(&token_program, contribute_result.clone());

    // 3. Make 9 more contributions from different contributors (like in test_checker)
    for _ in 0..9 {
        let (new_contributor, mut new_contributor_account) = get_new_contributor_account(
            &mollusk,
            system_program
        );

        let (
            new_contributor_acc,
            new_contributor_bump,
            new_contributor_ata,
            mut new_contributor_acc_account,
            mut new_contributor_ata_account,
        ) = setup_contributor(
            &mollusk,
            new_contributor,
            fundraiser,
            mint_to_raise,
            system_program,
            token_program
        );

        let contribute_instruction = create_contribute_instruction(
            new_contributor,
            mint_to_raise,
            fundraiser,
            new_contributor_acc,
            new_contributor_ata,
            vault,
            new_contributor_bump,
            fundraiser_bump,
            system_program,
            token_program,
            1_000_000 // 1 token
        );

        let contribute_accounts = vec![
            (new_contributor, new_contributor_account.clone()),
            (mint_to_raise, mint_to_raise_account.clone()),
            (fundraiser, fundraiser_account.clone()),
            (new_contributor_acc, new_contributor_acc_account.clone()),
            (new_contributor_ata, new_contributor_ata_account.clone()),
            (vault, vault_account.clone()),
            (system_program, system_account.clone()),
            (token_program, token_account.clone())
        ];

        // Execute additional contribution
        let contribute_result = mollusk.process_and_validate_instruction(
            &contribute_instruction,
            &contribute_accounts,
            &[Check::success()]
        );

        // Update account states
        new_contributor_account.refresh(&new_contributor, contribute_result.clone());
        new_contributor_acc_account.refresh(&new_contributor_acc, contribute_result.clone());
        mint_to_raise_account.refresh(&mint_to_raise, contribute_result.clone());
        fundraiser_account.refresh(&fundraiser, contribute_result.clone());
        vault_account.refresh(&vault, contribute_result.clone());
        new_contributor_ata_account.refresh(&new_contributor_ata, contribute_result.clone());
        system_account.refresh(&system_program, contribute_result.clone());
        token_account.refresh(&token_program, contribute_result.clone());
    }

    // 4. Setup maker ATA for checker
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

    // 5. Checker instruction (after 10 contributions)
    let checker_instruction = create_checker_instruction(
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        maker_ata,
        system_program,
        token_program
    );
    let checker_accounts = vec![
        (maker, maker_account.clone()),
        (mint_to_raise, mint_to_raise_account.clone()),
        (fundraiser, fundraiser_account.clone()),
        (vault, vault_account.clone()),
        (maker_ata, maker_ata_account.clone()),
        (system_program, system_account.clone()),
        (token_program, token_account.clone())
    ];

    // Run the benchmarks for each instruction independently
    // Since we've already executed the instructions to prepare the state,
    // we'll benchmark them individually to see their compute units
    MolluskComputeUnitBencher::new(mollusk)
        .bench(("Initialize", &init_instruction, &init_accounts))
        .bench(("Contribute", &contribute_instruction, &contribute_accounts))
        .bench(("Checker (after 10 contributions)", &checker_instruction, &checker_accounts))
        .must_pass(true)
        .out_dir("benches/")
        .execute();
}

fn create_initialize_instruction(
    maker: Pubkey,
    mint_to_raise: Pubkey,
    fundraiser: Pubkey,
    vault: Pubkey,
    fundraiser_bump: u8,
    system_program: Pubkey,
    token_program: Pubkey,
    amount: u64,
    duration: u8
) -> Instruction {
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
    Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts)
}

fn create_contribute_instruction(
    contributor: Pubkey,
    mint_to_raise: Pubkey,
    fundraiser: Pubkey,
    contributor_acc: Pubkey,
    contributor_ata: Pubkey,
    vault: Pubkey,
    contributor_bump: u8,
    fundraiser_bump: u8,
    system_program: Pubkey,
    token_program: Pubkey,
    amount: u64
) -> Instruction {
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
    Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts)
}

fn create_checker_instruction(
    maker: Pubkey,
    mint_to_raise: Pubkey,
    fundraiser: Pubkey,
    vault: Pubkey,
    maker_ata: Pubkey,
    system_program: Pubkey,
    token_program: Pubkey
) -> Instruction {
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

    // Serialize instruction with discriminator
    let mut ser_ix_data = vec![2]; // Ix discriminator = 2
    ser_ix_data.extend_from_slice(unsafe { to_bytes(
            &(InitializeIxData {
                amount: 0,
                duration: 0,
                bump: 0,
            })
        ) });

    // Create instruction
    Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts)
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

pub fn get_spl_token_program() -> (Pubkey, Account) {
    (spl_token::ID, program::create_program_account_loader_v3(&spl_token::ID))
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
                amount: 10_000_000, // 10 tokens available to contribute
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
