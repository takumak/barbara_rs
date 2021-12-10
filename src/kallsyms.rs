extern crate kallsyms;

use crate::decl_c_symbol_addr;
decl_c_symbol_addr!(__kallsyms, kallsyms_addr);

pub fn safe_search<'a>(addr: usize, buf: &'a mut [u8]) -> Option<(&'a str, usize)> {
    let kallsyms = kallsyms::KAllSyms::new(kallsyms_addr());
    kallsyms.safe_search(addr, buf)
}
