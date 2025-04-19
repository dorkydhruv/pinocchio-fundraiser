# Pinocchio Fundraiser (WIP)

## Instructions tested..

Note: All ATAs are initialized on client/test

- Initialize - init fundraiser pda (2178 CUs) [Uses CreateAccount CPI]
- Contribute - contribute mint amount for fundraising campaign (6457 CUs) [Uses CreateAccount and TransferChecked CPIs]
- Checker - completes the fundraising campaign (7247 CUs) [Uses TransferChecked and CloseAccount CPIs]

## Get Started!

### 1. clone the repo

```bash
git clone https://github.com/dorkydhruv/pinocchio-fundraiser.git
```

### 2. Directory structure

- [src/](src/)

  - [entrypoint.rs](src/entrypoint.rs) - the entrypoint of the program

    - **Note:** it uses nostd_panic_handler to handle panics
      also global allocator is disabled meaning no heap allocations

  - [lib.rs](src/lib.rs) - lib crate

    - **Note:** uses no_std so we cannot use std library (for performance tweaks)

  - [instruction](src/instruction) - all instructions are defined here

  - [state](src/state/) - all account states are defined here

  - [utils.rs](src/utils.rs) - utils for state which provide serialization and deserialization helper fns( load_acc , load_mut_acc, etc)

  - [error.rs](program/src/error.rs) - program errors are listed here

- [tests](tests/) - all tests are defined here

  - **Note:** we are using mollusk-svm - a lightweight solana testing framework for running tests in a local environment without the need of a full solana cluster
  - [elfs](tests/elfs/) - compiled solana elfs can be added here and loaded to mollusk while testing
  - [unit_tests.rs](tests/unit_tests.rs) - has the unit tests for the program

- [benches](benches/) - all the benchmarks are defined here
  - [compute_units.md](benches/compute_units.md) - compute unit benchmarks

### 3. Build program

```bash
cargo build-sbf
```

- After build is successful get the program pubkey and replace with the pinocchio_pubkey::declare_id!(...)

```bash
solana address -k target/deploy/pinocchio-fundraiser-keypair.json
```

### 4. Running Tests

```bash
cargo test --features test-default
```

### 5. Running Benchmarks

```bash
cargo bench --features bench-default
```

#### Compute Unit Benchmarks

#### 2025-04-19 23:24:51.634550527 UTC

Solana CLI Version: solana-cli 2.1.17 (src:4adcd0f2; feat:3271415109, client:Agave)

| Name                             | CUs  | Delta   |
| -------------------------------- | ---- | ------- |
| Initialize                       | 2178 | --      |
| Contribute                       | 6457 | --      |
| Checker (after 10 contributions) | 7247 | - new - |
