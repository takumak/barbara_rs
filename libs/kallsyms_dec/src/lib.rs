#![cfg_attr(not(test), no_std)]

mod kallsyms;
mod types;

pub use crate::kallsyms::KAllSyms;
pub use types::AddrTblEntry;
pub use types::Header;
pub use types::StrTblOff;
pub type KAddress = types::AddrTblEntry;
