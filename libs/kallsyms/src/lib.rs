#![cfg_attr(all(feature = "no_std", not(test)), no_std)]

mod kallsyms;
mod types;

pub use crate::kallsyms::KAllSyms;
pub type Address = types::AddrTblEntry;

#[cfg(not(all(feature = "no_std", not(test))))]
mod compress;
#[cfg(not(all(feature = "no_std", not(test))))]
mod pack;
#[cfg(not(all(feature = "no_std", not(test))))]
pub use pack::pack;
