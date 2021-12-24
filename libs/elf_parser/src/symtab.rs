extern crate posix;
use posix::Errno;

extern crate stpack;
use stpack::{unpacker, Unpacker};

use crate::{
    ElfSection,
    ElfSymbol,
};

use crate::err::ElfParserError;
use crate::raw::{
    ident::{
        ElfClass,
        ElfEndian,
    },
    section_header::ElfSectionHeaderType,
};

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

pub struct ElfSymtabIterator<'a> {
    class: ElfClass,
    le: bool,
    sections: &'a Vec<ElfSection<'a>>,
    curr_secidx: usize,
    curr_symidx: usize,
}

impl<'a> ElfSymtabIterator<'a> {
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
        }
    }
}

impl<'a> Iterator for ElfSymtabIterator<'a> {
    type Item = Result<ElfSymbol<'a>, ElfParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut secidx = self.curr_secidx;
        let mut symidx = self.curr_symidx;
        let seccnt = self.sections.len();

        loop {
            if secidx >= seccnt {
                break;
            }

            let sec = &self.sections[secidx];
            if sec.typ == ElfSectionHeaderType::Symtab {
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

        let (nameoff, value, size, info, other, shndx) = match self.class {
            ElfClass::Elf32 =>
                match Elf32SymtabEntry::unpack(data, self.le) {
                    Ok((ent, _)) => (
                        ent.name as usize,
                        ent.value as u64,
                        ent.size as u64,
                        ent.info,
                        ent.other,
                        ent.shndx,
                    ),
                    Err(_) => return Some(Err(ElfParserError::new(
                        Errno::EINVAL, String::from("Failed to parse symtab entry")))),
                },
            ElfClass::Elf64 =>
                match Elf64SymtabEntry::unpack(data, self.le) {
                    Ok((ent, _)) => (
                        ent.name as usize,
                        ent.value,
                        ent.size,
                        ent.info,
                        ent.other,
                        ent.shndx,
                    ),
                    Err(_) => return Some(Err(ElfParserError::new(
                        Errno::EINVAL, String::from("Failed to parse symtab entry")))),
                },
        };

        if sec.link as usize >= self.sections.len() {
            return Some(Err(ElfParserError::new(
                Errno::EINVAL,
                format!("Symtab refer invalid strtab section index: \
                         {} (must be less than {})",
                        sec.link, self.sections.len()))));
        }

        let strtab_sec = &self.sections[sec.link as usize];
        if strtab_sec.typ != ElfSectionHeaderType::Strtab {
            return Some(Err(ElfParserError::new(
                Errno::EINVAL,
                format!("Symtab linked section is not SHT_STRTAB: {}", sec.link))));
        }

        use crate::raw::strtab;

        let name = strtab::read_at(
            strtab_sec.content, nameoff);

        self.curr_symidx = symidx + 1;

        Some(Ok(ElfSymbol {
            name,
            value,
            size,
            info,
            other,
            shndx,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ElfSection,
        ElfSymbol,
        raw::ident::{
            ElfClass,
            ElfEndian,
        },
        raw::section_header::ElfSectionHeaderType,
        symtab::{
            ElfSymtabIterator,
            Elf32SymtabEntry,
            Elf64SymtabEntry,
        },
        stpack::Unpacker,
    };

    #[test]
    fn elf32be_first_section_is_zero_length_symtab() {
        let sections = vec![

            ElfSection {
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
                flags: 0,
                addr: 0,
                link: 2,
                info: 0,
                addralign: 0,
                entsize: Elf32SymtabEntry::SIZE as u64,
                content: &[] as &[u8],
            },

            ElfSection {
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
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
                name: b"",
                typ: ElfSectionHeaderType::Strtab,
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
            ElfSymtabIterator::new(ElfClass::Elf32,
                                   ElfEndian::ElfBE,
                                   &sections)
                .map(|r| r.unwrap())
                .collect::<Vec<ElfSymbol>>(),
            vec![
                ElfSymbol {
                    name: b"test",
                    value: 0x11223344u64,
                    size: 0,
                    info: 0,
                    other: 0,
                    shndx: 0,
                },
            ]
        );
    }

    #[test]
    fn elf32be_invalid_symtab_entsize() {
        let sections = vec![

            ElfSection {
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
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
                name: b"",
                typ: ElfSectionHeaderType::Strtab,
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
            ElfSymtabIterator::new(
                ElfClass::Elf32, ElfEndian::ElfBE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }

    #[test]
    fn elf32be_incomplete_symtab() {
        let sections = vec![

            ElfSection {
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
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
                name: b"",
                typ: ElfSectionHeaderType::Strtab,
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
            ElfSymtabIterator::new(
                ElfClass::Elf32, ElfEndian::ElfBE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }

    #[test]
    fn elf32be_symtab_link_out_of_range() {
        let sections = vec![

            ElfSection {
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
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
                name: b"",
                typ: ElfSectionHeaderType::Strtab,
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
            ElfSymtabIterator::new(
                ElfClass::Elf32, ElfEndian::ElfBE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }

    #[test]
    fn elf64le() {
        let sections = vec![

            ElfSection {
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
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
                name: b"",
                typ: ElfSectionHeaderType::Strtab,
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
            ElfSymtabIterator::new(ElfClass::Elf64,
                                   ElfEndian::ElfLE,
                                   &sections)
                .map(|r| r.unwrap())
                .collect::<Vec<ElfSymbol>>(),
            vec![
                ElfSymbol {
                    name: b"test",
                    value: 0x8899aabb_ccddeeffu64,
                    size: 0,
                    info: 0,
                    other: 0,
                    shndx: 0,
                },
            ]
        );
    }

    #[test]
    fn elf64le_incomplete_symtab() {
        let sections = vec![

            ElfSection {
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
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
                name: b"",
                typ: ElfSectionHeaderType::Strtab,
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
            ElfSymtabIterator::new(
                ElfClass::Elf64, ElfEndian::ElfLE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }

    #[test]
    fn elf32be_shstrtab_invalid_type() {
        let sections = vec![

            ElfSection {
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
                flags: 0,
                addr: 0,
                link: 1,
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
                name: b"",
                typ: ElfSectionHeaderType::Symtab,
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
            ElfSymtabIterator::new(
                ElfClass::Elf32, ElfEndian::ElfBE, &sections);

        iter.next().unwrap().expect_err(
            "Parsing broken symtab unexpectedly succeed");
    }
}
