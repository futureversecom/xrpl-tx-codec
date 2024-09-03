#![cfg_attr(not(test), no_std)]

extern crate alloc;
#[cfg(not(test))]
pub use alloc::vec::Vec;
#[cfg(test)]
pub use std::vec::Vec;

mod error;
pub mod field;
pub mod traits;
pub mod transaction;
pub mod types;
pub mod utils;
