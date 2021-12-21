extern crate posix;
use posix::Errno;

extern crate stpack;
use stpack::Unpacker;

use crate::err::ElfParserError;
use crate::ident::{ElfEndian, ElfIdent, ELF_IDENT_SIZE};
use crate::header::ElfHeader;
use crate::section_header::ElfSectionHeader;

pub struct RawSectionParser<H, SH>
where H: Unpacker + ElfHeader,
      SH: Unpacker + ElfSectionHeader,
{
    le: bool,
    pub header: H,
    _sh: [SH; 0],
}

impl<H, SH> RawSectionParser<H, SH>
where H: Unpacker + ElfHeader,
      SH: Unpacker + ElfSectionHeader,
{
    pub fn new(data: &[u8], ident: &ElfIdent) ->
        Result<Self, ElfParserError>
    {
        let le = ident.endian == ElfEndian::ElfLE;
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
        let (sh, _) = SH::unpack(&data[off..], self.le)
            .or(Err(ElfParserError::new(
                Errno::EINVAL,
                format!("Failed to parse elf section header: {}", idx))))?;

        let off = sh.get_offset() as usize;
        let size = sh.get_size() as usize;
        if data.len() < (off + size) {
            return Err(ElfParserError::new(
                Errno::EINVAL,
                format!("Elf section content out of range: \
                         section={:#x}--{:#x}, filesize={:#x}",
                        off, off + size, data.len())))
        }

        Ok((sh, &data[off..(off+size)]))
    }
}
