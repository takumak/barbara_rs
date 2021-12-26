extern crate kallsyms_dec;
use kallsyms_dec::{KAllSyms, KAddress};

use crate::decl_c_symbol_addr;
decl_c_symbol_addr!(__kallsyms, kallsyms_addr);

pub fn safe_search<'a>(addr: usize, buf: &'a mut [u8]) -> Option<(&'a str, usize)> {
    let kallsyms = KAllSyms::new(kallsyms_addr());
    match kallsyms.safe_search(addr as KAddress, buf) {
        Some((name, off)) => Some((name, off as usize)),
        None => None,
    }
}
