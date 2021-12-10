use std::process;

extern crate clap;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[clap(last = true)]
        args: Vec<String>,
    },
    Build {
        #[clap(last = true)]
        args: Vec<String>,
    },
    Testall {
        #[clap(last = true)]
        args: Vec<String>,
    },
}

fn cargo_target(cmd: &str, args: &Vec<String>) {
    let mut args_all = vec![cmd, "--target", "thumbv8m.main-none-eabi"];
    args_all.extend(args.iter().map(|s| &**s));

    let mut cargo = process::Command::new("cargo");
    cargo.args(args_all);
    cargo.env("RUSTFLAGS", "-C link-arg=-Tlink.x -C force-frame-pointers=y");

    let status = cargo.status().unwrap();
    assert!(status.success(), "failed to execute: {:?}", cargo);
}

fn cargo_testall(args: &Vec<String>) {
    let mut args_all = vec![
        "test",
        "-p", "kallsyms",
        "-p", "linked_list_allocator",
    ];
    args_all.extend(args.iter().map(|s| &**s));

    let mut cargo = process::Command::new("cargo");
    cargo.args(args_all);
    // cargo.env("RUST_BACKTRACE", "1");

    let status = cargo.status().unwrap();
    assert!(status.success(), "failed to execute: {:?}", cargo);
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run { args }) => {
            cargo_target("run", args)
        }
        Some(Commands::Build { args }) => {
            cargo_target("build", args)
        }
        Some(Commands::Testall { args }) => {
            cargo_testall(args)
        }
        None => {}
    }
}
