extern crate stpack;
use stpack::unpacker;

use crate::struct_parser;

struct_parser! {
    pub trait ElfHeader;
    {
        pub struct Elf32Header;
        pub struct Elf64Header;
    }
    {
        pub typ:       {u16, u16,} get_type(u16);
        pub machine:   {u16, u16,} get_machine(u16);
        pub version:   {u32, u32,} get_version(u32);
        pub entry:     {u32, u64,} get_entry(u64);
        pub phoff:     {u32, u64,} get_phoff(u64);
        pub shoff:     {u32, u64,} get_shoff(u64);
        pub flags:     {u32, u32,} get_flags(u32);
        pub ehsize:    {u16, u16,} get_ehsize(u16);
        pub phentsize: {u16, u16,} get_phentsize(u16);
        pub phnum:     {u16, u16,} get_phnum(u16);
        pub shentsize: {u16, u16,} get_shentsize(u16);
        pub shnum:     {u16, u16,} get_shnum(u16);
        pub shstrndx:  {u16, u16,} get_shstrndx(u16);
    }
}

#[cfg(test)]
mod tests {
    use crate::header::{ElfHeader, Elf32Header, Elf64Header};
    use crate::stpack::Unpacker;

    #[test]
    fn elf32header() {
        let data: Vec<u8> = (0u8..0xffu8).collect();
        let (header32, _) = Elf32Header::unpack(&data, false).unwrap();
        let header: &dyn ElfHeader = &header32;

        assert_eq!(header32.typ,                          0x0001u16);
        assert_eq!(header32.machine,                      0x0203u16);
        assert_eq!(header32.version,                  0x04050607u32);
        assert_eq!(header32.entry,                    0x08090a0bu32);
        assert_eq!(header32.phoff,                    0x0c0d0e0fu32);
        assert_eq!(header32.shoff,                    0x10111213u32);
        assert_eq!(header32.flags,                    0x14151617u32);
        assert_eq!(header32.ehsize,                       0x1819u16);
        assert_eq!(header32.phentsize,                    0x1a1bu16);
        assert_eq!(header32.phnum,                        0x1c1du16);
        assert_eq!(header32.shentsize,                    0x1e1fu16);
        assert_eq!(header32.shnum,                        0x2021u16);
        assert_eq!(header32.shstrndx,                     0x2223u16);

        assert_eq!(header.get_type(),                     0x0001u16);
        assert_eq!(header.get_machine(),                  0x0203u16);
        assert_eq!(header.get_version(),              0x04050607u32);
        assert_eq!(header.get_entry(),       0x00000000_08090a0bu64);
        assert_eq!(header.get_phoff(),       0x00000000_0c0d0e0fu64);
        assert_eq!(header.get_shoff(),       0x00000000_10111213u64);
        assert_eq!(header.get_flags(),                0x14151617u32);
        assert_eq!(header.get_ehsize(),                   0x1819u16);
        assert_eq!(header.get_phentsize(),                0x1a1bu16);
        assert_eq!(header.get_phnum(),                    0x1c1du16);
        assert_eq!(header.get_shentsize(),                0x1e1fu16);
        assert_eq!(header.get_shnum(),                    0x2021u16);
        assert_eq!(header.get_shstrndx(),                 0x2223u16);
    }

    #[test]
    fn elf64header() {
        let data: Vec<u8> = (0u8..0xffu8).collect();
        let (header64, _) = Elf64Header::unpack(&data, false).unwrap();
        let header: &dyn ElfHeader = &header64;

        assert_eq!(header64.typ,                          0x0001u16);
        assert_eq!(header64.machine,                      0x0203u16);
        assert_eq!(header64.version,                  0x04050607u32);
        assert_eq!(header64.entry,           0x08090a0b_0c0d0e0fu64);
        assert_eq!(header64.phoff,           0x10111213_14151617u64);
        assert_eq!(header64.shoff,           0x18191a1b_1c1d1e1fu64);
        assert_eq!(header64.flags,                    0x20212223u32);
        assert_eq!(header64.ehsize,                       0x2425u16);
        assert_eq!(header64.phentsize,                    0x2627u16);
        assert_eq!(header64.phnum,                        0x2829u16);
        assert_eq!(header64.shentsize,                    0x2a2bu16);
        assert_eq!(header64.shnum,                        0x2c2du16);
        assert_eq!(header64.shstrndx,                     0x2e2fu16);

        assert_eq!(header.get_type(),                     0x0001u16);
        assert_eq!(header.get_machine(),                  0x0203u16);
        assert_eq!(header.get_version(),              0x04050607u32);
        assert_eq!(header.get_entry(),       0x08090a0b_0c0d0e0fu64);
        assert_eq!(header.get_phoff(),       0x10111213_14151617u64);
        assert_eq!(header.get_shoff(),       0x18191a1b_1c1d1e1fu64);
        assert_eq!(header.get_flags(),                0x20212223u32);
        assert_eq!(header.get_ehsize(),                   0x2425u16);
        assert_eq!(header.get_phentsize(),                0x2627u16);
        assert_eq!(header.get_phnum(),                    0x2829u16);
        assert_eq!(header.get_shentsize(),                0x2a2bu16);
        assert_eq!(header.get_shnum(),                    0x2c2du16);
        assert_eq!(header.get_shstrndx(),                 0x2e2fu16);
    }
}
