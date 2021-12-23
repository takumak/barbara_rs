#[derive(PartialEq)]
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

#[derive(PartialEq)]
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
