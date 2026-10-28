"""Print the variant names of named Rust enums.

usage: enum_variants.py <src_dir> EnumName [EnumName ...]
"""
import glob
import os
import re
import sys

src, names = sys.argv[1], sys.argv[2:]
STR = re.compile(r'"(?:[^"\\]|\\.)*"')
for path in glob.glob(os.path.join(src, "**", "*.rs"), recursive=True):
    lines = open(path, encoding="utf8").read().split("\n")
    for i, line in enumerate(lines):
        m = re.match(r"\s*pub(?:\([^)]*\))?\s+enum\s+(\w+)", line)
        if not m or m.group(1) not in names:
            continue
        depth, variants, j = 0, [], i
        while j < len(lines):
            s = re.sub(r"//.*", "", STR.sub('""', lines[j]))
            if depth == 1:
                vm = re.match(r"\s*([A-Z]\w*)\s*[({,=]?", s)
                if vm and not s.strip().startswith("#"):
                    variants.append(vm.group(1))
            depth += s.count("{") - s.count("}")
            if j > i and depth <= 0:
                break
            j += 1
        rel = os.path.relpath(path, src).replace(os.sep, "/")
        print(f"{m.group(1)} ({rel}:{i + 1}, {len(variants)}): {' '.join(variants)}")
