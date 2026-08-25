#!/usr/bin/env python3
"""Scryfall census behind `plans/replacement-architecture.md` §3.2c and §8b.

Two questions, two subcommands:

    python plans/references/replacement-census.py clauses
        Pulls every card matching o:/would.*instead/ and classifies each
        "would ... instead" clause by the event kind it watches. Answers
        §3.2c: does the `Rewrite` algebra need a sixth arm? (2026-08-24: no,
        0 of 574 clauses.)

    python plans/references/replacement-census.py sizing
        For each CR 701 keyword action and each core mutation, counts cards
        that watch it as a unit -- via a replacement ("would X ... instead")
        or a trigger ("whenever ... X"). Answers §8b: how large does
        `GameAction` get? (2026-08-24: ~16 at end of RE, ceiling low 40s.)

**This is a research tool, not a generator.** It queries a live external API
and prints numbers a human reads and pastes into prose. It produces no file
the project reads, and nothing in the repo is derived from it -- so the lesson
that got `extract-phase-index.py` deleted (a script that regenerated
*authoritative artifacts* from a stale source, silently reintroducing drift)
does not apply. Re-run it when the algebra changes; the numbers in the doc are
stamped with the date they were taken.

Results are cached in this directory (gitignored) so a rerun is cheap and a
rate-limit interruption never loses work. Scryfall asks for a courteous request
rate; the delay and backoff below are deliberate -- do not tighten them.
"""
import argparse
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter, defaultdict

UA = "mtgsim-research/1.0 (contact: maiercluke@gmail.com)"
HERE = os.path.dirname(os.path.abspath(__file__))
DELAY = 0.3


def _get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    for attempt in range(6):
        try:
            with urllib.request.urlopen(req) as r:
                return json.load(r)
        except urllib.error.HTTPError as e:
            if e.code == 404:
                return None
            if e.code == 429:
                time.sleep(5 * (attempt + 1))
                continue
            raise
    raise RuntimeError("rate limited out: " + url)


def search_all(q):
    """Every card matching `q`, following pagination."""
    url = f"https://api.scryfall.com/cards/search?q={urllib.parse.quote(q)}&unique=cards"
    out = []
    while url:
        d = _get(url)
        if d is None:
            break
        out.extend(d["data"])
        url = d.get("next_page")
        time.sleep(DELAY)
    return out


def count(q):
    d = _get(f"https://api.scryfall.com/cards/search?q={urllib.parse.quote(q)}&unique=cards")
    return 0 if d is None else d.get("total_cards", 0)


# --------------------------------------------------------------------------
# `clauses` -- §3.2c
# --------------------------------------------------------------------------

# Non-vocabulary kinds are tested first so a broad arm cannot swallow them.
KINDS_MISSING = [
    ("!RollDice", r"would roll"),
    ("!Scry", r"would scry|would surveil"),
    ("!CounterSpell", r"would counter|would be countered"),
    ("!SearchLibrary", r"would search"),
    ("!Sacrifice", r"would (be )?sacrifice"),
    ("!Cast/Play", r"would cast|would play"),
    ("!Copy", r"would copy|would be copied"),
    ("!Attack/Block", r"would (attack|block)"),
    ("!TurnFaceUp", r"turned face"),
    ("!Vote", r"would vote"),
]
KINDS_KNOWN = [
    ("EnterBattlefield*", r"would enter"),
    ("ZoneChange*", r"would be put into|would be exiled|would die|would be destroyed|would leave"
                    r"|would be returned|would be shuffled|would be milled|would mill"
                    r"|would put .*into (a|its|their|your|the) (graveyard|library|hand|exile)"
                    r"|would be discarded|would discard|causes you to discard"),
    ("DealDamage*", r"damage|would be dealt"),
    ("DrawCard*", r"would draw"),
    ("AddCounters*", r"counters?"),
    ("CreateTokens*", r"would create|tokens? would be created|would be created"),
    ("GainLife*", r"would gain .*life"),
    ("LoseLife*", r"would lose .*life|life total would be reduced|would pay life"),
    ("TapUntap*", r"would (become )?(untap|tap)"),
    ("ProduceMana*", r"mana"),
    ("BeginStep*", r"would (begin|skip)|skips? (their|your|the)"),
    ("PlayerLosesWins", r"would (lose|win) the game"),
]


