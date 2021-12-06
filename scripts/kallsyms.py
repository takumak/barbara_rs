import sys
from elftools.elf.elffile import ELFFile
from rust_demangler import demangle
import io
import struct

def iter_symbols(filename):
    with open(filename, 'rb') as f:
        elf = ELFFile(f)
        symtab = elf.get_section_by_name('.symtab')
        for sym in symtab.iter_symbols():
            if sym.name.startswith('$'):
                continue

            try:
                name = demangle(sym.name)
            except:
                name = sym.name

            addr = sym['st_value']
            if addr == 0:
                continue

            yield addr, name

def symbol_table(filename):
    return list(sorted(iter_symbols(filename), key=lambda p: p[0]))

def symbol_table_bin(table):
    #   +00  +---------+
    #        |  count  |                   4 bytes
    #   +04  +---------+---------+
    #        |   addr  |   off   |         4+4 bytes
    #   +0c  +---------+---------+
    #        |   addr  |   off   |         4+4 bytes
    #   +14  +---------+---------+
    #        |   addr  |   off   |         4+4 bytes
    #        +---------+---------+
    #       ... `count` entries ...
    #        +-------------------+
    #        |  null separated   |
    #        |    name vector    |
    #        +-------------------+

    header_size = 4
    entry_size = 8

    addr_off_table = []
    name_vec_off = header_size + (entry_size * len(table))
    names_buf = io.BytesIO()

    for i, (addr, name) in enumerate(table):
        name = name.encode('utf-8')
        name_off = entry_size * (len(table) - i)
        name_off += names_buf.getbuffer().nbytes

        addr_off_table.append((addr, name_off))
        names_buf.write(name)
        names_buf.write(b'\0')

    table_buf = io.BytesIO()
    table_buf.write(struct.pack('<I', len(table)))
    for addr, off in addr_off_table:
        table_buf.write(struct.pack('<II', addr, off))
    table_buf.write(names_buf.getvalue())

    return table_buf.getvalue()

def ldscript(table, out):
    bindata = symbol_table_bin(table)
    data_quads = bindata[:len(bindata) - (len(bindata) % 8)]
    data_bytes = bindata[-(len(bindata) % 8):]

    from textwrap import dedent
    print(dedent('''
    SECTIONS {
        .kallsyms : {
            . = __kallsyms_dummy + 4;
            . = ALIGN(4);
            __kallsyms = .;
    '''), file=out)

    for i, (q,) in enumerate(struct.iter_unpack('<Q', data_quads)):
        print(f'QUAD({q:#x});', file=out, end=['\n', ''][int(bool(i % 4))])
    print(file=out)
    for i, (b,) in enumerate(struct.iter_unpack('<B', data_bytes)):
        print(f'BYTE({b:#x});', file=out, end='')
    print(file=out)

    print(dedent('''
        } >ROM
    }
    '''), file=out)

    return bindata

if __name__ == '__main__':
    # for addr, name in sorted(iter_symbols(sys.argv[1]), key=lambda p: p[0]):
    #     print(f'{addr:08x} {name}')
    symbol_table_ld(sys.argv[1], sys.stdout)
