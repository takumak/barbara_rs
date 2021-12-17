use std::process;

extern crate cargo_toml;

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
        "--tests",
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
            let manifest = cargo_toml::Manifest::from_path("Cargo.toml").unwrap();
            let workspace = manifest.workspace.unwrap();
            let test_packages: Vec<String> =
                workspace.members.iter().filter(|&s| s != "xtask")
                .map(|s| s.to_owned()).collect();
            println!("All packages: {:?}", workspace.members);
            println!("Test packages: {:?}", test_packages);

            let mut new_args: Vec<String> = Vec::new();
            for p in test_packages {
                new_args.push("-p".into());
                new_args.push(p.as_str().split('/').last().unwrap().into());
            }
            new_args.extend(args.iter().cloned());
            cargo_testall(&new_args);
        }
        None => {}
    }
}
