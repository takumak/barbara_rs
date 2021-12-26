use kallsyms_tools::{
    ldscript::ldscript,
    symbol::symbols_from_file,
};

const LD_COMMAND: &str = "rust-lld";

fn main() {
    use std::{
        env,
        fs,
        path::Path,
        process,
    };

    let args: Vec<String> =
        env::args().into_iter().skip(1).collect();
    let (o_pos, _) =
        args
        .iter()
        .enumerate()
        .find(|(_i, a)| *a == "-o")
        .expect("-o argument not found");
    assert!(o_pos + 1 < args.len(), "-o argument found at the last");
    let out_path_pos = o_pos + 1;

    let out_path_original = args[out_path_pos].clone();
    let out_path = Path::new(&args[out_path_pos]);
    let out_dir = out_path.parent()
        .expect("Faild to get parent directory of output file");
    let out_fn: String = out_path.file_name()
        .expect("Faild to get output file name")
        .to_str()
        .expect("Faild convert output file name into utf-8")
        .into();

    let mut prev_outfn: String = String::new();
    let mut success = false;
    for i in 0..4 {
        let tmp_fn_bin = format!("._tmp_{}_{}", i, out_fn);
        let mut tmp_path_bin = out_dir.to_path_buf();
        tmp_path_bin.push(tmp_fn_bin.clone());
        let tmp_path_bin = String::from(
            tmp_path_bin.to_str()
                .expect("Failed to convert tmp_path_bin into utf-8"));

        let mut args_tmp = args.clone();
        args_tmp[out_path_pos] = tmp_path_bin.clone();

        if i > 0 {
            println!("iteration {}", i);

            // generate kallsyms from previous output
            let tmp_fn_ld = format!("{}.ld", tmp_fn_bin);
            let mut tmp_path_ld = out_dir.to_path_buf();
            tmp_path_ld.push(tmp_fn_ld);
            let mut file = fs::File::create(tmp_path_ld.as_os_str())
                .expect("Failed to create linker script file");
            ldscript(&prev_outfn, &mut file);
            file.sync_all()
                .expect("Failed to write linker script file");

            let tmp_path_ld = tmp_path_ld.to_str()
                .expect("Failed to convert tmp_path_ld into utf-8");
            args_tmp.push(format!("-T{}", tmp_path_ld));
        }

        let mut linker = process::Command::new(LD_COMMAND);
        linker.args(&args_tmp);
        let status = linker.status()
            .expect("Failed to execute linker process");
        assert!(status.success(), "Failed to execute: {:?}", linker);

        if i > 0 {
            // validate
            let prev_syms = symbols_from_file(&prev_outfn);
            let curr_syms = symbols_from_file(&tmp_path_bin);
            if curr_syms == prev_syms {
                success = true;
                fs::rename(tmp_path_bin, out_path_original)
                    .expect("Failed to rename output file");
                break;
            }
        }

        prev_outfn = tmp_path_bin;
    }

    assert!(success, "Could not generate kallsyms table");
}
