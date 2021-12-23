#[derive(PartialEq, Debug)]
pub enum SymbolType {
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
pub enum SymbolBind {
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
    pub fn get_type(&self) -> SymbolType {
        match self.info & 0xf {
            0  => SymbolType::Notype,
            1  => SymbolType::Object,
            2  => SymbolType::Func,
            3  => SymbolType::Section,
            4  => SymbolType::File,
            5  => SymbolType::Common,
            6  => SymbolType::Tls,
            7  => SymbolType::Num,
            10 => SymbolType::GnuIfunc,
            12 => SymbolType::Hios,
            13 => SymbolType::Loproc,
            15 => SymbolType::Hiproc,
            t  => SymbolType::Unknown(t),
        }
    }

    pub fn get_bind(&self) -> SymbolBind {
        match (self.info >> 4) & 0xf {
            0  => SymbolBind::Local,
            1  => SymbolBind::Global,
            2  => SymbolBind::Weak,
            3  => SymbolBind::Num,
            10 => SymbolBind::GnuUnique,
            12 => SymbolBind::Hios,
            13 => SymbolBind::Loproc,
            15 => SymbolBind::Hiproc,
            b  => SymbolBind::Unknown(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::symbol::{
        SymbolType,
        SymbolBind,
        Symbol,
    };

    #[test]
    fn symboltype_partialeq() {
        let a = SymbolType::Object;
        assert_eq!(a, SymbolType::Object);
    }

    #[test]
    fn symboltype_debug() {
        let a = SymbolType::Object;
        assert_eq!(format!("{:?}", a), "Object");
    }

    #[test]
    fn symbolbind_partialeq() {
        let a = SymbolBind::Global;
        assert_eq!(a, SymbolBind::Global);
    }

    #[test]
    fn symbolbind_debug() {
        let a = SymbolBind::Global;
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
        assert_eq!(a.get_type(), SymbolType::Notype);
        assert_eq!(a.get_bind(), SymbolBind::Num);
        a.info = 0x21;
        assert_eq!(a.get_type(), SymbolType::Object);
        assert_eq!(a.get_bind(), SymbolBind::Weak);
        a.info = 0x12;
        assert_eq!(a.get_type(), SymbolType::Func);
        assert_eq!(a.get_bind(), SymbolBind::Global);
        a.info = 0x03;
        assert_eq!(a.get_type(), SymbolType::Section);
        assert_eq!(a.get_bind(), SymbolBind::Local);
        a.info = 0x04;
        assert_eq!(a.get_type(), SymbolType::File);
        a.info = 0x05;
        assert_eq!(a.get_type(), SymbolType::Common);
        a.info = 0x06;
        assert_eq!(a.get_type(), SymbolType::Tls);
        a.info = 0x07;
        assert_eq!(a.get_type(), SymbolType::Num);
        a.info = 0xfa;
        assert_eq!(a.get_type(), SymbolType::GnuIfunc);
        assert_eq!(a.get_bind(), SymbolBind::Hiproc);
        a.info = 0xdc;
        assert_eq!(a.get_type(), SymbolType::Hios);
        assert_eq!(a.get_bind(), SymbolBind::Loproc);
        a.info = 0xcd;
        assert_eq!(a.get_type(), SymbolType::Loproc);
        assert_eq!(a.get_bind(), SymbolBind::Hios);
        a.info = 0xaf;
        assert_eq!(a.get_type(), SymbolType::Hiproc);
        assert_eq!(a.get_bind(), SymbolBind::GnuUnique);

        a.info = 0x98;
        assert_eq!(a.get_type(), SymbolType::Unknown(8));
        assert_eq!(a.get_bind(), SymbolBind::Unknown(9));
    }
}
