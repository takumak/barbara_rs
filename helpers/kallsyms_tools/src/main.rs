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
    Dic {
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
        Some(Commands::Dic { filename }) => {
            use symbol::symbols_from_file;
            use compress::make_dic::make_dic;

            let syms: Vec<Vec<u8>> =
                symbols_from_file(filename)
                .iter()
                .map(|s| s.name.as_bytes().to_vec())
                .collect();

            for (token, count) in make_dic(syms.iter().map(|s| s.as_slice()).collect()) {
                println!("{} ({})", std::str::from_utf8(&token).unwrap(), count);
            }
        }
        None => {}
    }
}
