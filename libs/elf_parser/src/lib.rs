extern crate posix;
extern crate stpack;
use posix::Errno;

mod err;
mod ident;
mod header;
mod section_header;

use err::ElfParserError;
use ident::{ElfClass, ElfEndian, ElfIdent, ELF_IDENT_SIZE};
use header::{Elf32Header, Elf64Header};
use section_header::{Elf32SectionHeader, Elf64SectionHeader};

struct ElfSection<'a> {
    name: &'a String,
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
    string_table: Vec<String>,
}

impl<'a> ElfParser<'a> {
    fn parse(data: &'a [u8]) -> Result<Self, ElfParserError> {
        let ident = ident::parse_ident(data)?;
        if ident.class == ElfClass::Elf32 {
            Self::parse_sections::<Elf32Header, Elf32SectionHeader>(
                &data[ELF_IDENT_SIZE..], ident)
        } else {
            Self::parse_sections::<Elf64Header, Elf64SectionHeader>(
                &data[ELF_IDENT_SIZE..], ident)
        }
    }

    fn parse_sections<H: stpack::Unpacker, SH: stpack::Unpacker>(
        data: &[u8], ident: ElfIdent) -> Result<Self, ElfParserError>
    {
        let le = ident.endian == ElfEndian::ElfLE;
        let (header, _) = H::unpack(&data[ELF_IDENT_SIZE..], le)
            .or(Err(ElfParserError::new(
                Errno::EINVAL, format!("Failed to parse ELF header"))))?;
        let shstr = header.section_header_by_index(header.e_shstrndx)?;
        let string_table: Vec<String> =
            shstr.get_content(data)?
            .split(|c| c == 0)
            .map(|s| String::from_utf8(s))
            .collect();
        let mut sections: Vec<ElfSection> = Vec::new();
        for idx in 0..header.shnum {
            let sh = header.section_header_by_index(header.e_shstrndx)?;
            let start: usize = sh.sh_offset;
            let end: usize = start + sh.sh_size;
            let sec = ElfSection {
                name: string_table[sh.sh_name],
                typ: sh.sh_type,
                flags: sh.sh_flags,
                addr: sh.sh_addr,
                link: sh.sh_link,
                info: sh.sh_info,
                addralign: sh.sh_addralign,
                entsize: sh.sh_entsize,
                content: &data[start..end],
            };
            sections.push(sec);
        }
        Ok(Self{
            data,
            ident,
            sections,
            string_table,
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
