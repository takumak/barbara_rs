extern crate posix;
use posix::Errno;

extern crate stpack;
use stpack::{unpacker, Unpacker};

use crate::ElfSection;
use crate::err::ElfParserError;
use crate::ident::{ElfClass, ElfEndian};
use crate::string_table;

const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;

unpacker! {
    pub struct Elf32SymtabEntry {
        pub name: u32,
        pub value: u32,
        pub size: u32,
        pub info: u8,
        pub other: u8,
        pub shndx: u16,
    }
}

unpacker! {
    pub struct Elf64SymtabEntry {
        pub name: u32,
        pub info: u8,
        pub other: u8,
        pub shndx: u16,
        pub value: u64,
        pub size: u64,
    }
}

pub struct SymtabIterator<'a> {
    class: ElfClass,
    le: bool,
    sections: &'a Vec<ElfSection<'a>>,
    curr_secidx: usize,
    curr_symidx: usize,
    strtab: Option<(usize, Vec<&'a str>)>,
}

impl<'a> SymtabIterator<'a> {
    pub(crate) fn new(class: ElfClass,
                      endian: ElfEndian,
                      sections: &'a Vec<ElfSection<'a>>) -> Self
    {
        Self {
            class,
            le: endian == ElfEndian::ElfLE,
            sections,
            curr_secidx: 0,
            curr_symidx: 0,
            strtab: None,
        }
    }
}

impl<'a> Iterator for SymtabIterator<'a> {
    type Item = Result<(u64, &'a str), ElfParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut secidx = self.curr_secidx;
        let mut symidx = self.curr_symidx;
        let seccnt = self.sections.len();

        loop {
            if secidx >= seccnt {
                break;
            }

            let sec = &self.sections[secidx];
            if sec.typ == SHT_SYMTAB {
                if sec.entsize == 0 {
                    return Some(Err(ElfParserError::new(
                        Errno::EINVAL, String::from("Symtab section entry size is 0 (file broken)"))))
                }

                if symidx < (sec.content.len() / (sec.entsize as usize)) {
                    break;
                }
            }

            secidx += 1;
            symidx = 0;
        }

        self.curr_secidx = secidx;
        self.curr_symidx = symidx;

        if secidx >= seccnt {
            return None;
        }

        let sec = &self.sections[secidx];
        let data = &sec.content[(sec.entsize as usize * symidx)..];

        let (addr, nameidx) = match self.class {
            ElfClass::Elf32 =>
                match Elf32SymtabEntry::unpack(data, self.le) {
                    Ok((ent, _)) => (ent.value as u64, ent.name as usize),
                    Err(_) => return Some(Err(ElfParserError::new(
                        Errno::EINVAL, String::from("Failed to parse symtab entry")))),
                },
            ElfClass::Elf64 =>
                match Elf64SymtabEntry::unpack(data, self.le) {
                    Ok((ent, _)) => (ent.value, ent.name as usize),
                    Err(_) => return Some(Err(ElfParserError::new(
                        Errno::EINVAL, String::from("Failed to parse symtab entry")))),
                },
        };

        if self.strtab.is_none() || self.strtab.as_ref().unwrap().0 != sec.link as usize {
            let strtab_secidx = sec.link as usize;
            if strtab_secidx >= self.sections.len() {
                return Some(Err(ElfParserError::new(
                    Errno::EINVAL,
                    format!("Symtab refer invalid strtab section index: \
                             {} (must be less than {})",
                            strtab_secidx, self.sections.len()))));
            }

            let strtab = string_table::parse(
                self.sections[strtab_secidx].content);
            self.strtab = Some((strtab_secidx, strtab));
        }

        let strtab = &self.strtab.as_ref().unwrap().1;
        if nameidx >= strtab.len() {
            return Some(Err(ElfParserError::new(
                Errno::EINVAL,
                format!("Symbol name index out of range: \
                         {} (must be less than {})",
                        nameidx, strtab.len()))));
        }

        self.curr_symidx = symidx + 1;

        Some(Ok((addr, strtab[nameidx])))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ElfSection,
        ident::{
            ElfClass,
            ElfEndian,
        },
        symtab::{
            SymtabIterator,
            Elf32SymtabEntry,
            Elf64SymtabEntry,
            SHT_SYMTAB,
            SHT_STRTAB,
        },
        stpack::Unpacker,
    };

