#!/usr/bin/env python3
"""
Extract per-phase ATOM/BOUNDARY/COMP indexes from session summaries.

Reads all session summary files and produces:
  1. A combined global index (all entries, sorted by phase then rule)
  2. Per-phase files: phase-index-{name}.md

Usage:
    python extract-phase-index.py                # writes all outputs
    python extract-phase-index.py --phase 5-pre  # writes only Phase 5-Pre index
    python extract-phase-index.py --stats         # print phase counts only
"""

import argparse
import os
import re
from collections import defaultdict
from pathlib import Path

SUMMARIES_DIR = Path(__file__).parent / "summaries"
OUTPUT_DIR = Path(__file__).parent

SESSIONS = ["1", "2", "3", "4", "5", "6", "7a", "7b", "8", "9a", "9b", "10"]

# Phases 1-4 are already implemented
IMPLEMENTED_PHASES = {"1", "2", "3", "4"}

# Canonical phase ordering
PHASE_ORDER = [
    "ALREADY-IMPL",
    "Phase 5-Pre",
    "Phase 5-Layers",
    "Phase 6",
    "Phase 7",
    "Phase 8",
    "Phase 9",
    "Post-v1",
    "Cross-cutting",
    "DEFERRED",
    "COMP-REF",
    "UNKNOWN",
]


def normalize_phase(raw: str) -> str:
    """Map a raw phase string to a canonical phase name.
    
    Strategy: extract all phase numbers mentioned, then assign to the
    LATEST phase (that's when the test can actually be implemented).
    Special cases for multi-phase labels, L-tickets, already-implemented, etc.
    """
    cleaned = raw.strip()
    lowered = cleaned.lower()

    # Skip COMP entries that have ATOM ID lists instead of phase labels
    if re.match(r'^ATOM-', cleaned):
        return "COMP-REF"

    # Already-implemented labels
    if "already" in lowered or lowered == "impl" or lowered.startswith("partial impl"):
        return "ALREADY-IMPL"

    # Post-v1
    if "post" in lowered or "pre-phase" in lowered:
        return "Post-v1"

    # DEFERRED without a phase number
    if lowered == "deferred":
        return "DEFERRED"

    # Per-system / cross-cutting
    if "per-system" in lowered or "cross" in lowered:
        return "Cross-cutting"

    # Extract all phase numbers mentioned
    # Matches: "Phase 5", "Phase 5-Pre", "Phase 7", "Phase 8", etc.
    phase_nums = re.findall(r'phase\s*(\d+)', lowered)
    
    # Also check for standalone numbers like in "Phase 7 + 8"
    # where "8" isn't preceded by "phase"
    combo_nums = re.findall(r'(\d+)', lowered)
    
    # Check for L-ticket references (Phase 5-Layers indicators)
    has_l_ticket = bool(re.search(r'\bL\d+', cleaned))
    # Check for T-ticket references (Phase 5-Pre indicators) 
    has_t_ticket = bool(re.search(r'\bT\d+', cleaned))
    # Check for D-ticket references
    has_d_ticket = bool(re.search(r'\bD\d+', cleaned))

    # Check for explicit sub-phase labels
    has_pre = "5-pre" in lowered or "5 pre" in lowered or "pre" in lowered
    has_layers = "5-layer" in lowered or "5 layer" in lowered or "layer" in lowered

    if not phase_nums and not combo_nums:
        # No phase numbers at all — check for L/T tickets
        if has_l_ticket:
            return "Phase 5-Layers"
        if has_t_ticket:
            return "Phase 5-Pre"
        return "UNKNOWN"

    # Use phase_nums if available, else fall back to combo_nums
    nums = [int(n) for n in phase_nums] if phase_nums else [int(n) for n in combo_nums]
    
    # Filter to valid phase range (1-9)
    nums = [n for n in nums if 1 <= n <= 9]
    if not nums:
        return "UNKNOWN"

    # Phases 1-4 are already implemented
    # If ALL mentioned phases are 1-4, it's already implemented
    if all(n <= 4 for n in nums):
        return "ALREADY-IMPL"

    # Take the latest (highest) phase number — that's when it can be built
    latest = max(nums)

    # Special handling for Phase 5 sub-phases
    if latest == 5:
        if has_layers or has_l_ticket:
            return "Phase 5-Layers"
        if has_pre or has_t_ticket:
            return "Phase 5-Pre"
        # Bare "Phase 5" with L-ticket in the original
        if has_l_ticket:
            return "Phase 5-Layers"
        # Default Phase 5 → Phase 5-Pre (safer default)
        return "Phase 5-Pre"

    phase_map = {
        6: "Phase 6",
        7: "Phase 7",
        8: "Phase 8",
        9: "Phase 9",
    }
    return phase_map.get(latest, "UNKNOWN")


