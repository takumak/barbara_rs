extern crate stpack;
use stpack::unpacker;

// pub trait ElfHeader {
//     type ElfSectionHeader;

//     fn section_header_by_index(&self, data: &[u8], idx: usize, le: bool) ->
//         Result<Self::ElfSectionHeader, ElfParserError>
//     {
//         if idx >= (self.shnum as usize) {
//             return Err(ElfParserError::new(
//                 Errno::EINVAL,
//                 format!("Section index out of range: {} (should less than {})", idx, self.shnum)));
//         }
//         let size = self.e_shentsize as usize;
//         let off = self.e_shoff as usize + (size * idx);
//         Ok(Self::ElfSectionHeader::unpack(data[off..(off + size)], le))
//     }
// }

unpacker! {
    pub struct Elf32Header {
        e_type:      u16,
        e_machine:   u16,
        e_version:   u32,
        e_entry:     u32,
        e_phoff:     u32,
        e_shoff:     u32,
        e_flags:     u32,
        e_ehsize:    u16,
        e_phentsize: u16,
        e_phnum:     u16,
        e_shentsize: u16,
        e_shnum:     u16,
        e_shstrndx:  u16,
    }
}

unpacker! {
    pub struct Elf64Header {
        e_type:      u16,
        e_machine:   u16,
        e_version:   u32,
        e_entry:     u64,
        e_phoff:     u64,
        e_shoff:     u64,
        e_flags:     u32,
        e_ehsize:    u16,
        e_phentsize: u16,
        e_phnum:     u16,
        e_shentsize: u16,
        e_shnum:     u16,
        e_shstrndx:  u16,
    }
}
