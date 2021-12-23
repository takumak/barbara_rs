extern crate clap;
use clap::{Parser, Subcommand};

mod symbol;
mod compress;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ListSymbols {
        #[clap(value_name = "FILENAME")]
        filename: String,
    },
    Huffman {
        #[clap(value_name = "FILENAME")]
        filename: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::ListSymbols { filename }) => {
            use symbol::symbols_from_file;
            for sym in symbols_from_file(filename) {
                println!("{:08x} {}", sym.addr, sym.name);
            }
        }
        Some(Commands::Huffman { filename }) => {
            use symbol::symbols_from_file;
            use compress::make_dic::make_dic;
            use compress::huffman::huffman;

            let syms = symbols_from_file(filename);
            let symnames: Vec<&[u8]> = syms.iter().map(|s| s.name.as_bytes()).collect();
            let dic = make_dic(symnames);
            let hufftbl = huffman(dic);

            for (token, code) in hufftbl {
                let code_s: Vec<u8> = code.iter().map(|c| *c + b'0').collect();
                println!("{} {:4}",
                         std::str::from_utf8(&token).unwrap(),
                         std::str::from_utf8(&code_s).unwrap());
            }
        }
        None => {}
    }
}