def phase_sort_key(phase: str) -> int:
    try:
        return PHASE_ORDER.index(phase)
    except ValueError:
        return len(PHASE_ORDER)


def parse_summary_table(filepath: Path, session_id: str) -> list[dict]:
    """Parse ATOM/BOUNDARY/COMP table rows from a session summary."""
    entries = []
    with open(filepath, "r", encoding="utf-8") as f:
        lines = f.readlines()

    in_table = False
    header_cols = []

    for line in lines:
        stripped = line.strip()

        # Detect table header
        if re.match(r"^\|\s*ID\s*\|", stripped, re.IGNORECASE):
            # Parse header columns
            header_cols = [c.strip().lower() for c in stripped.split("|")[1:-1]]
            in_table = True
            continue

        # Skip separator rows
        if in_table and re.match(r"^\|[\s\-:]+\|", stripped):
            continue

        # Parse data rows
        if in_table and stripped.startswith("|"):
            cols = [c.strip() for c in stripped.split("|")[1:-1]]
            if len(cols) >= 4:
                entry_id = cols[0].strip()
                # Only keep ATOM, BOUNDARY, COMP entries
                if re.match(r"^(ATOM|BOUNDARY|COMP)-", entry_id):
                    entry = {
                        "id": entry_id,
                        "rule": cols[1].strip() if len(cols) > 1 else "",
                        "summary": cols[2].strip() if len(cols) > 2 else "",
                        "phase_raw": cols[3].strip() if len(cols) > 3 else "",
                        "ticket": cols[4].strip() if len(cols) > 4 else "",
                        "tags": cols[5].strip() if len(cols) > 5 else "",
                        "session": session_id,
                    }
                    entry["phase"] = normalize_phase(entry["phase_raw"])
                    entries.append(entry)
            continue

        # End of table (non-table line after table started)
        if in_table and not stripped.startswith("|") and stripped:
            # Check if this is a new section header — could be another table
            if stripped.startswith("#"):
                in_table = False

    return entries


def rule_sort_key(entry: dict) -> tuple:
    """Sort by rule number for stable ordering within a phase."""
    rule = entry.get("rule", "")
    # Extract number parts: "702.84a" -> [702, 84]
    parts = re.findall(r"(\d+)", rule)
    nums = [int(p) for p in parts]
    # Pad to 4 elements for consistent comparison
    while len(nums) < 4:
        nums.append(0)
    # Append any trailing letter as a separate element
    letters = re.findall(r"[a-z]+", rule.lower())
    letter_part = letters[-1] if letters else ""
    # Return (int, int, int, int, str) — always comparable
    return (nums[0], nums[1], nums[2], nums[3], letter_part)


def collect_all_entries() -> list[dict]:
    """Read all session summaries and collect every table entry."""
    all_entries = []
    for session_id in SESSIONS:
        path = SUMMARIES_DIR / f"session-{session_id}-summary.md"
        if not path.exists():
            print(f"  WARNING: {path.name} not found, skipping")
            continue
        entries = parse_summary_table(path, session_id)
        all_entries.extend(entries)
    return all_entries


def deduplicate(entries: list[dict]) -> list[dict]:
    """Remove exact ID duplicates, keeping the first occurrence."""
    seen = set()
    result = []
    for e in entries:
        if e["id"] not in seen:
            seen.add(e["id"])
            result.append(e)
    return result


