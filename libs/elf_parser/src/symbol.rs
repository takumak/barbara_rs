#[derive(PartialEq, Debug)]
pub enum ElfSymbolType {
    Notype,
    Object,
    Func,
    Section,
    File,
    Common,
    Tls,
    Num,
    Loos,
    GnuIfunc,
    Hios,
    Loproc,
    Hiproc,
    Unknown(u8),
}

#[derive(PartialEq, Debug)]
pub enum ElfSymbolBind {
    Local,
    Global,
    Weak,
    Num,
    Loos,
    GnuUnique,
    Hios,
    Loproc,
    Hiproc,
    Unknown(u8),
}

#[derive(PartialEq, Debug)]
pub struct Symbol<'a> {
    pub name: &'a [u8],
    pub value: u64,
    pub size: u64,
    pub info: u8,
    pub other: u8,
    pub shndx: u16,
}

impl<'a> Symbol<'a> {
    pub fn get_type(&self) -> ElfSymbolType {
        match self.info & 0xf {
            0  => ElfSymbolType::Notype,
            1  => ElfSymbolType::Object,
            2  => ElfSymbolType::Func,
            3  => ElfSymbolType::Section,
            4  => ElfSymbolType::File,
            5  => ElfSymbolType::Common,
            6  => ElfSymbolType::Tls,
            7  => ElfSymbolType::Num,
            10 => ElfSymbolType::GnuIfunc,
            12 => ElfSymbolType::Hios,
            13 => ElfSymbolType::Loproc,
            15 => ElfSymbolType::Hiproc,
            t  => ElfSymbolType::Unknown(t),
        }
    }

    pub fn get_bind(&self) -> ElfSymbolBind {
        match (self.info >> 4) & 0xf {
            0  => ElfSymbolBind::Local,
            1  => ElfSymbolBind::Global,
            2  => ElfSymbolBind::Weak,
            3  => ElfSymbolBind::Num,
            10 => ElfSymbolBind::GnuUnique,
            12 => ElfSymbolBind::Hios,
            13 => ElfSymbolBind::Loproc,
            15 => ElfSymbolBind::Hiproc,
            b  => ElfSymbolBind::Unknown(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::symbol::{
        ElfSymbolType,
        ElfSymbolBind,
        Symbol,
    };

    #[test]
    fn symboltype_partialeq() {
        let a = ElfSymbolType::Object;
        assert_eq!(a, ElfSymbolType::Object);
    }

    #[test]
    fn symboltype_debug() {
        let a = ElfSymbolType::Object;
        assert_eq!(format!("{:?}", a), "Object");
    }

    #[test]
    fn symbolbind_partialeq() {
        let a = ElfSymbolBind::Global;
        assert_eq!(a, ElfSymbolBind::Global);
    }

    #[test]
    fn symbolbind_debug() {
        let a = ElfSymbolBind::Global;
        assert_eq!(format!("{:?}", a), "Global");
    }

    #[test]
    fn symbol_partialeq() {
        let a = Symbol {
            name: &[],
            value: 0,
            size: 0,
            info: 0,
            other: 0,
            shndx: 0,
        };

        let b = Symbol {
            name: &[],
            value: 0,
            size: 0,
            info: 0,
            other: 0,
            shndx: 0,
        };

        assert_eq!(a, b);
    }

    #[test]
    fn symbol_debug() {
        let a = Symbol {
            name: &[],
            value: 1,
            size: 2,
            info: 3,
            other: 4,
            shndx: 5,
        };
        assert_eq!(
            format!("{:?}", a),
            "Symbol { \
             name: [], \
             value: 1, \
             size: 2, \
             info: 3, \
             other: 4, \
             shndx: 5 \
             }"
        );
    }

    #[test]
    fn symbol_get_type_get_bind() {
        let mut a = Symbol {
            name: &[],
            value: 1,
            size: 2,
            info: 3,
            other: 4,
            shndx: 5,
        };
        a.info = 0x30;
        assert_eq!(a.get_type(), ElfSymbolType::Notype);
        assert_eq!(a.get_bind(), ElfSymbolBind::Num);
        a.info = 0x21;
        assert_eq!(a.get_type(), ElfSymbolType::Object);
        assert_eq!(a.get_bind(), ElfSymbolBind::Weak);
        a.info = 0x12;
        assert_eq!(a.get_type(), ElfSymbolType::Func);
        assert_eq!(a.get_bind(), ElfSymbolBind::Global);
        a.info = 0x03;
        assert_eq!(a.get_type(), ElfSymbolType::Section);
        assert_eq!(a.get_bind(), ElfSymbolBind::Local);
        a.info = 0x04;
        assert_eq!(a.get_type(), ElfSymbolType::File);
        a.info = 0x05;
        assert_eq!(a.get_type(), ElfSymbolType::Common);
        a.info = 0x06;
        assert_eq!(a.get_type(), ElfSymbolType::Tls);
        a.info = 0x07;
        assert_eq!(a.get_type(), ElfSymbolType::Num);
        a.info = 0xfa;
        assert_eq!(a.get_type(), ElfSymbolType::GnuIfunc);
        assert_eq!(a.get_bind(), ElfSymbolBind::Hiproc);
        a.info = 0xdc;
        assert_eq!(a.get_type(), ElfSymbolType::Hios);
        assert_eq!(a.get_bind(), ElfSymbolBind::Loproc);
        a.info = 0xcd;
        assert_eq!(a.get_type(), ElfSymbolType::Loproc);
        assert_eq!(a.get_bind(), ElfSymbolBind::Hios);
        a.info = 0xaf;
        assert_eq!(a.get_type(), ElfSymbolType::Hiproc);
        assert_eq!(a.get_bind(), ElfSymbolBind::GnuUnique);

        a.info = 0x98;
        assert_eq!(a.get_type(), ElfSymbolType::Unknown(8));
        assert_eq!(a.get_bind(), ElfSymbolBind::Unknown(9));
    }
}
