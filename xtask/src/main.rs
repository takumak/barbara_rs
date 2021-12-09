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
    Run { },
    Build { },
}

fn cargo(cmd: &str) {
    let mut cargo = process::Command::new("cargo");
    cargo.args(&[cmd, "--target", "thumbv8m.main-none-eabi"]);
    cargo.env("RUSTFLAGS", "-C link-arg=-Tlink.x -C force-frame-pointers=y");

    let status = cargo.status().unwrap();
    assert!(status.success(), "failed to execute: {:?}", cargo);
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run { }) => {
            cargo("run")
        }
        Some(Commands::Build { }) => {
            cargo("build")
        }
        None => {}
    }
}