def format_phase_table(entries: list[dict], phase_name: str) -> str:
    """Format a phase's entries as a markdown table."""
    sorted_entries = sorted(entries, key=rule_sort_key)
    lines = [
        f"## {phase_name}",
        f"",
        f"**{len(sorted_entries)} entries**",
        f"",
        f"| ID | Rule | Summary | Ticket | Session | Tags |",
        f"|----|------|---------|--------|---------|------|",
    ]
    for e in sorted_entries:
        lines.append(
            f"| {e['id']} | {e['rule']} | {e['summary']} | {e['ticket']} | S{e['session']} | {e['tags']} |"
        )
    lines.append("")
    return "\n".join(lines)


def write_global_index(entries: list[dict]):
    """Write the combined global index grouped by phase."""
    by_phase = defaultdict(list)
    for e in entries:
        by_phase[e["phase"]].append(e)

    lines = [
        "# Global Atomic Test Index",
        "",
        f"> Extracted from {len(SESSIONS)} session summaries",
        f"> Total entries: {len(entries)} (after dedup)",
        "",
        "---",
        "",
    ]

    # Summary table
    lines.append("## Phase Counts\n")
    lines.append("| Phase | ATOMs | BOUNDARYs | COMPs | Total |")
    lines.append("|-------|-------|-----------|-------|-------|")
    for phase in sorted(by_phase.keys(), key=phase_sort_key):
        ph_entries = by_phase[phase]
        atoms = sum(1 for e in ph_entries if e["id"].startswith("ATOM-"))
        bounds = sum(1 for e in ph_entries if e["id"].startswith("BOUNDARY-"))
        comps = sum(1 for e in ph_entries if e["id"].startswith("COMP-"))
        lines.append(f"| {phase} | {atoms} | {bounds} | {comps} | {len(ph_entries)} |")
    lines.append("")
    lines.append("---\n")

    # Per-phase tables
    for phase in sorted(by_phase.keys(), key=phase_sort_key):
        lines.append(format_phase_table(by_phase[phase], phase))
        lines.append("---\n")

    outpath = OUTPUT_DIR / "global-test-index.md"
    with open(outpath, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"Wrote {outpath.name} ({len(entries)} entries)")


def write_phase_file(entries: list[dict], phase: str):
    """Write a single phase index file."""
    safe_name = phase.lower().replace(" ", "-").replace("/", "-")
    filename = f"phase-index-{safe_name}.md"

    lines = [
        f"# {phase} — Test Index",
        "",
        f"> {len(entries)} entries extracted from session summaries",
        "",
        "---",
        "",
        format_phase_table(entries, phase),
    ]

    outpath = OUTPUT_DIR / filename
    with open(outpath, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"Wrote {filename} ({len(entries)} entries)")


def main():
    parser = argparse.ArgumentParser(description="Extract per-phase test indexes from session summaries")
    parser.add_argument("--phase", type=str, default=None,
                        help="Extract only this phase (e.g. '5-pre', '6', '7', '8', '9')")
    parser.add_argument("--stats", action="store_true",
                        help="Print phase counts only, don't write files")
    args = parser.parse_args()

    print("Collecting entries from session summaries...")
    all_entries = collect_all_entries()
    print(f"  Raw entries: {len(all_entries)}")

    all_entries = deduplicate(all_entries)
    print(f"  After dedup: {len(all_entries)}")

    by_phase = defaultdict(list)
    for e in all_entries:
        by_phase[e["phase"]].append(e)

    print(f"\nPhase breakdown:")
    for phase in sorted(by_phase.keys(), key=phase_sort_key):
        count = len(by_phase[phase])
        atoms = sum(1 for e in by_phase[phase] if e["id"].startswith("ATOM-"))
        print(f"  {phase}: {count} entries ({atoms} ATOMs)")

    if args.stats:
        return

    if args.phase:
        # Normalize the requested phase
        target = normalize_phase(args.phase)
        if target in by_phase:
            write_phase_file(by_phase[target], target)
        else:
            print(f"ERROR: Phase '{args.phase}' (normalized: '{target}') not found")
            print(f"Available: {list(by_phase.keys())}")
    else:
        # Write everything
        write_global_index(all_entries)
        for phase in sorted(by_phase.keys(), key=phase_sort_key):
            if phase != "ALREADY-IMPL":  # Skip already-implemented
                write_phase_file(by_phase[phase], phase)


if __name__ == "__main__":
    main()
