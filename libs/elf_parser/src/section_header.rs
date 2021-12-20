extern crate stpack;
use stpack::unpacker;

// pub trait ElfSectionHeader {
//     fn get_content(&self, data: &[u8]) -> Result<&[u8], ElfParserError> {
//         let start: usize = self.sh_offset;
//         let end: usize = start + self.sh_size;
//         if data.len() < end {
//             Err(ElfParserError::new(
//                 Errno::EINVAL,
//                 format!("Section content not in file range: filesize={:#x} section={:#x}-{:#x}",
//                         data.len(), start, end)));
//         } else {
//             Ok(&data[start..end])
//         }
//     }
// }


unpacker! {
    pub struct Elf32SectionHeader {
        sh_name: u32,
        sh_type: u32,
        sh_flags: u32,
        sh_addr: u32,
        sh_offset: u32,
        sh_size: u32,
        sh_link: u32,
        sh_info: u32,
        sh_addralign: u32,
        sh_entsize: u32,
    }
}

unpacker! {
    pub struct Elf64SectionHeader {
        sh_name: u32,
        sh_type: u32,
        sh_flags: u64,
        sh_addr: u64,
        sh_offset: u64,
        sh_size: u64,
        sh_link: u32,
        sh_info: u32,
        sh_addralign: u64,
        sh_entsize: u64,
    }
}
