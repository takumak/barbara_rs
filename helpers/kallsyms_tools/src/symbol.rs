extern crate elf_parser;
use elf_parser::{
    ElfParser,
    ElfSymbol,
    ElfSymbolType,
};

extern crate rustc_demangle;
use rustc_demangle::demangle;

pub struct Symbol {
    pub addr: u32,
    pub name: String,
}

impl<'a> From<ElfSymbol<'a>> for Symbol {
    fn from(sym: ElfSymbol) -> Self {
        let name: String = match std::str::from_utf8(sym.name) {
            Ok(name) => format!("{:#}", demangle(name)),
            Err(_) => format!("{:?}", sym.name),
        };
        Self {
            addr: sym.value as u32,
            name,
        }
    }
}

pub fn symbols_from_file(filename: &str) -> Vec<Symbol> {
    let data = std::fs::read(filename).unwrap();
    let parser = ElfParser::from_bytes(&data).unwrap();
    let mut syms = Vec::new();
    for s in parser.iter_symbols() {
        let s = s.expect("Elf parse error");
        if s.get_type() == ElfSymbolType::Func {
            syms.push(Symbol::from(s));
        }
    }
    syms.sort_by(|a, b| a.addr.cmp(&b.addr));
    syms
}
