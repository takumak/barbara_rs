#![cfg_attr(not(test), no_std)]

pub struct KAllSyms {
    base: usize,
    count: usize,
    addr_table_off: usize,
    name_table_off: usize,
    token_table_off: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct KAllSymsHeader {
    _reserved: u32,
    count: u16,
    addr_table_off: u16,
    name_table_off: u16,
    token_table_off: u16,
}

impl KAllSyms {
    pub const fn new(base: usize) -> Self {
        let header = unsafe { &*(base as *const KAllSymsHeader) };
        Self {
            base,
            count: header.count as usize,
            addr_table_off: header.addr_table_off as usize,
            name_table_off: header.name_table_off as usize,
            token_table_off: header.token_table_off as usize,
        }
    }

    fn nth_addr(&self, i: usize) -> usize {
        use core::mem;
        let addr = self.base +
            self.addr_table_off +
            ((mem::size_of::<u32>()) * i);
        let entry = addr as *const u32;
        unsafe { *entry as usize }
    }

    fn get_u8_array(&self, table_off: usize, i: usize) -> &'static [u8] {
        use core::mem;
        let addr_table = self.base + table_off;
        let addr_off = addr_table + ((mem::size_of::<u16>()) * i);
        let off = addr_table + unsafe { *(addr_off as *const u16) as usize };
        let ptr = off as *const u8;
        unsafe {
            core::slice::from_raw_parts(
                ptr.add(1),
                *ptr as usize
            )
        }
    }

    fn nth_token(&self, i: u8) -> &'static [u8] {
        return self.get_u8_array(
            self.token_table_off, i as usize);
    }

    fn safe_nth_name<'a>(&self, i: usize, buf: &'a mut [u8]) -> &'a str {
        use core::cmp::min;

        let tokens = self.get_u8_array(
            self.name_table_off, i);
        let mut buf_i: usize = 0;
        for tok_i in tokens {
            let token = self.nth_token(*tok_i);
            let wlen = min(buf.len() - buf_i, token.len());
            buf[buf_i..(buf_i + wlen)].copy_from_slice(&token[..wlen]);
            buf_i += token.len();
            if buf_i >= buf.len() {
                break
            }
        }
        core::str::from_utf8(&buf[..buf_i]).unwrap()
    }

    fn search_idx(&self, addr: usize) -> Option<usize> {
        if self.count == 0 {
            return None
        }
        if addr < self.nth_addr(0) {
            return None
        }

        let mut left: usize = 0;
        let mut right: usize = self.count - 1;

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

    pub fn safe_search<'a>(&self, addr: usize, buf: &'a mut [u8])
                           -> Option<(&'a str, usize)> {
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
