extern crate stpack;
use stpack::Stpack;

use crate::types::{
    Header,
    AddrTblEntry,
    StrTblOff,
};

pub struct KAllSyms {
    base: usize,
    header: Header,
}

impl KAllSyms {
    pub fn new(base: usize) -> Self {
        let header = unsafe {
            core::slice::from_raw_parts(
                base as *const u8, Header::SIZE) };
        Self {
            base,
            header: Header::unpack_le(header).unwrap(),
        }
    }

    fn nth_addr(&self, i: usize) -> AddrTblEntry {
        use core::mem;
        let addr = self.base +
            self.header.addr_table_off as usize +
            ((mem::size_of::<AddrTblEntry>()) * i);
        let entry = addr as *const AddrTblEntry;
        unsafe { *entry }
    }

    fn get_u8_array(&self, table_off: u16, i: usize) -> &'static [u8] {
        use core::mem;
        let addr_table = self.base + table_off as usize;
        let addr_off = addr_table + ((mem::size_of::<StrTblOff>()) * i);
        let off = addr_table + unsafe { *(addr_off as *const StrTblOff) as usize };
        let ptr = off as *const u8;
        unsafe {
            core::slice::from_raw_parts(
                ptr.add(1), *ptr as usize) }
    }

    fn nth_token(&self, i: u8) -> &'static [u8] {
        return self.get_u8_array(
            self.header.token_table_off, i as usize);
    }

    fn safe_nth_name<'a>(&self, i: usize, buf: &'a mut [u8]) -> &'a str {
        use core::cmp::min;

        let tokens = self.get_u8_array(
            self.header.name_table_off, i);
        let mut buf_i: usize = 0;
        for tok_i in tokens {
            let token = self.nth_token(*tok_i);
            let wlen = min(buf.len() - buf_i, token.len());
            buf[buf_i..(buf_i + wlen)].copy_from_slice(&token[..wlen]);
            buf_i += wlen;
            if buf_i >= buf.len() {
                break
            }
        }
        core::str::from_utf8(&buf[..buf_i]).unwrap()
    }

    fn search_idx(&self, addr: AddrTblEntry) -> Option<usize> {
        if self.header.count == 0 {
            return None
        }
        if addr < self.nth_addr(0) {
            return None
        }

        let mut left: usize = 0;
        let mut right: usize = self.header.count as usize - 1;

        let idx = loop {
            if right - left < 2 {
                break if self.nth_addr(right) <= addr {
                    right
                } else {
                    left
                }
            }

            let center = (left + right) / 2;
            let center_addr = self.nth_addr(center);
            if center_addr <= addr {
                left = center;
            } else {
                right = center;
            }
        };

        Some(idx)
    }

    pub fn safe_search<'a>(&self, addr: AddrTblEntry, buf: &'a mut [u8])
                           -> Option<(&'a str, AddrTblEntry)> {
        match self.search_idx(addr) {
            None => None,
            Some(idx) => {
                let name = self.safe_nth_name(idx, buf);
                let off = addr - self.nth_addr(idx);
                Some((name, off))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use stpack::Stpack;

    use crate::{
        pack,
        types::Header,
    };

    #[test]
    fn normal1() {
        let data = pack(
            &vec![
                (String::from("alloc::vec::Vec<T>::new"), 0x1000),
                (String::from("alloc::raw_vec::alloc_guard"), 0x2000),
                (String::from("core::alloc::global::GlobalAlloc::realloc"), 0x3000),
            ]
        );

        let kallsyms = crate::KAllSyms::new(data.as_ptr() as usize);
        let mut namebuf: [u8; 30] = [0; 30];

        assert_eq!(kallsyms.safe_search(0x0fff, &mut namebuf),
                   None);

        assert_eq!(kallsyms.safe_search(0x1000, &mut namebuf),
                   Some(("alloc::vec::Vec<T>::new", 0)));

        assert_eq!(kallsyms.safe_search(0x1fff, &mut namebuf),
                   Some(("alloc::vec::Vec<T>::new", 0xfff)));

        assert_eq!(kallsyms.safe_search(0x2000, &mut namebuf),
                   Some(("alloc::raw_vec::alloc_guard", 0)));

        assert_eq!(kallsyms.safe_search(0x10000, &mut namebuf),
                   Some(("core::alloc::global::GlobalAll", 0xd000)));
    }

    #[test]
    fn empty() {
        let data: [u8; Header::SIZE] = [0; Header::SIZE];

        let kallsyms = crate::KAllSyms::new(data.as_ptr() as usize);
        let mut namebuf: [u8; 64] = [0; 64];

        assert_eq!(kallsyms.safe_search(0x1000, &mut namebuf), None);
    }
}