def cmd_clauses(_args):
    cache = os.path.join(HERE, ".census-clauses.json")
    if os.path.exists(cache):
        cards = json.load(open(cache, encoding="utf-8"))
    else:
        cards = search_all("o:/would.*instead/ -is:funny")
        json.dump(cards, open(cache, "w", encoding="utf-8"))

    clauses = []
    for c in cards:
        txt = c.get("oracle_text") or ""
        if not txt and "card_faces" in c:
            txt = "\n".join(f.get("oracle_text", "") for f in c["card_faces"])
        for sent in re.split(r"(?<=\.)\s+|\n", txt):
            if re.search(r"would", sent, re.I) and re.search(r"instead", sent, re.I):
                clauses.append((c["name"], " ".join(sent.split())))

    order = KINDS_MISSING + KINDS_KNOWN

    def kind(s):
        for name, pat in order:
            if re.search(pat, s, re.I):
                return name
        return "UNCLASSIFIED"

    counts, ex = Counter(), defaultdict(list)
    for name, s in clauses:
        k = kind(s)
        counts[k] += 1
        if len(ex[k]) < 30:
            ex[k].append((name, s))

    print(f"cards: {len(cards)}   clauses: {len(clauses)}\n")
    known = missing = unk = 0
    for k, n in counts.most_common():
        print(f"  {k:<20} {n:>4}")
        if k.endswith("*"):
            known += n
        elif k.startswith("!"):
            missing += n
        else:
            unk += n
    print(f"\nin the GameAction vocabulary: {known}")
    print(f"needs a new GameAction:       {missing}")
    print(f"unclassified:                 {unk}   (expect CR 701 keyword actions)")
    print("\nUNCLASSIFIED residual:")
    for name, s in ex["UNCLASSIFIED"]:
        print(f"  - {name}: {s[:165]}")
    print("\nA sixth `Rewrite` arm is needed only if some clause above cannot be")
    print("expressed as Prevent / Amount / Retarget / EnterWith / Instead. Read the")
    print("residual and decide by hand -- this script counts, it does not judge.")


