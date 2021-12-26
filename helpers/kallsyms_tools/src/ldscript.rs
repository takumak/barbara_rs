use std::io;

extern crate kallsyms_enc;

pub fn ldscript<T>(filename: &String, writer: &mut T)
where
    T: io::Write,
{
    use crate::symbol::symbols_from_file;

    let symbols: Vec<(String, u32)> = symbols_from_file(filename)
        .into_iter()
        .map(|s| (s.name, s.addr))
        .collect();
    let data = kallsyms_enc::pack(&symbols);

    writer.write(b"SECTIONS {\n").unwrap();
    writer.write(b"    .kallsyms : {\n").unwrap();
    writer
        .write(b"        . = __kallsyms_dummy + 12;\n")
        .unwrap();
    writer.write(b"        . = ALIGN(4);\n").unwrap();
    writer.write(b"        __kallsyms = .;\n").unwrap();

    for (i, qbytes) in data.chunks(8).enumerate() {
        if qbytes.len() < 8 {
            writer.write(b"\n").unwrap();
            for b in qbytes {
                let src = format!("BYTE({:#x}); ", b);
                writer.write(src.as_bytes()).unwrap();
            }
        } else {
            let q = u64::from_le_bytes(<[u8; 8]>::try_from(qbytes).unwrap());
            let end = if i % 4 == 3 { "\n" } else { " " };
            let src = format!("QUAD({:#x});{}", q, end);
            writer.write(src.as_bytes()).unwrap();
        }
    }
    writer.write(b"\n").unwrap();
    writer.write(b"    } >ROM\n").unwrap();
    writer.write(b"}\n").unwrap();
}
