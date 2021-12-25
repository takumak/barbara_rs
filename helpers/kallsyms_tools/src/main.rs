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
        None => {}
    }
}
