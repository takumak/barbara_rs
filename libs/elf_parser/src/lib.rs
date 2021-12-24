extern crate stpack;
use stpack::Unpacker;

mod err;
mod raw;
mod symtab;
mod symbol;

pub use err::ElfParserError;

pub use symbol::{
    ElfSymbol,
    ElfSymbolBind,
    ElfSymbolType,
};

pub use raw::{
    ident::{
        ElfClass,
        ElfEndian,
    },
};

use raw::{
    header::ElfHeader,
    section_header::{
        ElfSectionHeader,
        ElfSectionHeaderType,
    }
};
use symtab::ElfSymtabIterator;

#[derive(PartialEq, Debug)]
struct ElfSection<'a> {
    name: &'a [u8],
    typ: ElfSectionHeaderType,
    flags: u64,
    addr: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entsize: u64,
    content: &'a [u8],
}

#[derive(Debug)]
pub struct ElfParser<'a> {
    pub class: ElfClass,
    pub endian: ElfEndian,
    sections: Vec<ElfSection<'a>>,
}

impl<'a> ElfParser<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ElfParserError> {
        use raw::{
            ident::parse_ident,
            header::{
                Elf32Header,
                Elf64Header,
            },
            section_header::{
                Elf32SectionHeader,
                Elf64SectionHeader,
            },
        };

        let (class, endian) = parse_ident(data)?;
        if class == ElfClass::Elf32 {
            Self::parse_sections::<Elf32Header, Elf32SectionHeader>(data, class, endian)
        } else {
            Self::parse_sections::<Elf64Header, Elf64SectionHeader>(data, class, endian)
        }
    }

    fn parse_sections<H, SH>(data: &'a [u8], class: ElfClass, endian: ElfEndian) ->
        Result<Self, ElfParserError>
    where H: Unpacker + ElfHeader,
          SH: Unpacker + ElfSectionHeader
    {
        use raw::{
            strtab,
            section_parser::SectionParser,
        };

        let parser = SectionParser::<H, SH>::new(data, endian)?;
        let (_, strtab_data) = parser.nth(data, parser.header.get_shstrndx() as usize)?;

        let mut sections: Vec<ElfSection<'a>> = Vec::new();
        for idx in 0..parser.header.get_shnum() {
            let (sh, content) = parser.nth(data, idx as usize)?;
            let name = strtab::read_at(
                strtab_data, sh.get_name() as usize);
            let sec = ElfSection {
                name,
                typ: sh.get_type(),
                flags: sh.get_flags(),
                addr: sh.get_addr(),
                link: sh.get_link(),
                info: sh.get_info(),
                addralign: sh.get_addralign(),
                entsize: sh.get_entsize(),
                content,
            };
            sections.push(sec);
        }

        Ok(Self{
            class,
            endian,
            sections,
        })
    }

    pub fn iter_symbols(&'a self) -> ElfSymtabIterator<'a> {
        ElfSymtabIterator::new(
            self.class,
            self.endian,
            &self.sections)
    }
}


#[cfg(test)]
mod tests {
    use crate::{
        ElfClass,
        ElfEndian,
        ElfParser,
        ElfSection,
        ElfSymbol,
        ElfSectionHeaderType,
    };

    #[test]
    fn elf32be() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // header
            0, 2,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            0, 0, 0, 1,                 // version = 1
            0xaa, 0xbb, 0xcc, 0xdd,     // entry point
            0, 0, 0, 0,                 // ph_off
            0, 0, 0, 0x34,              // sh_off
            0, 0, 0, 0,                 // flags
            0, 0x34,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0, 0x28,                    // shentsize
            0, 1,                       // shnum
            0, 0,                       // shstrndx

            // .shstrtab section header
            0, 0, 0, 1,                 // name
            0, 0, 0, 3,                 // type = SHT_STRTAB
            0, 0, 0, 0x20,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0, 0, 0, 0x5c,              // offset
            0, 0, 0, 0x0b,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            0, 0, 0, 1,                 // addralign
            0, 0, 0, 0,                 // entsize

            // .shstrtab section content
            0,
            b'.', b's', b'h', b's',
            b't', b'r', b't', b'a',
            b'b', 0,
        ];

        let parser = ElfParser::from_bytes(&data).unwrap();
        assert_eq!(
            parser.sections,
            vec![
                ElfSection {
                    name: b".shstrtab",
                    typ: ElfSectionHeaderType::Strtab,
                    flags: 0x20,
                    addr: 0,
                    link: 0,
                    info: 0,
                    addralign: 1,
                    entsize: 0,
                    content: &[
                        0,
                        b'.', b's', b'h', b's',
                        b't', b'r', b't', b'a',
                        b'b', 0,
                    ],
                }
            ]
        );
    }

    #[test]
    fn elf32be_section_index_out_of_range() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // header
            0, 2,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            0, 0, 0, 1,                 // version = 1
            0xaa, 0xbb, 0xcc, 0xdd,     // entry point
            0, 0, 0, 0,                 // ph_off
            0, 0, 0, 0x34,              // sh_off
            0, 0, 0, 0,                 // flags
            0, 0x34,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0, 0x28,                    // shentsize
            0, 1,                       // shnum
            0, 1,                       // shstrndx

            // .shstrtab section header
            0, 0, 0, 1,                 // name
            0, 0, 0, 3,                 // type = SHT_STRTAB
            0, 0, 0, 0x20,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0, 0, 0, 0x5c,              // offset
            0, 0, 0, 0x0b,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            0, 0, 0, 1,                 // addralign
            0, 0, 0, 0,                 // entsize

            // .shstrtab section content
            0,
            b'.', b's', b'h', b's',
            b't', b'r', b't', b'a',
            b'b', 0,
        ];

        ElfParser::from_bytes(&data)
            .expect_err("ElfParser::from_bytes unexpectedly succeed");
    }

    #[test]
    fn elf32be_header_parse_error() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // header
            0, 2,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            0, 0, 0, 1,                 // version = 1
            0xaa, 0xbb, 0xcc, 0xdd,     // entry point
            0, 0, 0, 0,                 // ph_off
            0, 0, 0, 0x34,              // sh_off
            0, 0, 0, 0,                 // flags
            0, 0x34,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0, 0x28,                    // shentsize
            0, 1,                       // shnum
            0, // 0,                       // shstrndx
        ];

        ElfParser::from_bytes(&data)
            .expect_err("ElfParser::from_bytes unexpectedly succeed");
    }

    #[test]
    fn elf32be_section_content_incomplete() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // header
            0, 2,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            0, 0, 0, 1,                 // version = 1
            0xaa, 0xbb, 0xcc, 0xdd,     // entry point
            0, 0, 0, 0,                 // ph_off
            0, 0, 0, 0x34,              // sh_off
            0, 0, 0, 0,                 // flags
            0, 0x34,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0, 0x28,                    // shentsize
            0, 1,                       // shnum
            0, 0,                       // shstrndx

            // .shstrtab section header
            0, 0, 0, 1,                 // name
            0, 0, 0, 3,                 // type = SHT_STRTAB
            0, 0, 0, 0x20,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0, 0, 0, 0x5c,              // offset
            0, 0, 0, 0x0b,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            0, 0, 0, 1,                 // addralign
            0, 0, 0, 0,                 // entsize

            // .shstrtab section content
            0,
            b'.', b's', b'h', b's',
            b't', b'r', b't', b'a',
            b'b',// 0,
        ];

        ElfParser::from_bytes(&data)
            .expect_err("ElfParser::from_bytes unexpectedly succeed");
    }

    #[test]
    fn elf32be_section_header_after_contents() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // header
            0, 2,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            0, 0, 0, 1,                 // version = 1
            0xaa, 0xbb, 0xcc, 0xdd,     // entry point
            0, 0, 0, 0,                 // ph_off
            0, 0, 0, 0x3f,              // sh_off
            0, 0, 0, 0,                 // flags
            0, 0x34,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0, 0x28,                    // shentsize
            0, 1,                       // shnum
            0, 0,                       // shstrndx

            // .shstrtab section content
            0,
            b'.', b's', b'h', b's',
            b't', b'r', b't', b'a',
            b'b', 0,

            // .shstrtab section header
            0, 0, 0, 1,                 // name
            0, 0, 0, 3,                 // type = SHT_STRTAB
            0, 0, 0, 0x20,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0, 0, 0, 0x34,              // offset
            0, 0, 0, 0x0b,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            0, 0, 0, 1,                 // addralign
            0, 0, 0, 0,                 // entsize
        ];

        let parser = ElfParser::from_bytes(&data).unwrap();
        assert_eq!(
            parser.sections,
            vec![
                ElfSection {
                    name: b".shstrtab",
                    typ: ElfSectionHeaderType::Strtab,
                    flags: 0x20,
                    addr: 0,
                    link: 0,
                    info: 0,
                    addralign: 1,
                    entsize: 0,
                    content: &[
                        0,
                        b'.', b's', b'h', b's',
                        b't', b'r', b't', b'a',
                        b'b', 0,
                    ],
                }
            ]
        );
    }

    #[test]
    fn elf32be_shstrtab_header_incomplete() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // header
            0, 2,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            0, 0, 0, 1,                 // version = 1
            0xaa, 0xbb, 0xcc, 0xdd,     // entry point
            0, 0, 0, 0,                 // ph_off
            0, 0, 0, 0x3f,              // sh_off
            0, 0, 0, 0,                 // flags
            0, 0x34,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0, 0x28,                    // shentsize
            0, 1,                       // shnum
            0, 0,                       // shstrndx

            // .shstrtab section content
            0,
            b'.', b's', b'h', b's',
            b't', b'r', b't', b'a',
            b'b', 0,

            // .shstrtab section header
            0, 0, 0, 1,                 // name
            0, 0, 0, 3,                 // type = SHT_STRTAB
            0, 0, 0, 0x20,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0, 0, 0, 0x34,              // offset
            0, 0, 0, 0x0b,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            0, 0, 0, 1,                 // addralign
            0, 0, 0, // 0,                 // entsize
        ];

        ElfParser::from_bytes(&data)
            .expect_err("ElfParser::from_bytes unexpectedly succeed");
    }

    #[test]
    fn elfsection_partialeq() {
        let sec1 = ElfSection {
            name: b"foo",
            typ: ElfSectionHeaderType::Null,
            flags: 1,
            addr: 2,
            link: 3,
            info: 4,
            addralign: 5,
            entsize: 6,
            content: &[0],
        };

        let sec2 = ElfSection {
            name: b"bar",
            typ: ElfSectionHeaderType::Null,
            flags: 1,
            addr: 2,
            link: 3,
            info: 4,
            addralign: 5,
            entsize: 6,
            content: &[0],
        };

        assert_ne!(sec1, sec2);
    }

    #[test]
    fn elfsection_debug() {
        let sec = ElfSection {
            name: b"foo",
            typ: ElfSectionHeaderType::Null,
            flags: 1,
            addr: 2,
            link: 3,
            info: 4,
            addralign: 5,
            entsize: 6,
            content: &[0],
        };

        assert_eq!(format!("{:?}", sec),
                   "ElfSection { \
                    name: [102, 111, 111], \
                    typ: Null, \
                    flags: 1, \
                    addr: 2, \
                    link: 3, \
                    info: 4, \
                    addralign: 5, \
                    entsize: 6, \
                    content: [0] \
                    }");
    }

    #[test]
    fn elfparser_debug() {
        let p = ElfParser {
            class: ElfClass::Elf32,
            endian: ElfEndian::ElfLE,
            sections: vec![],
        };

        assert_eq!(format!("{:?}", p),
                   "ElfParser { \
                    class: Elf32, \
                    endian: ElfLE, \
                    sections: [] \
                    }");
    }

    #[test]
    fn elf32be_ident_incomplete() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, //0,        // padding
        ];

        ElfParser::from_bytes(&data)
            .expect_err("ElfParser::from_bytes unexpectedly succeed");
    }

    #[test]
    fn elf64le_iter_symbols() {
        let data: &[u8] = &[
            // +0000 ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            2,                          // 1: 32bit, 2: 64bit, others: error
            1,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // +0010 header
            2, 0,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            1, 0, 0, 0,                 // version = 1
            0, 0, 0, 0, 0, 0, 0, 0,     // entry point
            0, 0, 0, 0, 0, 0, 0, 0,     // ph_off
            0x40, 0, 0, 0, 0, 0, 0, 0,  // sh_off
            0, 0, 0, 0,                 // flags
            0x40, 0,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0x40, 0,                    // shentsize
            3, 0,                       // shnum
            0, 0,                       // shstrndx

            // +0040 .shstrtab section header
            1, 0, 0, 0,                 // name
            3, 0, 0, 0,                 // type = SHT_STRTAB
            0x20, 0, 0, 0, 0, 0, 0, 0,  // flags = SHF_STRINGS
            0, 0, 0, 0, 0, 0, 0, 0,     // addr
            0x00, 1, 0, 0, 0, 0, 0, 0,  // offset
            0x1b, 0, 0, 0, 0, 0, 0, 0,  // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            1, 0, 0, 0, 0, 0, 0, 0,     // addralign
            0, 0, 0, 0, 0, 0, 0, 0,     // entsize

            // +0080 .strtab section header
            0xb, 0, 0, 0,               // name
            3, 0, 0, 0,                 // type = SHT_STRTAB
            0x20, 0, 0, 0, 0, 0, 0, 0,  // flags = SHF_STRINGS
            0, 0, 0, 0, 0, 0, 0, 0,     // addr
            0x20, 1, 0, 0, 0, 0, 0, 0,  // offset
            0x0d, 0, 0, 0, 0, 0, 0, 0,  // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            1, 0, 0, 0, 0, 0, 0, 0,     // addralign
            0, 0, 0, 0, 0, 0, 0, 0,     // entsize

            // +00c0 .symtab section header
            0x13, 0, 0, 0,              // name
            2, 0, 0, 0,                 // type = SHT_SYMTAB
            0, 0, 0, 0, 0, 0, 0, 0,     // flags = 0
            0, 0, 0, 0, 0, 0, 0, 0,     // addr
            0x30, 1, 0, 0, 0, 0, 0, 0,  // offset
            0x30, 0, 0, 0, 0, 0, 0, 0,  // size
            1, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            1, 0, 0, 0, 0, 0, 0, 0,     // addralign
            0x18, 0, 0, 0, 0, 0, 0, 0,  // entsize

            // +0100 .shstrtab section content
            0, b'.', b's', b'h',
            b's', b't', b'r', b't',
            b'a', b'b', 0, b'.',
            b's', b't', b'r', b't',
            b'a', b'b', 0, b'.',
            b's', b'y', b'm', b't',
            b'a', b'b', 0, 0,
            0, 0, 0, 0,

            // +0120 .strtab section content
            0, b't', b'e', b's',
            b't', b'1', 0, b't',
            b'e', b's', b't', b'2',
            0, 0, 0, 0,

            // +0120 .symtab section content
            // +0120 symtab[0]
            1, 0, 0, 0,                 // name
            0,                          // info
            0,                          // other
            0, 0,                       // shndx
            0, 1, 2, 3, 4, 5, 6, 7,     // value
            0, 0, 0, 0, 0, 0, 0, 0,     // size
            // +0138 symtab[1]
            7, 0, 0, 0,                 // name
            0,                          // info
            0,                          // other
            0, 0,                       // shndx
            8, 9, 0, 1, 2, 3, 4, 5,     // value
            0, 0, 0, 0, 0, 0, 0, 0,     // size
        ];

        let p = ElfParser::from_bytes(&data)
            .expect("ElfParser::from_bytes failed");

        assert_eq!(
            p.sections[0],
            ElfSection {
                name: b".shstrtab",
                typ: ElfSectionHeaderType::Strtab,
                flags: 0x20,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 1,
                entsize: 0,
                content: &[
                    0, b'.', b's', b'h',
                    b's', b't', b'r', b't',
                    b'a', b'b', 0, b'.',
                    b's', b't', b'r', b't',
                    b'a', b'b', 0, b'.',
                    b's', b'y', b'm', b't',
                    b'a', b'b', 0,
                ]
            }
        );

        assert_eq!(
            p.sections[1],
            ElfSection {
                name: b".strtab",
                typ: ElfSectionHeaderType::Strtab,
                flags: 0x20,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 1,
                entsize: 0,
                content: &[
                    0, b't', b'e', b's',
                    b't', b'1', 0, b't',
                    b'e', b's', b't', b'2',
                    0,
                ]
            }
        );

        assert_eq!(
            p.sections[2],
            ElfSection {
                name: b".symtab",
                typ: ElfSectionHeaderType::Symtab,
                flags: 0,
                addr: 0,
                link: 1,
                info: 0,
                addralign: 1,
                entsize: 0x18,
                content: &[
                    // +0120 symtab[0]
                    1, 0, 0, 0,                 // name
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                    0, 1, 2, 3, 4, 5, 6, 7,     // value
                    0, 0, 0, 0, 0, 0, 0, 0,     // size
                    // +0138 symtab[1]
                    7, 0, 0, 0,                 // name
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                    8, 9, 0, 1, 2, 3, 4, 5,     // value
                    0, 0, 0, 0, 0, 0, 0, 0,     // size
                ]
            }
        );

        assert_eq!(p.sections.len(), 3);

        let syms: Vec<ElfSymbol> =
            p.iter_symbols().map(|r| r.unwrap()).collect();

        assert_eq!(
            syms,
            vec![
                ElfSymbol {
                    name: b"test1",
                    value: 0x07060504_03020100u64,
                    size: 0,
                    info: 0,
                    other: 0,
                    shndx: 0,
                },
                ElfSymbol {
                    name: b"test2",
                    value: 0x05040302_01000908u64,
                    size: 0,
                    info: 0,
                    other: 0,
                    shndx: 0,
                },
            ]
        );
    }

    #[test]
    fn elf32le_iter_symbols() {
        let data: &[u8] = &[
            // +0000 ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            1,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // +0010 header
            2, 0,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            1, 0, 0, 0,                 // version = 1
            0, 0, 0, 0,                 // entry point
            0, 0, 0, 0,                 // ph_off
            0x34, 0, 0, 0,              // sh_off
            0, 0, 0, 0,                 // flags
            0x34, 0,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0x28, 0,                    // shentsize
            3, 0,                       // shnum
            0, 0,                       // shstrndx

            // +0034 .shstrtab section header
            1, 0, 0, 0,                 // name
            3, 0, 0, 0,                 // type = SHT_STRTAB
            0x20, 0, 0, 0,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0xac, 0, 0, 0,              // offset
            0x1b, 0, 0, 0,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            1, 0, 0, 0,                 // addralign
            0, 0, 0, 0,                 // entsize

            // +005c .strtab section header
            0xb, 0, 0, 0,               // name
            3, 0, 0, 0,                 // type = SHT_STRTAB
            0x20, 0, 0, 0,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0xcc, 0, 0, 0,              // offset
            0x0d, 0, 0, 0,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            1, 0, 0, 0,                 // addralign
            0, 0, 0, 0,                 // entsize

            // +0084 .symtab section header
            0x13, 0, 0, 0,              // name
            2, 0, 0, 0,                 // type = SHT_SYMTAB
            0, 0, 0, 0,                 // flags = 0
            0, 0, 0, 0,                 // addr
            0xdc, 0, 0, 0,              // offset
            0x20, 0, 0, 0,              // size
            1, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            1, 0, 0, 0,                 // addralign
            0x10, 0, 0, 0,              // entsize

            // +00ac .shstrtab section content
            0, b'.', b's', b'h',
            b's', b't', b'r', b't',
            b'a', b'b', 0, b'.',
            b's', b't', b'r', b't',
            b'a', b'b', 0, b'.',
            b's', b'y', b'm', b't',
            b'a', b'b', 0, 0,
            0, 0, 0, 0,

            // +00cc .strtab section content
            0, b't', b'e', b's',
            b't', b'1', 0, b't',
            b'e', b's', b't', b'2',
            0, 0, 0, 0,

            // +00dc .symtab section content
            // +00dc symtab[0]
            1, 0, 0, 0,                 // name
            0, 1, 2, 3,                 // value
            0, 0, 0, 0,                 // size
            0,                          // info
            0,                          // other
            0, 0,                       // shndx
            // +00ec symtab[1]
            7, 0, 0, 0,                 // name
            8, 9, 0, 1,                 // value
            0, 0, 0, 0,                 // size
            0,                          // info
            0,                          // other
            0, 0,                       // shndx
        ];

        let p = ElfParser::from_bytes(&data)
            .expect("ElfParser::from_bytes failed");

        assert_eq!(
            p.sections[0],
            ElfSection {
                name: b".shstrtab",
                typ: ElfSectionHeaderType::Strtab,
                flags: 0x20,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 1,
                entsize: 0,
                content: &[
                    0, b'.', b's', b'h',
                    b's', b't', b'r', b't',
                    b'a', b'b', 0, b'.',
                    b's', b't', b'r', b't',
                    b'a', b'b', 0, b'.',
                    b's', b'y', b'm', b't',
                    b'a', b'b', 0,
                ]
            }
        );

        assert_eq!(
            p.sections[1],
            ElfSection {
                name: b".strtab",
                typ: ElfSectionHeaderType::Strtab,
                flags: 0x20,
                addr: 0,
                link: 0,
                info: 0,
                addralign: 1,
                entsize: 0,
                content: &[
                    0, b't', b'e', b's',
                    b't', b'1', 0, b't',
                    b'e', b's', b't', b'2',
                    0,
                ]
            }
        );

        assert_eq!(
            p.sections[2],
            ElfSection {
                name: b".symtab",
                typ: ElfSectionHeaderType::Symtab,
                flags: 0,
                addr: 0,
                link: 1,
                info: 0,
                addralign: 1,
                entsize: 0x10,
                content: &[
                    // +0120 symtab[0]
                    1, 0, 0, 0,                 // name
                    0, 1, 2, 3,                 // value
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                    // +0138 symtab[1]
                    7, 0, 0, 0,                 // name
                    8, 9, 0, 1,                 // value
                    0, 0, 0, 0,                 // size
                    0,                          // info
                    0,                          // other
                    0, 0,                       // shndx
                ]
            }
        );

        assert_eq!(p.sections.len(), 3);

        let syms: Vec<ElfSymbol> =
            p.iter_symbols().map(|r| r.unwrap()).collect();

        assert_eq!(
            syms,
            vec![
                ElfSymbol {
                    name: b"test1",
                    value: 0x03020100u64,
                    size: 0,
                    info: 0,
                    other: 0,
                    shndx: 0,
                },

                ElfSymbol {
                    name: b"test2",
                    value: 0x01000908u64,
                    size: 0,
                    info: 0,
                    other: 0,
                    shndx: 0,
                },
            ]
        );
    }

    #[test]
    fn elf32le_second_section_header_incomplete() {
        let data: &[u8] = &[
            // +0000 ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            1,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // +0010 header
            2, 0,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            1, 0, 0, 0,                 // version = 1
            0, 0, 0, 0,                 // entry point
            0, 0, 0, 0,                 // ph_off
            0x34, 0, 0, 0,              // sh_off
            0, 0, 0, 0,                 // flags
            0x34, 0,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0x28, 0,                    // shentsize
            2, 0,                       // shnum
            0, 0,                       // shstrndx

            // +0034 .shstrtab section header
            1, 0, 0, 0,                 // name
            3, 0, 0, 0,                 // type = SHT_STRTAB
            0x20, 0, 0, 0,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0x34, 0, 0, 0,              // offset
            0, 0, 0, 0,                 // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            1, 0, 0, 0,                 // addralign
            0, 0, 0, 0,                 // entsize

            // +005c .strtab section header
            0xb, 0, 0, 0,               // name
            3, 0, 0, 0,                 // type = SHT_STRTAB
            0x20, 0, 0, 0,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0x5c, 0, 0, 0,              // offset
            0, 0, 0, 0,                 // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            1, 0, 0, 0,                 // addralign
            0, 0, 0, // 0,                 // entsize
        ];

        ElfParser::from_bytes(&data)
            .expect_err("ElfParser::from_bytes unexpectedly succeed");
    }

    #[test]
    fn elf32be_too_small_shentsize() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // header
            0, 2,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            0, 0, 0, 1,                 // version = 1
            0xaa, 0xbb, 0xcc, 0xdd,     // entry point
            0, 0, 0, 0,                 // ph_off
            0, 0, 0, 0x34,              // sh_off
            0, 0, 0, 0,                 // flags
            0, 0x34,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0, 0x27,                    // shentsize
            0, 1,                       // shnum
            0, 0,                       // shstrndx

            // .shstrtab section header
            0, 0, 0, 1,                 // name
            0, 0, 0, 3,                 // type = SHT_STRTAB
            0, 0, 0, 0x20,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0, 0, 0, 0x5c,              // offset
            0, 0, 0, 0x0b,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            0, 0, 0, 1,                 // addralign
            0, 0, 0, 0,                 // entsize

            // .shstrtab section content
            0,
            b'.', b's', b'h', b's',
            b't', b'r', b't', b'a',
            b'b', 0,
        ];

        ElfParser::from_bytes(&data)
            .expect_err("ElfParser::from_bytes unexpectedly succeed");
    }

    #[test]
    fn elf32be_nobits() {
        let data: &[u8] = &[
            // ident
            0x7f, b'E', b'L', b'F',     // magic; should be [0x7f, 'E', 'L', 'F']
            1,                          // 1: 32bit, 2: 64bit, others: error
            2,                          // 1: Little endian, 2: Big endian, others: error
            1,                          // elf version; should be 1
            3,                          // OS ABI
            0,                          // ABI version
            0, 0, 0, 0, 0, 0, 0,        // padding

            // header
            0, 2,                       // type = ET_EXEC (executable file)
            0, 0,                       // machine = EM_NONE
            0, 0, 0, 1,                 // version = 1
            0xaa, 0xbb, 0xcc, 0xdd,     // entry point
            0, 0, 0, 0,                 // ph_off
            0, 0, 0, 0x34,              // sh_off
            0, 0, 0, 0,                 // flags
            0, 0x34,                    // ehsize
            0, 0,                       // phentsize
            0, 0,                       // phnum
            0, 0x28,                    // shentsize
            0, 2,                       // shnum
            0, 0,                       // shstrndx

            // .shstrtab section header
            0, 0, 0, 1,                 // name
            0, 0, 0, 3,                 // type = SHT_STRTAB
            0, 0, 0, 0x20,              // flags = SHF_STRINGS
            0, 0, 0, 0,                 // addr
            0, 0, 0, 0x84,              // offset
            0, 0, 0, 0x10,              // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            0, 0, 0, 1,                 // addralign
            0, 0, 0, 0,                 // entsize

            // .shstrtab section header
            0, 0, 0, 0xb,               // name
            0, 0, 0, 8,                 // type = SHT_NOBITS
            0, 0, 0, 0,                 // flags = 0
            0, 0, 0x01, 0x00,           // addr
            0, 0, 0x01, 0x00,           // offset
            0, 0, 0x01, 0x00,           // size
            0, 0, 0, 0,                 // link
            0, 0, 0, 0,                 // info
            0, 0, 0, 1,                 // addralign
            0, 0, 0, 0,                 // entsize

            // .shstrtab section content
            0,
            b'.', b's', b'h', b's',
            b't', b'r', b't', b'a',
            b'b', 0, b'.', b'b',
            b's', b's', 0
        ];

        let parser = ElfParser::from_bytes(&data).unwrap();
        assert_eq!(
            parser.sections,

            vec![
                ElfSection {
                    name: b".shstrtab",
                    typ: ElfSectionHeaderType::Strtab,
                    flags: 0x20,
                    addr: 0,
                    link: 0,
                    info: 0,
                    addralign: 1,
                    entsize: 0,
                    content: &[
                        0,
                        b'.', b's', b'h', b's',
                        b't', b'r', b't', b'a',
                        b'b', 0, b'.', b'b', b's',
                        b's', 0
                    ],
                },

                ElfSection {
                    name: b".bss",
                    typ: ElfSectionHeaderType::Nobits,
                    flags: 0,
                    addr: 0x0100,
                    link: 0,
                    info: 0,
                    addralign: 1,
                    entsize: 0,
                    content: &[ ],
                }
            ]
        );
    }
}
