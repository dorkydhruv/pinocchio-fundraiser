#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(feature = "std")]
extern crate std;
#[cfg(not(feature = "no-entrypoint"))]

mod entrypoint;
pub mod error;
pub mod instruction;
pub mod state;
pub mod utils;
pub mod constants;
pinocchio_pubkey::declare_id!("AvvaLMBjGBWNamh1qV72gzG412kiZWVFHu2PMi36Bg3G");
