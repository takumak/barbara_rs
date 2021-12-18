use std::{env, fs, process};

extern crate serde;
extern crate serde_json;

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
    cargo
        .args(args_all)
        .env("RUSTFLAGS", "-C link-arg=-Tlink.x -C force-frame-pointers=y");

    let status = cargo
        .status()
        .expect("failed to execute cargo process");

    assert!(status.success(), "failed to execute: {:?}", cargo);
}

#[derive(serde::Deserialize, Debug)]
struct CompilerMessage {
    reason: String,
    executable: Option<String>,
    target: Option<Target>,
}

#[derive(serde::Deserialize, Debug)]
struct Target {
    name: String,
}

fn cargo_testall(args: &Vec<String>) {

    /* Remove previously generated coverage data */

    let cov_dir = env::current_dir().unwrap().join("cov");
    if !cov_dir.exists() {
        fs::create_dir(&cov_dir).unwrap();
    }

    let profraw_files =
        |cov_dir: &std::path::PathBuf| cov_dir.read_dir().unwrap()
        .filter(|e| e.is_ok())
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_str().unwrap_or("").ends_with(".profraw"));

    for p in profraw_files(&cov_dir) {
        fs::remove_file(p).unwrap();
    }

    let profdata = cov_dir.join("json5format.profdata");
    if profdata.exists() {
        fs::remove_file(profdata).unwrap();
    }

    /* Build test programs */

    let mut args_all = vec![
        "test",
        "--workspace",
        "--lib",
        "--no-run",
        "--message-format", "json",
    ];
    args_all.extend(args.iter().map(|s| &**s));

    let mut cargo = process::Command::new("cargo");
    let mut child = cargo
        .args(args_all)
        .env("RUSTFLAGS", "-Zinstrument-coverage")
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to spawn cargo command");

    let reader = child.stdout.take().unwrap();
    let deserializer = serde_json::Deserializer::from_reader(reader);
    let mut test_progs: Vec<(String, String)> = Vec::new();

    for msg in deserializer.into_iter::<CompilerMessage>() {
        let msg = msg.unwrap();
        if msg.reason == "compiler-artifact" && msg.executable.is_some() {
            let name =
                match msg.target {
                    Some(t) => t.name,
                    None => String::from("????"),
                };
            test_progs.push((
                name,
                msg.executable.unwrap(),
            ));
        }
    }

    let status = child
        .wait()
        .expect("wait failed");

    assert!(status.success(), "failed to execute: {:?}", cargo);

    /* Run test programs */

    for (name, prog) in test_progs.iter() {
        let mut cmd = process::Command::new(prog);

        println!("**** {} ****", name);

        cmd.env("LLVM_PROFILE_FILE",
                format!("{}/json5format-%m.profraw",
                        cov_dir.to_str().unwrap_or(".")));

        let status = cmd
            .status()
            .expect("failed to execute cargo process");

        assert!(status.success(), "failed to execute: {:?}", cargo);
    }

    // merge coverage report

    let mut args: Vec<String> = vec![
        "profdata", "--",
        "merge",
        "--sparse",
        "-o",
        "json5format.profdata",
    ].iter().map(|&s| s.into()).collect();
    for p in profraw_files(&cov_dir) {
        args.push(p.to_str().unwrap().to_owned());
    }
    let mut cargo = process::Command::new("cargo");
    cargo
        .current_dir(&cov_dir)
        .args(args);

    let status = cargo
        .status()
        .expect("failed to execute cargo process");

    assert!(status.success(), "failed to execute: {:?}", cargo);

    // generate coverage report

    let mut args: Vec<String> = vec![
        "cov", "--",
        "show",
        "--ignore-filename-regex=/.cargo/registry",
        "--ignore-filename-regex=/library/std/",
        "--instr-profile=json5format.profdata",
        "--show-instantiations",
        "--show-line-counts-or-regions",
        "--Xdemangler=rustfilt",
        "--format=html",
    ].iter().map(|&s| s.into()).collect();
    for (_, prog) in test_progs.iter() {
        args.push("--object".into());
        args.push(prog.clone());
    }

    let mut cargo = process::Command::new("cargo");
    cargo
        .current_dir(&cov_dir)
        .stdout(fs::File::create(cov_dir.join("coverage.html")).unwrap())
        .args(args);

    println!("Run: {:?}", cargo);

    let status = cargo
        .status()
        .expect("failed to execute cargo process");

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
            cargo_testall(args);
        }
        None => {}
    }
}