# --------------------------------------------------------------------------
# `sizing` -- §8b
# --------------------------------------------------------------------------
# (label, "would <verb>" forms, "whenever ... <verb>" forms)
ACTIONS = [
    ("Activate", ["would activate"], ["activates an ability", "activates a"]),
    ("Attach", ["would be attached", "would attach"], ["becomes attached"]),
    ("Cast", ["would cast"], ["casts a", "cast a", "you cast"]),
    ("Counter", ["would counter", "would be countered"], ["is countered", "counters a"]),
    ("Create", ["would create", "would be created"], ["creates a token", "create a token", "token is created"]),
    ("Destroy", ["would be destroyed", "would destroy"], ["is destroyed"]),
    ("Discard", ["would discard", "causes you to discard"], ["discards a card", "discard a card"]),
    ("Exchange", ["would exchange"], ["exchanges"]),
    ("Exile", ["would be exiled", "would exile"], ["is exiled", "exiles a"]),
    ("Fight", ["would fight"], ["fights"]),
    ("Goad", ["would be goaded"], ["becomes goaded", "is goaded"]),
    ("Investigate", ["would investigate"], ["investigates", "you investigate"]),
    ("Mill", ["would mill"], ["mills a card", "mills one or more"]),
    ("Play", ["would play"], ["plays a land", "you play a land"]),
    ("Regenerate", ["would be regenerated"], ["is regenerated"]),
    ("Reveal", ["would reveal"], ["reveals a card"]),
    ("Sacrifice", ["would be sacrificed", "would sacrifice"], ["sacrifices a"]),
    ("Scry", ["would scry"], ["you scry", "scries"]),
    ("Search", ["would search"], ["searches their library", "searches a library"]),
    ("Shuffle", ["would shuffle"], ["shuffles their library"]),
    ("Surveil", ["would surveil"], ["you surveil", "surveils"]),
    ("Tap/Untap", ["would become tapped", "would become untapped", "would untap", "would tap"],
     ["becomes tapped", "becomes untapped"]),
    ("Transform", ["would transform"], ["transforms"]),
    ("Proliferate", ["would proliferate"], ["you proliferate"]),
    ("Populate", ["would populate"], ["you populate"]),
    ("Vote", ["would vote"], ["votes", "you vote"]),
    ("Explore", ["would explore"], ["explores"]),
    ("Amass", ["would amass"], ["you amass"]),
    ("Learn", ["would learn"], ["you learn"]),
    ("Connive", ["would connive"], ["connives"]),
    ("Venture", ["would venture"], ["you venture"]),
    ("Discover", ["would discover"], ["you discover"]),
    ("Forage", ["would forage"], ["you forage"]),
    ("Incubate", ["would incubate"], ["you incubate"]),
    ("Adapt", ["would adapt"], ["adapts"]),
    ("Monstrosity", ["would become monstrous"], ["becomes monstrous"]),
    ("Exert", ["would exert"], ["exerts", "you exert"]),
    ("Support", ["would support"], ["supports"]),
    ("Bolster", ["would bolster"], ["you bolster"]),
    ("Manifest", ["would manifest"], ["you manifest"]),
    ("Meld", ["would meld"], ["melds"]),
    ("Detain", ["would be detained"], ["is detained"]),
    ("Clash", ["would clash"], ["you clash"]),
    ("Fateseal", ["would fateseal"], ["you fateseal"]),
    ("RollDice", ["would roll"], ["rolls one or more dice", "you roll"]),
    ("FlipCoin", ["would flip"], ["you flip a coin", "flips a coin"]),
    # core mutations, for comparison -- prefixed * in the output
    ("*DealDamage", ["would deal", "would be dealt"], ["deals damage", "is dealt damage"]),
    ("*Draw", ["would draw"], ["you draw a card", "draws a card"]),
    ("*GainLife", ["would gain"], ["gains life", "you gain life"]),
    ("*LoseLife", ["would lose"], ["loses life", "you lose life"]),
    ("*Counters", ["counters would be put", "would be put on"], ["counter is put on", "counters are put on"]),
    ("*EnterBF", ["would enter"], ["enters", "enters the battlefield"]),
    ("*ZoneChange", ["would be put into"], ["is put into a graveyard", "dies"]),
    ("*LoseGame", ["would lose the game"], ["loses the game"]),
    ("*BeginStep", ["would begin", "skips"], ["at the beginning of"]),
    ("*Mana", ["mana would", "would be spent"], ["adds one or more mana", "taps for mana"]),
]


def cmd_sizing(_args):
    cache_path = os.path.join(HERE, ".census-sizing.json")
    cache = json.load(open(cache_path, encoding="utf-8")) if os.path.exists(cache_path) else {}
    rows = []
    for label, repl, trig in ACTIONS:
        queries = {
            label + "|R": "(" + " or ".join(f'o:"{f}"' for f in repl) + ") o:instead -is:funny",
            label + "|T": " or ".join(f'o:"{f}"' for f in trig) + " -is:funny",
        }
        for key, q in queries.items():
            if key not in cache:
                cache[key] = count(q)
                time.sleep(DELAY)
        json.dump(cache, open(cache_path, "w", encoding="utf-8"))
        rows.append((label, cache[label + "|R"], cache[label + "|T"]))
        print(f"  {label:<14} repl={rows[-1][1]:>5}  trig={rows[-1][2]:>6}", file=sys.stderr)

    print("\n| Event kind | replacements | triggers |")
    print("|---|---|---|")
    for label, r, t in sorted(rows, key=lambda x: -(x[1] + x[2])):
        print(f"| {label} | {r} | {t} |")

    kw = [r for r in rows if not r[0].startswith("*")]
    dead = [r[0] for r in kw if r[1] + r[2] == 0]
    print(f"\nCR 701 actions sampled:     {len(kw)}")
    print(f"  watched by something:     {len(kw) - len(dead)}")
    print(f"  watched by nothing:       {len(dead)}  -> {dead}")
    print("\nPer-phrasing text search: order-of-magnitude signal, not exact counts.")
    print("Known distortions: Regenerate reads 0 because it replaces *destruction*,")
    print("and EnterBattlefield's trigger count is 'every ETB creature ever printed'.")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("clauses", help="§3.2c -- classify every 'would ... instead' clause")
    sub.add_parser("sizing", help="§8b -- how large does GameAction get")
    args = ap.parse_args()
    {"clauses": cmd_clauses, "sizing": cmd_sizing}[args.cmd](args)


if __name__ == "__main__":
    main()
