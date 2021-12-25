extern crate stpack;
use stpack::{stpack, Stpack};

use crate::bits_struct;

#[derive(PartialEq, Debug)]
pub enum ElfSectionHeaderType {
    Null,
    Progbits,
    Symtab,
    Strtab,
    Rela,
    Hash,
    Dynamic,
    Note,
    Nobits,
    Rel,
    Shlib,
    Dynsym,
    InitArray,
    FiniArray,
    PreinitArray,
    Group,
    SymtabShndx,
    Num,
    Unknown(u32)
}

bits_struct! {
    pub(crate) trait ElfSectionHeader {
        fn get_type(&self) -> ElfSectionHeaderType {
            match self.get_type_value() {
                0 => ElfSectionHeaderType::Null,
                1 => ElfSectionHeaderType::Progbits,
                2 => ElfSectionHeaderType::Symtab,
                3 => ElfSectionHeaderType::Strtab,
                4 => ElfSectionHeaderType::Rela,
                5 => ElfSectionHeaderType::Hash,
                6 => ElfSectionHeaderType::Dynamic,
                7 => ElfSectionHeaderType::Note,
                8 => ElfSectionHeaderType::Nobits,
                9 => ElfSectionHeaderType::Rel,
                10 => ElfSectionHeaderType::Shlib,
                11 => ElfSectionHeaderType::Dynsym,
                14 => ElfSectionHeaderType::InitArray,
                15 => ElfSectionHeaderType::FiniArray,
                16 => ElfSectionHeaderType::PreinitArray,
                17 => ElfSectionHeaderType::Group,
                18 => ElfSectionHeaderType::SymtabShndx,
                19 => ElfSectionHeaderType::Num,
                t => ElfSectionHeaderType::Unknown(t),
            }
        }
    }
    {
        pub(crate) struct Elf32SectionHeader;
        pub(crate) struct Elf64SectionHeader;
    }
    {
        pub name: {u32, u32,} get_name(u32);
        pub typ: {u32, u32,} get_type_value(u32);
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

#[cfg(test)]
mod tests {
    use crate::raw::section_header::{
        ElfSectionHeader,
        Elf32SectionHeader,
        ElfSectionHeaderType,
    };

    #[test]
    fn sectionheadertype_partialeq() {
        let a = ElfSectionHeaderType::Progbits;
        assert_eq!(a, ElfSectionHeaderType::Progbits);
    }

    #[test]
    fn sectionheadertype_debug() {
        let a = ElfSectionHeaderType::Progbits;
        assert_eq!(format!("{:?}", a), "Progbits");
    }

    #[test]
    fn elfsectionheader_get_type() {
        let mut a = Elf32SectionHeader {
            name: 0,
            typ: 0,
            flags: 0,
            addr: 0,
            offset: 0,
            size: 0,
            link: 0,
            info: 0,
            addralign: 0,
            entsize: 0,
        };
        a.typ = 0;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Null);
        a.typ = 1;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Progbits);
        a.typ = 2;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Symtab);
        a.typ = 3;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Strtab);
        a.typ = 4;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Rela);
        a.typ = 5;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Hash);
        a.typ = 6;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Dynamic);
        a.typ = 7;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Note);
        a.typ = 8;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Nobits);
        a.typ = 9;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Rel);
        a.typ = 10;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Shlib);
        a.typ = 11;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Dynsym);
        a.typ = 14;
        assert_eq!(a.get_type(), ElfSectionHeaderType::InitArray);
        a.typ = 15;
        assert_eq!(a.get_type(), ElfSectionHeaderType::FiniArray);
        a.typ = 16;
        assert_eq!(a.get_type(), ElfSectionHeaderType::PreinitArray);
        a.typ = 17;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Group);
        a.typ = 18;
        assert_eq!(a.get_type(), ElfSectionHeaderType::SymtabShndx);
        a.typ = 19;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Num);
        a.typ = 20;
        assert_eq!(a.get_type(), ElfSectionHeaderType::Unknown(20));
    }
}
