import sys
from elftools.elf.elffile import ELFFile
from rust_demangler import demangle
import io
import struct
from collections import Counter
import re

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

def uncompressed_symbol_table_bin(table):
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

def find_best_token(text):
    chars = Counter(text)
    if '\0' in chars:
        del chars['\0']

    for char, _cnt in chars.most_common():
        token = char
        score = 0

        def enlarge(token, score, right):
            while True:
                if right:
                    pat = re.escape(token) + r'[^\0]'
                else:
                    pat = r'[^\0]' + re.escape(token)
                scores = Counter(re.findall(pat, text))
                if not scores:
                    break
                newtoken, newcount = scores.most_common(1)[0]
                newscore = len(newtoken) * newcount
                if newscore < score:
                    break
                token = newtoken
                score = newscore
            return token, score

        token, score = enlarge(token, score, True)
        token, score = enlarge(token, score, False)

        if len(token) > 1:
            return token

    return None

def makedic(symbols):
    text = '\0'.join(symbols)
    tokens = list(set(text) - set(['\0']))

    def cleanup(tokens, text):
        newtokens = []
        for t in tokens:
            if len(t) > 1 or t in text:
                newtokens.append(t)
        return newtokens

    while len(tokens) < 256 and text != '\0':
        token = find_best_token(text)
        if tokens is None:
            break
        tokens.append(token)
        text = re.sub(f'\\0?{re.escape(token)}\\0?', '\0', text)
        tokens = cleanup(tokens, text)

    # verify
    remains = set(text) - set(tokens)
    if remains != set(['\0']):
        raise RuntimeError('[Bug] Some single chars cannot be represented by tokens')
    if len(tokens) > 256:
        raise RuntimeError('[Bug] More than 256 tokens generated')

    singlechars = [t for t in tokens if len(t) == 1]
    tokens = [t for t in tokens if len(t) > 1] + singlechars
    return tokens

def compress(symbols):
    tok_dic = makedic(symbols)
    # raw_len = sum([len(s) + 1 for s in symbols])

    def tokenize(symbol, tok_dic):
        if symbol == '':
            return []

        for i, token in enumerate(tok_dic):
            if token in symbol:
                tokens = []
                for j, p in enumerate(symbol.split(token)):
                    if j:
                        tokens.append(i)
                    tokens += tokenize(p, tok_dic)
                return tokens

        raise RuntimeError(f'[Bug] Cannot tokenize symbol {repr(symbol)}')

    symbols = [tokenize(s, tok_dic) for s in symbols]
    return tok_dic, symbols

def symbol_table_bin(table):
    #   +--------------+
    #   |    header    |
    #   +--------------+
    #   |  addr table  |
    #   +--------------+
    #   |  name table  |
    #   +--------------+
    #   |  token table |
    #   +--------------+
    #
    #   struct header {
    #     uint32_t reserved;            // 0
    #     uint16_t count;               // addr, name table count
    #     uint16_t addr_table_off;      // offset to addr_table from header top
    #     uint16_t name_table_off;      // offset to name_table from header top
    #     uint16_t token_table_off;     // offset to token_table from header top
    #   };
    #
    #   struct addr_table {
    #     uint32_t addrs[count];
    #   };
    #
    #   struct name_table {
    #     uint16_t name_offs[count];    // byte offset to name_entry from table top
    #   };
    #
    #   struct name_entry {
    #     uint8_t token_count;
    #     uint8_t tokens[token_count];
    #   };
    #
    #   struct token_table {
    #     uint16_t token_offs[256];     // byte offset to token_entry from table top
    #   };
    #
    #   struct token_entry {
    #     uint8_t length;
    #     uint8_t name[];
    #   };

    tok_dic, symbols = compress([s for a, s in table])
    addrs = [a for a, s in table]

    def align_up(align, off):
        if off % align:
            return off + (align - (off % align))
        else:
            return off

    addr_table_size = align_up(4, 4 * len(addrs))
    name_table_size = align_up(4, 2 * len(symbols) + sum([1 + len(s) for s in symbols]))

    addr_table_off = 12
    name_table_off = addr_table_off + addr_table_size
    token_table_off = name_table_off + name_table_size

    def align(align_bytes, buf):
        length = buf.getbuffer().nbytes
        if length % align_bytes:
            buf.write(b'\0' * (align_bytes - (length % align_bytes)))

    buf = io.BytesIO()
    buf.write(struct.pack(
        '<IHHHH',
        0,                      # reserved
        len(table),             # count
        addr_table_off,
        name_table_off,
        token_table_off,
    ))

    for a in addrs:
        buf.write(struct.pack('<I', a))

    off = 2 * len(symbols)
    name_entries = [[len(s), *s] for s in symbols]
    for e in name_entries:
        buf.write(struct.pack('<H', off))
        off += len(e)
    for e in name_entries:
        buf.write(bytes(e))

    align(4, buf)

    off = 2 * len(tok_dic)
    tok_dic_u8 = [t.encode('utf-8') for t in tok_dic]
    token_entries = [bytes([len(t)]) + t for t in tok_dic_u8]
    for t in token_entries:
        buf.write(struct.pack('<H', off))
        off += len(t)
    for t in token_entries:
        buf.write(t)

    bindata = buf.getvalue()
    compressed_size = len(bindata)
    uncompressed_size = len(uncompressed_symbol_table_bin(table))
    compression_ratio = compressed_size/uncompressed_size
    print(f'compression ratio: {compression_ratio}', file=sys.stderr)
    return bindata

def ldscript(table, out):
    bindata = symbol_table_bin(table)
    data_quads = bindata[:len(bindata) - (len(bindata) % 8)]
    data_bytes = bindata[-(len(bindata) % 8):]

    from textwrap import dedent
    print(dedent('''
    SECTIONS {
        .kallsyms : {
            . = __kallsyms_dummy + 12;
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
    ldscript(symbol_table(sys.argv[1]), sys.stdout)
