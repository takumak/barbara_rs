import sys
from elftools.elf.elffile import ELFFile
from rust_demangler import demangle
import io
import struct

magic = 0xea805138

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
            yield addr, name

def symbol_table(filename):
    #   +00  +---------+
    #        |  magic  |                   4 bytes
    #   +04  +---------+
    #        |  count  |                   4 bytes
    #   +08  +---------+---------+
    #        |   addr  |   off   |         4+4 bytes
    #   +10  +---------+---------+
    #        |   addr  |   off   |         4+4 bytes
    #   +18  +---------+---------+
    #        |   addr  |   off   |         4+4 bytes
    #   +20  +---------+---------+
    #       ... `count` entries ...
    #        +-------------------+
    #        |  null separated   |
    #        |    name vector    |
    #        +-------------------+

    symbols = list(sorted(iter_symbols(filename), key=lambda p: p[0]))
    addr_off_table = []
    name_vec_off = 8 + (8 * len(symbols))
    names_buf = io.BytesIO()

    for i, (addr, name) in enumerate(symbols):
        name = name.encode('utf-8')
        if len(name) > 255:
            name = name[:255]

        name_off = 8 * (len(symbols) - i)
        name_off += names_buf.getbuffer().nbytes

        addr_off_table.append((addr, name_off))
        names_buf.write(name)
        names_buf.write(b'\0')

    table_buf = io.BytesIO()
    table_buf.write(struct.pack('<II', magic, len(symbols)))
    for addr, off in addr_off_table:
        table_buf.write(struct.pack('<II', addr, off))
    table_buf.write(names_buf.getvalue())

    return table_buf.getvalue()

if __name__ == '__main__':
    for addr, name in sorted(iter_symbols(sys.argv[1]), key=lambda p: p[0]):
        print(f'{addr:08x} {name}')
