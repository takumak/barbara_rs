extern crate stpack;
use stpack::Unpacker;

mod err;
mod ident;
mod header;
mod section_header;
mod raw_section_parser;

use err::ElfParserError;
use ident::{ElfClass, ElfIdent, ELF_IDENT_SIZE};
use header::{ElfHeader, Elf32Header, Elf64Header};
use section_header::{ElfSectionHeader, Elf32SectionHeader, Elf64SectionHeader};
use raw_section_parser::RawSectionParser;

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
    #[test]
    fn foo_size() {
        // assert_eq!(Foo::SIZE, 7);
    }
}
