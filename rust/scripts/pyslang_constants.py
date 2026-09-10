#!/usr/bin/env python3
"""Constant-value oracle for the sv-lang differential test.

Reads a SystemVerilog file, elaborates it with pyslang, and prints JSON mapping
each top-level parameter/localparam name to its integer constant value:

    {"X": {"i": 200, "width": 8, "signed": false, "unknown": false}, ...}

`i` is null when the value has unknown (x/z) bits. Non-integer params are
omitted. Requires pyslang on PYTHONPATH (build with SLANG_INCLUDE_PYLIB=ON).
"""
import json
import sys

import pyslang as ps


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: pyslang_constants.py <file.sv>", file=sys.stderr)
        return 2

    with open(sys.argv[1]) as f:
        text = f.read()

    tree = ps.syntax.SyntaxTree.fromText(text)
    comp = ps.ast.Compilation()
    comp.addSyntaxTree(tree)

    out = {}
    for top in comp.getRoot().topInstances:
        for m in top.body:
            name = getattr(m, "name", "")
            val = getattr(m, "value", None)
            if not name or val is None:
                continue
            iv = getattr(val, "value", None)
            if not isinstance(iv, ps.SVInt):
                continue
            unknown = bool(iv.hasUnknown)
            out[name] = {
                "i": None if unknown else int(iv),
                "width": int(iv.bitWidth),
                "signed": bool(iv.isSigned),
                "unknown": unknown,
            }

    json.dump(out, sys.stdout)
    return 0


if __name__ == "__main__":
    sys.exit(main())
