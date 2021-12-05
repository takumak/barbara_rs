/*
 *  +00  +---------+
 *       |  magic  |                   4 bytes
 *  +04  +---------+
 *       |  count  |                   4 bytes
 *  +08  +---------+---------+
 *       |   addr  |   off   |         4+4 bytes
 *  +10  +---------+---------+
 *       |   addr  |   off   |         4+4 bytes
 *  +18  +---------+---------+
 *       |   addr  |   off   |         4+4 bytes
 *  +20  +---------+---------+
 *      ... `count` entries ...
 *       +-------------------+
 *       |  null separated   |
 *       |    name vector    |
 *       +-------------------+
 */

const MAGIC: u32 = 0xea805138;

#[derive(Clone, Copy)]
struct KAllSymsHeader {
    magic: u32,
    count: u32,
}

#[derive(Clone, Copy)]
struct SymbolEntry {
    addr: u32,
    name_off: u32,
}

struct KAllSyms {
    base: usize,
    count: usize,
}

#[derive(Clone, Copy)]
pub struct Symbol {
    pub addr: usize,
    name_addr: usize,
    name_len: usize,
}

impl Symbol {
    fn new(entry_ptr: *const SymbolEntry) -> Symbol {
        let entry = unsafe { *entry_ptr };

        let name_addr = entry_ptr as usize + entry.name_off as usize;
        let ptr = name_addr as *const u8;
        let mut name_len: usize = 0;
        loop {
            if unsafe { *ptr.add(name_len) } == 0 {
                break;
            }
            name_len += 1;
        };

        Symbol {
            addr: entry.addr as usize,
            name_addr,
            name_len,
        }
    }

    pub fn name(&self) -> &str {
        let name = unsafe {
            core::slice::from_raw_parts(
                self.name_addr as *const u8,
                self.name_len
            )
        };
        core::str::from_utf8(name).unwrap()
    }
}

struct SymbolIterator {
    ptr: *const SymbolEntry,
    end: *const SymbolEntry,
}

impl Iterator for SymbolIterator {
    type Item = Symbol;

    fn next(&mut self) -> Option<Symbol> {
        let curr = self.ptr;
        if unsafe { curr.offset_from(self.end) } >= 0 {
            None
        } else {
            self.ptr = unsafe { curr.add(1) };
            Some(Symbol::new(curr))
        }
    }
}

impl KAllSyms {
    fn new() -> KAllSyms {
        extern "C" {
            static __kallsyms: u8;
        }
        let base = unsafe { &__kallsyms as *const _ as usize };
        let header = unsafe { *(base as *const KAllSymsHeader) };
        let count = if header.magic == MAGIC {
            header.count
        } else {
            0
        };
        KAllSyms {
            base,
            count: count as usize,
        }
    }

    fn nth_entry(&self, i: usize) -> *const SymbolEntry {
        let top = self.base + 8;
        let ptr = top as *const SymbolEntry;
        unsafe { ptr.add(i) }
    }

    fn nth_addr(&self, i: usize) -> usize {
        let sym = self.nth_entry(i);
        unsafe { (*sym).addr as usize }
    }

    fn nth(&self, i: usize) -> Symbol {
        Symbol::new(self.nth_entry(i))
    }

    fn iter(&self) -> SymbolIterator {
        let top = self.base + 8;
        let ptr = top as *const SymbolEntry;
        let end = unsafe { ptr.add(self.count) };
        SymbolIterator { ptr, end }
    }

    fn search(&self, addr: usize) -> Option<Symbol> {
        if self.count == 0 {
            return None
        }
        if addr < self.nth_addr(0) {
            return None
        }

        let mut left = 0;
        let mut right = self.count;

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

        Some(self.nth(idx))
    }
}

#[allow(dead_code)]
pub fn walk(func: fn(usize, &str)) {
    let kallsyms = KAllSyms::new();
    for sym in kallsyms.iter() {
        func(sym.addr, sym.name())
    }
}

pub fn search(addr: usize) -> Option<Symbol> {
    KAllSyms::new().search(addr)
}