    #[test]
    fn elf32be_first_section_is_zero_length_symtab() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 2,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[] as &[u8],
            },

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 2,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[
                    0, 0, 0, 1,                 // name
                    0x11, 0x22, 0x33, 0x44,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b't', b'e', b's', b't', 0,
                ],
            },

        ];

        assert_eq!(
            SymtabIterator::new(ElfClass::Elf32,
                                ElfEndian::ElfBE,
                                &sections)
                .map(|r| r.unwrap())
                .collect::<Vec<(u64, &str)>>(),
            vec![(0x11223344u64, "test")]
        );
    }

    #[test]
    fn elf32be_invalid_symtab_entsize() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 1,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0, 0, 0, 1,                 // name
                    0x11, 0x22, 0x33, 0x44,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b't', b'e', b's', b't', 0,
                ],
            },

        ];

        let mut iter =
            SymtabIterator::new(
                ElfClass::Elf32, ElfEndian::ElfBE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }

    #[test]
    fn elf32be_incomplete_symtab() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 1,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64 - 1,
                content: &[
                    0, 0, 0, 1,                 // name
                    0x11, 0x22, 0x33, 0x44,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, // 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b't', b'e', b's', b't', 0,
                ],
            },

        ];

        let mut iter =
            SymtabIterator::new(
                ElfClass::Elf32, ElfEndian::ElfBE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }

    #[test]
    fn elf32be_symtab_link_out_of_range() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 2,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[
                    0, 0, 0, 1,                 // name
                    0x11, 0x22, 0x33, 0x44,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b't', b'e', b's', b't', 0,
                ],
            },

        ];

        let mut iter =
            SymtabIterator::new(
                ElfClass::Elf32, ElfEndian::ElfBE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }

    #[test]
    fn elf32be_symbol_name_index_out_of_range() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 1,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[
                    0, 0, 0, 2,                 // name
                    0x11, 0x22, 0x33, 0x44,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b't', b'e', b's', b't', 0,
                ],
            },

        ];

        let mut iter =
            SymtabIterator::new(
                ElfClass::Elf32, ElfEndian::ElfBE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab entry unexpectedly succeed");
    }

    #[test]
    fn elf32be_multi_strtab() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 2,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[
                    0, 0, 0, 1,                 // name
                    0x11, 0x22, 0x33, 0x44,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 3,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[
                    0, 0, 0, 1,                 // name
                    0x55, 0x66, 0x77, 0x88,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b'2', b'-', b'1', 0,
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b'3', b'-', b'1', 0,
                ],
            },

        ];

        assert_eq!(
            SymtabIterator::new(ElfClass::Elf32,
                                ElfEndian::ElfBE,
                                &sections)
                .map(|r| r.unwrap())
                .collect::<Vec<(u64, &str)>>(),
            vec![
                (0x11223344u64, "2-1"),
                (0x55667788u64, "3-1"),
            ]
        );
    }

    #[test]
    fn elf64le() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 1,
                info: 0,
                addralign: 0,
                entsize: Elf64SymtabEntry::SIZE as u64,
                content: &[
                    1, 0, 0, 0,                 // name
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                    0xff, 0xee, 0xdd, 0xcc,     // addr
                    0xbb, 0xaa, 0x99, 0x88,     // addr
                    0, 0, 0, 0,                 // size
                    0, 0, 0, 0,                 // size
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b't', b'e', b's', b't', 0,
                ],
            },

        ];

        assert_eq!(
            SymtabIterator::new(ElfClass::Elf64,
                                ElfEndian::ElfLE,
                                &sections)
                .map(|r| r.unwrap())
                .collect::<Vec<(u64, &str)>>(),
            vec![(0x8899aabbccddeeffu64, "test")]
        );
    }

    #[test]
    fn elf64le_incomplete_symtab() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 1,
                info: 0,
                addralign: 0,
                entsize: Elf64SymtabEntry::SIZE as u64 - 1,
                content: &[
                    1, 0, 0, 0,                 // name
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                    0xff, 0xee, 0xdd, 0xcc,     // addr
                    0xbb, 0xaa, 0x99, 0x88,     // addr
                    0, 0, 0, 0,                 // size
                    0, 0, 0, // 0,                 // size
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b't', b'e', b's', b't', 0,
                ],
            },

        ];

        let mut iter =
            SymtabIterator::new(
                ElfClass::Elf64, ElfEndian::ElfLE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }

    #[test]
    fn elf32be_multi_symtab() {
        let sections = vec![

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 2,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[
                    0, 0, 0, 1,                 // name
                    0x11, 0x22, 0x33, 0x44,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_SYMTAB,
                flags: 0,
                addr: 0,
                link: 2,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[
                    0, 0, 0, 2,                 // name
                    0xcc, 0xdd, 0xee, 0xff,     // addr
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ],
            },

            ElfSection {
                name: "",
                typ: SHT_STRTAB,
                flags: 0,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 0,
                entsize: 0,
                content: &[
                    0,
                    b'1', b'-', b'1', 0,
                    b'1', b'-', b'2', 0,
                ],
            },

        ];

        assert_eq!(
            SymtabIterator::new(ElfClass::Elf32,
                                ElfEndian::ElfBE,
                                &sections)
                .map(|r| r.unwrap())
                .collect::<Vec<(u64, &str)>>(),
            vec![
                (0x11223344u64, "1-1"),
                (0xccddeeffu64, "1-2"),
            ]
        );
    }
}
