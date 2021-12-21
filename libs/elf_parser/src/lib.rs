extern crate stpack;
use stpack::Unpacker;

mod err;
mod string_table;
mod ident;
mod header;
mod section_header;
mod raw_section_parser;
mod struct_parser;

use err::ElfParserError;
use ident::{ElfClass, ElfIdent, ELF_IDENT_SIZE};
use header::{ElfHeader, Elf32Header, Elf64Header};
use section_header::{ElfSectionHeader, Elf32SectionHeader, Elf64SectionHeader};
use raw_section_parser::RawSectionParser;

#[derive(PartialEq, Debug)]
struct ElfSection<'a> {
    name: &'a str,
    typ: u32,
    flags: u64,
    addr: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entsize: u64,
    content: &'a [u8],
}

#[derive(Debug)]
struct ElfParser<'a> {
    data: &'a [u8],
    ident: ElfIdent,
    sections: Vec<ElfSection<'a>>,
}

impl<'a> ElfParser<'a> {
    fn parse(data: &'a [u8]) -> Result<Self, ElfParserError> {
        let ident = ident::parse_ident(data)?;
        if ident.class == ElfClass::Elf32 {
            Self::parse_sections::<Elf32Header, Elf32SectionHeader>(data, ident)
        } else {
            Self::parse_sections::<Elf64Header, Elf64SectionHeader>(data, ident)
        }
    }

    fn parse_sections<H, SH>(data: &'a [u8], ident: ElfIdent) ->
        Result<Self, ElfParserError>
    where H: Unpacker + ElfHeader,
          SH: Unpacker + ElfSectionHeader
    {
        let parser = RawSectionParser::<H, SH>::new(data, &ident)?;
        let (_, str_tbl) = parser.nth(data, parser.header.get_shstrndx() as usize)?;

        let string_table: Vec<&str> = str_tbl
            .split(|&c| c == 0)
            .map(|s| core::str::from_utf8(s).unwrap_or(""))
            .collect();

        let mut sections: Vec<ElfSection<'a>> = Vec::new();
        for idx in 0..parser.header.get_shnum() {
            let (sh, content) = parser.nth(data, idx as usize)?;
            let sec = ElfSection {
                name: string_table[sh.get_name() as usize],
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
            data,
            ident,
            sections,
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::{ElfParser, ElfSection};

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

        let parser = ElfParser::parse(&data).unwrap();
        assert_eq!(
            parser.sections,
            vec![
                ElfSection {
                    name: &".shstrtab",
                    typ: 3,
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

        ElfParser::parse(&data)
            .expect_err("ElfParser::parse unexpectedly succeed");
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

        ElfParser::parse(&data)
            .expect_err("ElfParser::parse unexpectedly succeed");
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

        ElfParser::parse(&data)
            .expect_err("ElfParser::parse unexpectedly succeed");
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

        let parser = ElfParser::parse(&data).unwrap();
        assert_eq!(
            parser.sections,
            vec![
                ElfSection {
                    name: &".shstrtab",
                    typ: 3,
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
    fn elf32be_section_header_incomplete() {
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

        ElfParser::parse(&data)
            .expect_err("ElfParser::parse unexpectedly succeed");
    }
}
