extern crate stpack;
use stpack::unpacker;

use crate::bits_struct;

bits_struct! {
    pub trait ElfSectionHeader { }
    {
        pub struct Elf32SectionHeader;
        pub struct Elf64SectionHeader;
    }
    {
        pub name: {u32, u32,} get_name(u32);
        pub typ: {u32, u32,} get_type(u32);
        pub flags: {u32, u64,} get_flags(u64);
        pub addr: {u32, u64,} get_addr(u64);
        pub offset: {u32, u64,} get_offset(u64);
        pub size: {u32, u64,} get_size(u64);
        pub link: {u32, u32,} get_link(u32);
        pub info: {u32, u32,} get_info(u32);
        pub addralign: {u32, u64,} get_addralign(u64);
        pub entsize: {u32, u64,} get_entsize(u64);
    }
}
