"""
rust_source - what the `check_*.py` gates that read Rust share.

Not a gate. Two readers of a `.rs` file's text, each keeping line numbers, so
a match's line is the source's:

- `code_only` blanks every comment and string literal, so a gate reads code
  and never a doc comment that *mentions* what it looks for;
- `test_items` finds the items under `#[cfg(test)]`, which a gate about the
  engine's behavior reads past: a test builds and inspects a board directly,
  which is what a test is for.
"""

import re

RAW_STRING = re.compile(r'b?r(#*)"')
CHAR_LITERAL = re.compile(r"'(?:\\u\{[0-9a-fA-F]+\}|\\.|[^'\\\n])'")
CFG_TEST = re.compile(r"#\[cfg\((?:test|any\(test\b[^\]]*\))\)\]")


def code_only(text: str) -> str:
    """`text` with every comment and string literal blanked to spaces, newlines
    kept, so a match's line number is the source's."""
    out = []
    i, n = 0, len(text)
    blank = lambda s: "".join("\n" if c == "\n" else " " for c in s)
    while i < n:
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end < 0 else end
            out.append(blank(text[i:end]))
            i = end
        elif text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            out.append(blank(text[i:j]))
            i = j
        elif (raw := RAW_STRING.match(text, i)) and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            close = '"' + raw.group(1)
            end = text.find(close, raw.end())
            end = n if end < 0 else end + len(close)
            out.append(blank(text[i:end]))
            i = end
        elif text[i] == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            out.append(blank(text[i : j + 1]))
            i = j + 1
        elif text[i] == "'" and (char := CHAR_LITERAL.match(text, i)):
            out.append(blank(char.group(0)))
            i = char.end()
        else:
            out.append(text[i])
            i += 1
    return "".join(out)


def test_items(code: str) -> list[tuple[int, int]]:
    """The (start, end) offsets in `code`, a `code_only` text, of each item
    under `#[cfg(test)]` or `#[cfg(any(test, ...))]`: a `mod tests { ... }`,
    a test-only `fn`, a `use` ending in `;`."""
    spans = []
    for m in CFG_TEST.finditer(code):
        j = m.end()
        while j < len(code) and code[j] not in "{;":
            j += 1
        if j < len(code) and code[j] == "{":
            depth = 0
            while j < len(code):
                depth += {"{": 1, "}": -1}.get(code[j], 0)
                if depth == 0:
                    break
                j += 1
        spans.append((m.start(), j + 1))
    return spans
