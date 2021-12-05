#!/usr/bin/env python

import sys
import os
import subprocess
import kallsyms

args = sys.argv[1:]
assert '-o' in args
assert args.index('-o') + 1 < len(args)

idx = args.index('-o') + 1
out = args[idx]
outdir = os.path.dirname(out)
outname = os.path.basename(out)
out_tmp = os.path.join(outdir, '._tmp_%s' % outname)
args[idx] = out_tmp

subprocess.run(['rust-lld', *args], check=True)
filename = os.path.join(outdir, 'kallsyms.bin')
with open(filename, 'wb') as f:
    f.write(kallsyms.symbol_table(out_tmp))
subprocess.run(
    [
        'arm-none-eabi-objcopy',
        '--update-section', '.kallsyms=%s' % filename,
        out_tmp,
        out,
    ],
    check=True
)
