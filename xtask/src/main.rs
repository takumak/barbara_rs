use std::{env, fs, path, process};
use std::os::unix::fs::PermissionsExt;

extern crate serde;
extern crate serde_json;

extern crate prettytable;

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

#[derive(serde::Deserialize, Debug)]
struct Coverage {
    data: Vec<CoverageData>,
}

#[derive(serde::Deserialize, Debug)]
struct CoverageData {
    files: Vec<CoverageFile>,
}

#[derive(serde::Deserialize, Debug)]
struct CoverageFile {
    filename: String,
    summary: CoverageSummary,
}

#[derive(serde::Deserialize, Debug)]
struct CoverageSummary {
    regions: CoverageSummaryItem,
    functions: CoverageSummaryItem,
    lines: CoverageSummaryItem,
}

#[derive(serde::Deserialize, Debug)]
struct CoverageSummaryItem {
    count: u32,
    covered: u32,
    percent: f64,
}

fn cargo_testall(args: &Vec<String>) {

    /* Remove previously generated coverage data */

    let cov_dir = env::current_dir().unwrap().join("cov");
    if !cov_dir.exists() {
        fs::create_dir(&cov_dir).unwrap();
    }

    let profraw_files =
        |cov_dir: &path::PathBuf| cov_dir.read_dir().unwrap()
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

    // generate coverage report HTML

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
        "--output-dir=.",
    ].iter().map(|&s| s.into()).collect();
    for (_, prog) in test_progs.iter() {
        args.push("--object".into());
        args.push(prog.clone());
    }

    let mut cargo = process::Command::new("cargo");
    cargo
        .current_dir(&cov_dir)
        .args(args);

    println!("Run: {:?}", cargo);

    let status = cargo
        .status()
        .expect("failed to execute cargo process");

    assert!(status.success(), "failed to execute: {:?}", cargo);

    // show coverage summary

    let mut args: Vec<String> = vec![
        "cov", "--",
        "export",
        "--format=text",
        "--ignore-filename-regex=/.cargo/registry",
        "--ignore-filename-regex=/library/std/",
        "--instr-profile=json5format.profdata",
    ].iter().map(|&s| s.into()).collect();
    for (_, prog) in test_progs.iter() {
        args.push("--object".into());
        args.push(prog.clone());
    }

    let mut cargo = process::Command::new("cargo");
    cargo
        .current_dir(&cov_dir)
        .args(args)
        .stdout(process::Stdio::piped());

    println!("Run: {:?}", cargo);

    let output = cargo
        .output()
        .expect("failed to spawn cargo command");

    let cov: Coverage = serde_json::from_slice(
        output.stdout.as_slice()).unwrap();

    use prettytable::format::{
        FormatBuilder,
        LinePosition,
        LineSeparator,
    };
    let mut table = prettytable::Table::new();
    table.set_format(
        FormatBuilder::new()
            .separator(LinePosition::Title,  LineSeparator::new('-', '+', '+', '+'))
            .separator(LinePosition::Bottom, LineSeparator::new('-', '+', '+', '+'))
            .separator(LinePosition::Top,    LineSeparator::new('-', '+', '+', '+'))
            .padding(2, 2)
            .build());
    table.set_titles(prettytable::Row::new(vec![
        prettytable::Cell::new("Filename"),
        prettytable::Cell::new("Regions"),
        prettytable::Cell::new("Functions"),
        prettytable::Cell::new("Lines"),
    ]));
    let rootdir = format!("{}/", env::current_dir().unwrap().to_str().unwrap());
    for data in cov.data {
        for file in data.files {
            let mut filename = file.filename;
            if filename.starts_with(&rootdir) {
                filename = String::from(
                    filename.strip_prefix(&rootdir).unwrap());
            }

            use prettytable::{Row, Cell, color, Attr};
            let mut row = Row::empty();
            row.add_cell(Cell::new(filename.as_str()));

            for col in [file.summary.regions,
                        file.summary.functions,
                        file.summary.lines] {
                let text =
                    format!("{:.2} ({}/{})",
                            col.percent,
                            col.covered,
                            col.count);

                let c =
                    if col.covered == col.count {
                        color::GREEN
                    } else if col.percent >= 80. {
                        color::YELLOW
                    } else {
                        color::RED
                    };

                row.add_cell(
                    Cell::new(text.as_str())
                        .with_style(
                            Attr::ForegroundColor(c)));
            }

            table.add_row(row);
        }
    }
    table.printstd();

    // fix permissions

    fn fix_permission(p: &path::Path) {
        let _ = fs::set_permissions(
            p,
            PermissionsExt::from_mode(
                if p.is_dir() {
                    0o755
                } else {
                    0o644
                }
            ));

        if p.is_dir() {
            let r = fs::read_dir(p);
            if r.is_ok() {
                for e in r.unwrap() {
                    if e.is_ok() {
                        fix_permission(&e.unwrap().path().as_path());
                    }
                }
            }
        }
    }

    fix_permission(cov_dir.as_path());
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
