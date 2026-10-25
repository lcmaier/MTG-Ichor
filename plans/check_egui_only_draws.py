#!/usr/bin/env python3
"""
check_egui_only_draws - the dev GUI's logic is plain Rust under tests, and
egui only draws (`engineering-practices.md#the-dev-gui`).

GUI code is the part of the tree the owner can least review line by line, so
the review path keeps it to a thin layer over plain Rust, which is reviewed
and tested like the engine. A rule held by convention alone erodes one handy
`egui::` import at a time; this holds it.

    python plans/check_egui_only_draws.py      # exit 1 on a crossing

**What counts as drawing.** A drawing file lays out values the view model has
already built and turns the person's clicks into `Input`s; nothing in it
decides what a test would need to see. Three files are that:

- `src/app.rs`, the window: panels, buttons, the hover text;
- `src/main.rs`, which starts eframe over what `launch` read from the
  command line;
- `tests/screenshots.rs`, which draws the window offscreen for the review
  pictures.

Adding a drawing file is a line in `DRAWING` below, which a review reads in the
diff.

**The rule, both ways.**
- **No other file names a window crate** (egui and its family, eframe, winit,
  wgpu), so the bridge, the snapshot, the prompt, the view model, the session
  and every other test build, run and are tested with no window.
- **No drawing file names the engine** (`mtgsim::`). What the window shows
  comes through the view model, where a test can read it, and never straight
  from the engine.

Comments and string literals are not read: `nothing here knows egui` is a doc
comment, not a dependency.
"""

import re
import sys
from pathlib import Path

from rust_source import code_only

ROOT = Path(__file__).resolve().parent.parent
DEVGUI = ROOT / "devgui"

#: The drawing files, relative to `devgui/`.
DRAWING = {"src/app.rs", "src/main.rs", "tests/screenshots.rs"}

#: The window's crates, named as a path's first segment or in a `use`.
WINDOW_CRATE = re.compile(r"\b(egui(?:_\w+)?|eframe|epaint|winit|wgpu|accesskit)\b")
ENGINE = re.compile(r"\bmtgsim\b")

def crossings(path: Path, drawing: bool):
    """(line, name) for each name `path` may not use."""
    code = code_only(path.read_text(encoding="utf-8"))
    forbidden = ENGINE if drawing else WINDOW_CRATE
    for m in forbidden.finditer(code):
        yield code.count("\n", 0, m.start()) + 1, m.group(0)


def main() -> int:
    files = sorted(p for p in DEVGUI.rglob("*.rs") if "target" not in p.relative_to(DEVGUI).parts)
    relative = {p.relative_to(DEVGUI).as_posix(): p for p in files}
    missing = sorted(DRAWING - relative.keys())
    findings = [
        (name, line, used, name in DRAWING)
        for name, path in relative.items()
        for line, used in crossings(path, name in DRAWING)
    ]

    print(f"egui only draws: {len(files)} .rs files under devgui/, {len(DRAWING)} of them drawing files")
    if not findings and not missing:
        print("\nno window crate outside the drawing files, and no engine path inside them.")
        return 0

    for name in missing:
        print(f"\n  DRAWING names devgui/{name}, which does not exist: take it off the list.")
    if findings:
        print(f"\n{len(findings)} crossing(s):\n")
        for name, line, used, drawing in findings:
            why = "a drawing file names the engine" if drawing else "names a window crate"
            print(f"  devgui/{name}:{line}  {used}  ({why})")
        print(
            "\nLogic goes in plain Rust under tests (the view model, the session), and a\n"
            "drawing file reads what it built. A new drawing file is a line in DRAWING."
        )
    return 1


if __name__ == "__main__":
    sys.exit(main())
