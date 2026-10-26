"""
rust_source - what the `check_*.py` gates that read Rust share.

Not a gate. Two readers of a `.rs` file's text, each keeping line numbers, so
a match's line is the source's:

- `code_only` blanks every comment and string literal, so a gate reads code
  and never a doc comment that *mentions* what it looks for;
- `test_items` finds the items under `#[cfg(test)]`, which a gate about the
  engine's behavior reads past: a test builds and inspects a board directly,
  which is what a test is for.

On top of them, `holders` lists every struct field and enum payload (what a
value is stored in), and `type_declarations` every struct and enum with all
its members, unit variants included (what a card's text can say).
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


# --- The holders of a type: every struct field and enum payload ------------

ITEM = re.compile(r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(struct|enum)\s+([A-Z][A-Za-z0-9_]*)")
ATTRIBUTE = re.compile(r"#\[[^\]]*(?:\[[^\]]*\][^\]]*)*\]")
VISIBILITY = re.compile(r"^\s*pub(?:\([^)]*\))?\s+")
TYPE_NAME = re.compile(r"\b[A-Z][A-Za-z0-9_]*\b")
ASSOCIATED = re.compile(r"\b([A-Z][A-Za-z0-9_]*)::[A-Z][A-Za-z0-9_]*\b")
PRIMITIVE = re.compile(r"\b(?:bool|char|str|[iu](?:8|16|32|64|128|size)|f32|f64)\b")


class Holder:
    """One struct field or enum variant's payload, outside test code.

    `owner` is the struct or enum it is in, `variant` its variant (empty for
    a struct), `field` its name (`.0` for a tuple's first), `ty` its type with
    whitespace collapsed, and `doc` its doc comment joined, or its variant's
    for a tuple payload.
    """

    __slots__ = ("path", "line", "owner", "variant", "field", "ty", "doc")

    def __init__(self, path, line, owner, variant, field, ty, doc):
        self.path, self.line, self.owner, self.variant = path, line, owner, variant
        self.field, self.ty, self.doc = field, ty, doc

    def name(self) -> str:
        """`Owner.field`, or `Owner::Variant.field`."""
        return f"{self.owner}{'::' + self.variant if self.variant else ''}.{self.field}"

    def types(self) -> set[str]:
        """The type names its type mentions, primitives among them; none for
        a borrow, which is a view of a value held elsewhere, never storage."""
        if self.ty.startswith("&"):
            return set()
        # `T::Held` names an associated type of `T`, not a type called `Held`.
        ty = ASSOCIATED.sub(r"\1", self.ty)
        return set(TYPE_NAME.findall(ty)) | set(PRIMITIVE.findall(ty))


def _split_top(body: str):
    """(offset, part) for each comma-separated part of `body` at depth 0."""
    text = body.replace("->", "~~")
    out, depth, start = [], 0, 0
    for i, c in enumerate(text + ","):
        if c in "<([{":
            depth += 1
        elif c in ">)]}":
            depth -= 1
        if c == "," and depth == 0:
            part = text[start:i]
            if part.strip():
                out.append((start + len(part) - len(part.lstrip()), part.strip().replace("~~", "->")))
            start = i + 1
    return out


def _closing(code: str, i: int) -> int:
    opening, closing = code[i], {"{": "}", "(": ")"}[code[i]]
    depth = 0
    for j in range(i, len(code)):
        depth += (code[j] == opening) - (code[j] == closing)
        if depth == 0:
            return j
    return len(code)


def _body_start(code: str, j: int):
    """The `{`, `(` or `;` after an item's name and generics."""
    depth = 0
    while j < len(code):
        c = code[j]
        depth += (c == "<") - (c == ">")
        if depth == 0 and c in "{(;":
            return j
        j += 1
    return None


def _doc_above(lines: list[str], line: int) -> str:
    got, i = [], line - 2
    while i >= 0:
        text = lines[i].strip()
        if text.startswith("#["):
            i -= 1
        elif text.startswith("///"):
            got.append(text[3:].strip())
            i -= 1
        else:
            break
    return " ".join(reversed(got))


def holders(path: str, text: str) -> list[Holder]:
    """Every field and enum payload `text` declares outside `#[cfg(test)]`."""
    code = code_only(text)
    lines = text.splitlines()
    tests = test_items(code)
    line_of = lambda offset: code.count("\n", 0, offset) + 1
    blank_attributes = lambda s: ATTRIBUTE.sub(lambda m: " " * len(m.group(0)), s)

    def fields(start: int, body: str, braces: bool):
        for k, (offset, part) in enumerate(_split_top(blank_attributes(body))):
            part = VISIBILITY.sub("", part)
            if braces:
                if ":" not in part:
                    continue
                name, ty = part.split(":", 1)
                yield start + offset, name.strip(), " ".join(ty.split())
            else:
                yield start + offset, f".{k}", " ".join(part.split())

    out = []
    for m in ITEM.finditer(code):
        if any(a <= m.start() < b for a, b in tests):
            continue
        kind, owner = m.groups()
        k = _body_start(code, m.end())
        if k is None or code[k] == ";":
            continue
        body = code[k + 1 : _closing(code, k)]
        if kind == "struct":
            for offset, field, ty in fields(k + 1, body, code[k] == "{"):
                line = line_of(offset)
                out.append(Holder(path, line, owner, "", field, ty, _doc_above(lines, line)))
            continue
        for offset, part in _split_top(blank_attributes(body)):
            variant = re.match(r"([A-Z][A-Za-z0-9_]*)\s*", part)
            if not variant:
                continue
            at = k + 1 + offset + variant.end()
            if at >= len(code) or code[at] not in "{(":
                continue
            variant_doc = _doc_above(lines, line_of(k + 1 + offset))
            payload = code[at + 1 : _closing(code, at)]
            for field_offset, field, ty in fields(at + 1, payload, code[at] == "{"):
                line = line_of(field_offset)
                doc = variant_doc if field.startswith(".") else _doc_above(lines, line)
                out.append(Holder(path, line, owner, variant.group(1), field, ty, doc))
    return out


class TypeDecl:
    """One struct or enum outside test code: its kind, name, file and line,
    and its members in order, as (name, payload or type, first sentence of
    its doc). An enum's unit variant has an empty payload; `holders` skips
    it, since nothing is stored in it, and this lists it, since a card's text
    can say it."""

    def __init__(self, kind, name, path, line):
        self.kind, self.name, self.path, self.line = kind, name, path, line
        self.members: list[tuple[str, str, str]] = []

    def mentions(self) -> set[str]:
        """The type names its members' payloads and types mention."""
        return {t for _, payload, _ in self.members for t in TYPE_NAME.findall(payload)}


def type_declarations(path: str, text: str) -> list[TypeDecl]:
    """Every struct and enum `text` declares outside `#[cfg(test)]`."""
    code, lines = code_only(text), text.splitlines()
    tests = test_items(code)
    line_of = lambda offset: code.count("\n", 0, offset) + 1
    out = []
    for m in ITEM.finditer(code):
        if any(a <= m.start() < b for a, b in tests):
            continue
        kind, name = m.groups()
        k = _body_start(code, m.end())
        if k is None or code[k] == ";":
            continue
        item = TypeDecl(kind, name, path, line_of(m.start()))
        body = ATTRIBUTE.sub(lambda a: " " * len(a.group(0)), code[k + 1 : _closing(code, k)])
        for offset, part in _split_top(body):
            doc = _doc_above(lines, line_of(k + 1 + offset)).split(". ")[0]
            if kind == "enum":
                arm = re.match(r"([A-Z][A-Za-z0-9_]*)\s*(.*)", part, re.S)
                if arm:
                    item.members.append((arm.group(1), " ".join(arm.group(2).split()), doc))
            elif code[k] == "{" and ":" in part:
                field, ty = part.split(":", 1)
                item.members.append((VISIBILITY.sub("", field).strip(), " ".join(ty.split()), doc))
        out.append(item)
    return out


ALIAS = re.compile(r"(?m)^(?:pub(?:\([^)]*\))?\s+)?type\s+([A-Z][A-Za-z0-9_]*)(?:<[^=;]*>)?\s*=\s*([^;]+);")


def aliases(text: str) -> dict[str, set[str]]:
    """Each `type Name = ...;` outside `#[cfg(test)]`, and the type names its
    right side mentions: what a field whose type is the alias holds."""
    code = code_only(text)
    tests = test_items(code)
    return {
        m.group(1): set(TYPE_NAME.findall(m.group(2)))
        for m in ALIAS.finditer(code)
        if not any(a <= m.start() < b for a, b in tests)
    }


def owned_types(by_type: dict, root: str, stops: set[str], alias_of: dict | None = None) -> set[str]:
    """The types `root` holds, transitively through its holders' types and
    through any alias among them, never entering one in `stops`."""
    alias_of = alias_of or {}
    owned, seen, todo = set(), set(), [root]
    while todo:
        name = todo.pop()
        if name in seen or name in stops:
            continue
        seen.add(name)
        if name in alias_of:
            todo.extend(alias_of[name] - seen)
        if name not in by_type:
            continue
        owned.add(name)
        for holder in by_type[name]:
            todo.extend(holder.types() - seen)
    return owned
