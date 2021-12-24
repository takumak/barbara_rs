extern crate posix;
use posix::Errno;

extern crate stpack;
use stpack::Unpacker;

use crate::{
    ElfEndian,
    ElfParserError,
    ElfSectionHeaderType,
};

use crate::raw::{
    ident::ELF_IDENT_SIZE,
    header::ElfHeader,
    section_header::ElfSectionHeader,
};

pub(crate) struct SectionParser<H, SH>
where H: Unpacker + ElfHeader,
      SH: Unpacker + ElfSectionHeader,
{
    le: bool,
    pub header: H,
    _sh: [SH; 0],
}

impl<H, SH> SectionParser<H, SH>
where H: Unpacker + ElfHeader,
      SH: Unpacker + ElfSectionHeader,
{
    pub fn new(data: &[u8], endian: ElfEndian) ->
        Result<Self, ElfParserError>
    {
        let le = endian == ElfEndian::ElfLE;
        let (header, _) = H::unpack(&data[ELF_IDENT_SIZE..], le)
            .or(Err(ElfParserError::new(
                Errno::EINVAL, format!("Failed to parse ELF header"))))?;

        Ok(Self {
            le,
            header,
            _sh: [],
        })
    }

    pub fn nth<'a>(&self, data: &'a[u8], idx: usize) ->
        Result<(SH, &'a [u8]), ElfParserError>
    {
        if idx >= (self.header.get_shnum() as usize) {
            return Err(ElfParserError::new(
                Errno::EINVAL,
                format!("Section index out of range: \
                         {} (must be less than {})",
                        idx, self.header.get_shnum())))
        }

        let size = self.header.get_shentsize() as usize;
        let off = self.header.get_shoff() as usize + (size * idx);
        if data.len() < off+size {
            return Err(ElfParserError::new(
                Errno::EINVAL,
                format!("Elf section header out of range: \
                         section={:#x}--{:#x}, filesize={:#x}",
                        off, off + size, data.len())))
        }
        let (sh, _) = SH::unpack(&data[off..(off+size)], self.le)
            .or(Err(ElfParserError::new(
                Errno::EINVAL,
                format!("Failed to parse elf section header: {}", idx))))?;

        let content =
            if sh.get_type() != ElfSectionHeaderType::Nobits {
                let off = sh.get_offset() as usize;
                let size = sh.get_size() as usize;
                if data.len() < (off + size) {
                    return Err(ElfParserError::new(
                        Errno::EINVAL,
                        format!("Elf section content out of range: \
                                 section={:#x}--{:#x}, filesize={:#x}",
                                off, off + size, data.len())))
                }
                &data[off..(off+size)]
            } else {
                &[]
            };
        Ok((sh, content))
    }
}


#[cfg(test)]
mod tests {
    use crate::ElfEndian;
    use crate::raw::{
        header::{
            Elf32Header,
            Elf64Header,
        },
        section_header::{
            Elf32SectionHeader,
            Elf64SectionHeader,
        },
        section_parser::SectionParser,
    };

    #[test]
    fn elf32_incomplete_header() {
        type Parser = SectionParser::<Elf32Header, Elf32SectionHeader>;
        let parser = Parser::new(
            &[0; 17],
            ElfEndian::ElfBE
        );
        assert!(parser.is_err());
    }

    #[test]
    fn elf64_incomplete_header() {
        type Parser = SectionParser::<Elf64Header, Elf64SectionHeader>;
        let parser = Parser::new(
            &[0; 17],
            ElfEndian::ElfBE
        );
        assert!(parser.is_err());
    }
}
