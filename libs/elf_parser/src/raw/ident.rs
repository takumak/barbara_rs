extern crate posix;
use posix::Errno;

use crate::ElfParserError;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ElfClass {
    Elf32,
    Elf64,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ElfEndian {
    ElfLE,
    ElfBE,
}

pub const ELF_IDENT_SIZE: usize = 16;

const MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

// offset definition
const OFF_CLASS: usize = 4;
const OFF_DATA: usize = 5;
const OFF_VERSION: usize = 6;
// we ignore ABI fields
// const OFF_OSABI: usize = 7;
// const OFF_ABIVERSION: usize = 8;

const CLASS32: u8 = 1;
const CLASS64: u8 = 2;

const DATA2LSB: u8 = 1;
const DATA2MSB: u8 = 2;

const EV_CURRENT: u8 = 1;

pub fn parse_ident(data: &[u8]) -> Result<(ElfClass, ElfEndian), ElfParserError> {
    if data.len() < ELF_IDENT_SIZE {
        return Err(ElfParserError::new(
            Errno::EINVAL, String::from("File size too small")));
    }

    let ident = <[u8; ELF_IDENT_SIZE]>::try_from(
        &data[..ELF_IDENT_SIZE]).unwrap();

    if ident[..4] != MAGIC {
        return Err(ElfParserError::new(
            Errno::EINVAL, String::from("Magic number mismatch")));
    }

    let class = match ident[OFF_CLASS] {
        CLASS32 => ElfClass::Elf32,
        CLASS64 => ElfClass::Elf64,
        c => return Err(ElfParserError::new(
            Errno::EINVAL, format!("Invalid elf class: {}", c))),
    };

    let endian = match ident[OFF_DATA] {
        DATA2LSB => ElfEndian::ElfLE,
        DATA2MSB => ElfEndian::ElfBE,
        d => return Err(ElfParserError::new(
            Errno::EINVAL, format!("Invalid elf data format: {}", d))),
    };

    if ident[OFF_VERSION] != EV_CURRENT {
        return Err(ElfParserError::new(
            Errno::EINVAL, format!("Unknown elf version: {}", ident[OFF_VERSION])));
    }

    Ok((class, endian))
}

#[cfg(test)]
mod tests {
    use crate::{
        ElfClass,
        ElfEndian,
    };
    use crate::raw::ident::parse_ident;

    #[test]
    fn elfclass_partialeq() {
        assert_eq!(ElfClass::Elf32, ElfClass::Elf32.clone());
        assert_ne!(ElfClass::Elf32, ElfClass::Elf64);
    }

    #[test]
    fn elfclass_debug() {
        assert_eq!(format!("{:?}", ElfClass::Elf32), "Elf32");
    }

    #[test]
    fn elfendian_partialeq() {
        assert_eq!(ElfEndian::ElfLE, ElfEndian::ElfLE.clone());
        assert_ne!(ElfEndian::ElfLE, ElfEndian::ElfBE);
    }

    #[test]
    fn elfendian_debug() {
        assert_eq!(format!("{:?}", ElfEndian::ElfLE), "ElfLE");
    }

    #[test]
    fn incomplete() {
        parse_ident(&[
            0x7fu8, b'E', b'L', b'F',   // magic; should be [0x7f, 'E', 'L', 'F']
            1u8,                        // 1: 32bit, 2: 64bit, others: error
            1u8,                        // 1: Little endian, 2: Big endian, others: error
            1u8,                        // elf version; should be 1
            3u8,                        // OS ABI
            0u8,                        // ABI version
            0u8, 0u8, 0u8,              // padding
            0u8, 0u8, 0u8, // 0u8,         // padding
        ]).expect_err("parse_ident for incomlete data unexpectedly succeed");
    }

    #[test]
    fn invalid_magic() {
        parse_ident(&[
            0u8, 0u8, 0u8, 0u8,         // magic; should be [0x7f, 'E', 'L', 'F']
            1u8,                        // 1: 32bit, 2: 64bit, others: error
            1u8,                        // 1: Little endian, 2: Big endian, others: error
            1u8,                        // elf version; should be 1
            3u8,                        // OS ABI
            0u8,                        // ABI version
            0u8, 0u8, 0u8,              // padding
            0u8, 0u8, 0u8, 0u8,         // padding
        ]).expect_err("parse_ident for invalid magic unexpectedly succeed");
    }

    #[test]
    fn invalid_class() {
        parse_ident(&[
            0x7fu8, b'E', b'L', b'F',   // magic; should be [0x7f, 'E', 'L', 'F']
            0u8, /* error */            // 1: 32bit, 2: 64bit, others: error
            1u8,                        // 1: Little endian, 2: Big endian, others: error
            1u8,                        // elf version; should be 1
            3u8,                        // OS ABI
            0u8,                        // ABI version
            0u8, 0u8, 0u8,              // padding
            0u8, 0u8, 0u8, 0u8,         // padding
        ]).expect_err("parse_ident for invalid magic unexpectedly succeed");
    }

    #[test]
    fn invalid_data() {
        parse_ident(&[
            0x7fu8, b'E', b'L', b'F',   // magic; should be [0x7f, 'E', 'L', 'F']
            1u8,                        // 1: 32bit, 2: 64bit, others: error
            0u8, /* error */            // 1: Little endian, 2: Big endian, others: error
            1u8,                        // elf version; should be 1
            3u8,                        // OS ABI
            0u8,                        // ABI version
            0u8, 0u8, 0u8,              // padding
            0u8, 0u8, 0u8, 0u8,         // padding
        ]).expect_err("parse_ident for invalid magic unexpectedly succeed");
    }

    #[test]
    fn invalid_version() {
        parse_ident(&[
            0x7fu8, b'E', b'L', b'F',   // magic; should be [0x7f, 'E', 'L', 'F']
            1u8,                        // 1: 32bit, 2: 64bit, others: error
            1u8,                        // 1: Little endian, 2: Big endian, others: error
            0u8, /* error */            // elf version; should be 1
            3u8,                        // OS ABI
            0u8,                        // ABI version
            0u8, 0u8, 0u8,              // padding
            0u8, 0u8, 0u8, 0u8,         // padding
        ]).expect_err("parse_ident for invalid magic unexpectedly succeed");
    }

    #[test]
    fn valid_32bit_little() {
        assert_eq!(
            parse_ident(&[
                0x7fu8, b'E', b'L', b'F',   // magic; should be [0x7f, 'E', 'L', 'F']
                1u8,                        // 1: 32bit, 2: 64bit, others: error
                1u8,                        // 1: Little endian, 2: Big endian, others: error
                1u8,                        // elf version; should be 1
                3u8,                        // OS ABI
                0u8,                        // ABI version
                0u8, 0u8, 0u8,              // padding
                0u8, 0u8, 0u8, 0u8,         // padding
            ]),
            Ok((ElfClass::Elf32, ElfEndian::ElfLE))
        );
    }

    #[test]
    fn valid_64bit_little() {
        assert_eq!(
            parse_ident(&[
                0x7fu8, b'E', b'L', b'F',   // magic; should be [0x7f, 'E', 'L', 'F']
                2u8,                        // 1: 32bit, 2: 64bit, others: error
                1u8,                        // 1: Little endian, 2: Big endian, others: error
                1u8,                        // elf version; should be 1
                3u8,                        // OS ABI
                0u8,                        // ABI version
                0u8, 0u8, 0u8,              // padding
                0u8, 0u8, 0u8, 0u8,         // padding
            ]),
            Ok((ElfClass::Elf64, ElfEndian::ElfLE))
        );
    }

    #[test]
    fn valid_32bit_big() {
        assert_eq!(
            parse_ident(&[
                0x7fu8, b'E', b'L', b'F',   // magic; should be [0x7f, 'E', 'L', 'F']
                1u8,                        // 1: 32bit, 2: 64bit, others: error
                2u8,                        // 1: Little endian, 2: Big endian, others: error
                1u8,                        // elf version; should be 1
                3u8,                        // OS ABI
                0u8,                        // ABI version
                0u8, 0u8, 0u8,              // padding
                0u8, 0u8, 0u8, 0u8,         // padding
            ]),
            Ok((ElfClass::Elf32, ElfEndian::ElfBE))
        );
    }
}
