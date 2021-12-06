#!/usr/bin/env python

import sys
import os
import subprocess
import kallsyms

LD_COMMAND = 'rust-lld'
#LD_COMMAND = 'arm-none-eabi-ld'

args = sys.argv[1:]
assert '-o' in args
assert args.index('-o') + 1 < len(args)

o_idx = args.index('-o') + 1
out = args[o_idx]
outdir = os.path.dirname(out)
outname = os.path.basename(out)

tmpouts = []
for i in range(4):
    # see comments in linux/scripts/link_vmlinux.sh
    # to answer "why iterate multiple times" question

    out_tmp = os.path.join(outdir, f'._tmp_{i}_{outname}')
    tmpouts.append(out_tmp)
    args[o_idx] = out_tmp

    if i == 0:
        subprocess.run([LD_COMMAND, *args], check=True)
    else:
        print(f'iteration {i}')
        # generate kallsyms from previous output
        table = kallsyms.symbol_table(tmpouts[-2])
        ldscript = f'{out_tmp}.ld'
        with open(ldscript, 'w') as ldout:
            kallsyms.ldscript(table, ldout)
        subprocess.run([LD_COMMAND, *args, f'-T{ldscript}'], check=True)
        # varidate
        newtable = kallsyms.symbol_table(out_tmp)
        if newtable == table:
            print(f'symbol table matched')
            import shutil
            shutil.copy(out_tmp, out)
            break
else:
    raise RuntimeError('Could not generate kallsyms table')
