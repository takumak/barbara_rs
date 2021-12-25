extern crate clap;
use clap::{Parser, Subcommand};

extern crate kallsyms;

mod symbol;

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

    Pack {
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

        Some(Commands::Pack { filename }) => {
            use std::io::Write;
            use symbol::symbols_from_file;

            let symbols: Vec<(String, u32)> =
                symbols_from_file(filename)
                .into_iter()
                .map(|s| (s.name, s.addr))
                .collect();
            let data = kallsyms::pack(&symbols);
            std::io::stdout().write(&data).unwrap();

            let plain_len =
                (symbols.len() * 6) +
                symbols.iter()
                .map(|(name, _addr)| name.as_bytes().len())
                .reduce(|acc, len| acc + len)
                .unwrap();
            eprintln!("compression ratio: {:.2}",
                      data.len() as f64 / plain_len as f64);
        }

        None => {}
    }
}
