#![cfg_attr(not(test), no_std)]

pub struct KAllSyms {
    base: usize,
    count: usize,
    addr_table_off: usize,
    name_table_off: usize,
    token_table_off: usize,
}

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

#[cfg(test)]
mod tests {
    struct TokenTable {
        tokens: Vec<Vec<u8>>,
    }

    impl TokenTable {
        fn new(tokens: &[&str]) -> Self {
            let mut tokens_vec: Vec<Vec<u8>> = Vec::new();
            for tok in tokens {
                tokens_vec.push(tok.as_bytes().to_vec());
            }
            tokens_vec.extend(
                std::iter::repeat(Vec::new())
                    .take(256 - tokens_vec.len()));
            Self { tokens: tokens_vec }
        }

        fn find(&self, token: &str) -> Option<u8> {
            let token = token.as_bytes().to_vec();
            match self.tokens.iter().position(|t| *t == token) {
                None => None,
                Some(pos) => Some(pos as u8),
            }
        }
    }

    fn make_u8array_table(rows: &Vec<Vec<u8>>) -> Vec<u8> {
        let mut index: Vec<u16> = vec![];
        let mut body: Vec<u8> = vec![];
        for row in rows {
            index.push(body.len() as u16);
            body.push(row.len() as u8);
            body.extend(row.iter());
        }

        let mut table: Vec<u8> = vec![];
        let index_size = (index.len() * 2) as u16;
        for off in index {
            let off = index_size + off;
            table.extend(off.to_le_bytes());
        }
        table.append(&mut body);

        table
    }

    fn make_name_table(token_table: &TokenTable, names: &[&[&str]]) -> Vec<Vec<u8>> {
        let mut names_vec: Vec<Vec<u8>> = Vec::new();
        for tokens in names {
            let mut tokens_vec: Vec<u8> = Vec::new();
            tokens_vec.extend(tokens.iter().map(|t| {
                match token_table.find(t) {
                    None => panic!("Token not found: {}", t),
                    Some(i) => i,
                }
            }));
            names_vec.push(tokens_vec);
        }
        names_vec
    }

    fn make_kallsyms_data(tokens: &[&str], symbols: &[(u32, &[&str])]) -> Vec<u8> {
        let token_table = TokenTable::new(tokens);
        let names: Vec<&[&str]> = symbols.iter().map(|(_, tokens)| *tokens).collect();
        let name_table = make_name_table(&token_table, names.as_slice());

        let mut name_table_bin = make_u8array_table(&name_table);
        let mut token_table_bin = make_u8array_table(&token_table.tokens);

        let _reserved: u32 = 0;
        let count: u16 = symbols.len() as u16;
        let addr_table_off: u16 = 12;
        let name_table_off: u16 = addr_table_off + (4 * count);
        let token_table_off: u16 = name_table_off + (name_table_bin.len() as u16);

        let mut result: Vec<u8> = Vec::new();
        result.extend(_reserved.to_le_bytes());
        result.extend(count.to_le_bytes());
        result.extend(addr_table_off.to_le_bytes());
        result.extend(name_table_off.to_le_bytes());
        result.extend(token_table_off.to_le_bytes());
        for (addr, _) in symbols {
            result.extend(addr.to_le_bytes());
        }
        result.append(&mut name_table_bin);
        result.append(&mut token_table_bin);

        result
    }

    #[test]
    fn normal1() {
        let data = make_kallsyms_data(
            &[
                "t", "es", "t_f", "unction", "1",
                "test_function2",
                "test_f", "3",
            ],
            &[
                (0x1000, &["test_f", "unction", "3"]),
                (0x2000, &["test_function2"]),
                (0x3000, &["t", "es", "t_f", "unction", "1"]),
            ]
        );

        let kallsyms = crate::KAllSyms::new(data.as_ptr() as usize);
        let mut namebuf: [u8; 64] = [0; 64];

        assert_eq!(kallsyms.safe_search(0x0fff, &mut namebuf),
                   None);

        assert_eq!(kallsyms.safe_search(0x1000, &mut namebuf),
                   Some(("test_function3", 0usize)));

        assert_eq!(kallsyms.safe_search(0x1fff, &mut namebuf),
                   Some(("test_function3", 0xfffusize)));

        assert_eq!(kallsyms.safe_search(0x2000, &mut namebuf),
                   Some(("test_function2", 0usize)));

        assert_eq!(kallsyms.safe_search(0x10000, &mut namebuf),
                   Some(("test_function1", 0xd000usize)));
    }

    #[test]
    fn empty() {
        let data: [u8; 12] = [0; 12];

        let kallsyms = crate::KAllSyms::new(data.as_ptr() as usize);
        let mut namebuf: [u8; 64] = [0; 64];

        assert_eq!(kallsyms.safe_search(0x1000, &mut namebuf),
                   None);
    }
}
