"""What reads a value chosen as an effect resolves, and who chooses it.

    python plans/references/chosen-value-readers-census.py                 # both tables
    python plans/references/chosen-value-readers-census.py --names READER  # one row's cards ("none", "inline" too)

Behind `values-chosen-at-resolution`'s design note
(`plans/handoffs/values-chosen-at-resolution.md`): after "choose a color",
what does the text do with the color? Its population is exactly
`choice-kinds-census.py`'s "value at resolution" rows, read through that
script's own predicates, with piles and votes left out (the row's second PR).

WHAT IT COUNTS
--------------
Each card's clauses chosen at resolution (`value_clauses`), searched for the
words that name the value again, sorted by the reader that takes them:

- a filter: "creatures of the chosen color", "cards with that name", "power
  greater than or equal to the chosen number", "creatures that player controls";
- a player acted on: "that player loses", "you and that player each" (counted
  only where the value is a player, or a word each player is sorted by:
  "that player" after "target player" is the target's);
- an amount: "that many", "that much";
- a change's own parameter: "becomes that type", "protection from the chosen
  color", "gains that ability";
- a condition: "if that card has the chosen name", "for each player who chose".

"Inline" is a choice made inside the instruction that uses it ("becomes the
creature type of your choice"), which no later clause names. A card can sit
in several rows. The words are matched in the clause, never anywhere in the
text, but a word match is still approximate: every row under 40 was read by
hand at the design (2026-11-14), and so were the 37 player cards of the
player row, whose bare "they" and "them" the regex cannot tell from a
target's.

The second table is who chooses: the controller, another player the text
names, each player in turn (CR 608.2e), or no one, at random.
"""
import importlib.util
import os
import re
import sys
from collections import Counter, OrderedDict

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("ck", os.path.join(HERE, "choice-kinds-census.py"))
ck = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ck)

R = lambda p: re.compile(p, re.I)

INLINE = R(r"\b(?:creature|basic land|card) type of your choice\b|\bcolor of your choice\b|\b(?:player|opponent) of your choice\b")

READERS = OrderedDict([
    ("filter", R(r"\b(?:of|with|from) (?:the chosen|that|a|the last chosen) (?:color|type|creature type|card type|name|quality|kind)\b"
                 r"|\bwith (?:the chosen name|that name|a name chosen)\b|\bthe chosen type\b|\bof the chosen kind\b"
                 r"|\b(?:mana value|power)[^.]*?\b(?:the chosen number|that number|the number|the chosen quality|that quality)\b"
                 r"|\b(?:creatures?|permanents?|lands?|cards?) (?:that|the chosen) player controls\b"
                 r"|\baren't of (?:the chosen|a) type\b|\bof a type chosen this way\b|\bthe creature type of your choice\b(?! until)"
                 r"|\bshares? a color with\b(?=[^.]*chosen)|\bof that type\b|\bof that color\b|\bthe card with the chosen name\b")),
    ("a player acted on", R(r"\b(?:that player|the chosen player|that opponent|the last chosen player)\b|\b(?:they|them|their)\b"
                            r"|\beach (?:friend|foe)\b")),
    ("an amount", R(r"\bthat many\b|\bthat much\b|\bequal to the (?:highest|chosen) number\b|\bequal to the number they guessed\b"
                    r"|\bthe last chosen number\b")),
    ("a change's parameter", R(r"\bbecomes? (?:the chosen|that) (?:color|type)\b|\bbecomes? that type\b|\bgains? that ability\b"
                               r"|\b(?:protection|hexproof|landwalk) (?:from|of) (?:the chosen|that) (?:color|type|card type)\b"
                               r"|\bmana of the chosen color\b|\bproduce mana of the chosen color\b|\bbecomes the chosen color\b"
                               r"|\b(?:protection|hexproof) from the (?:color|card type) of your choice\b"
                               r"|\bbecomes the (?:creature|basic land) type of your choice\b")),
    ("a condition", R(r"\bif\b[^.]*\b(?:the chosen|that name|the name they chose|is the chosen|of the chosen)\b"
                      r"|\b(?:who|each opponent) chose\b|\bguess(?:es|ed)?\b|\bif you win all\b")),
])
# A duration or a later event in the clause: the value outlives the
# resolution, in a lasting effect or a delayed trigger.
LASTING = R(r"\buntil (?:end of turn|your next turn|the end of)\b|\bthis turn\b|\bcan't be cast\b|\bcan't be played\b|\bemblem\b|\bwhenever\b|\bat the beginning of\b")

CHOOSERS = [
    ("each player or each opponent, in turn (CR 608.2e)", R(r"\beach (?:player|opponent) (?:secretly )?chooses\b|\bfor each player, choose\b")),
    ("at random", R(r"\bchooses? (?:[\w ]+? )?at random\b|\bat random\b[^.]*\bchosen\b")),
    ("another player the text names", R(r"\b(?:target (?:player|opponent)|that player|an opponent|they) (?:secretly )?chooses\b")),
]


def rows():
    fetched, cards = ck.load()
    families = [k for k in ck.FAMILIES if k.startswith("value at resolution: ") and "pile" not in k and "vote" not in k]
    out = []
    for c in cards:
        t = ck.REMINDER.sub("", c.get("text") or "")
        kinds = [k.removeprefix("value at resolution: ") for k in families if ck.FAMILIES[k](c, t)]
        if not kinds:
            continue
        text = " ".join(x for when, x in ck.value_clauses(t) if when == "resolution")
        readers = [name for name, rx in READERS.items() if rx.search(text)]
        if not any(k.startswith("a player") or k.startswith("a word") for k in kinds):
            readers = [r for r in readers if r != "a player acted on"]
        chooser = next((name for name, rx in CHOOSERS if rx.search(text)), "its controller")
        out.append(dict(name=c["name"], kinds=kinds, readers=readers, inline=bool(INLINE.search(text)),
                        lasting=bool(LASTING.search(text)), chooser=chooser, text=text))
    return fetched, out


def main(argv):
    fetched, found = rows()
    if "--names" in argv:
        want = argv[argv.index("--names") + 1]
        for r in found:
            if (want == "none" and not r["readers"]) or want in r["readers"] or (want == "inline" and r["inline"]):
                print(f"{r['name']} [{'/'.join(r['kinds'])}]{' L' if r['lasting'] else ''} | {r['text'][:200]}")
        return
    print(f"corpus fetched {fetched}; {len(found)} cards choose a value as an effect resolves (piles and votes left out)")
    print("\nWhat reads the value (a card can sit in several rows; L = a duration or later event in the clause):")
    count = Counter(r for f in found for r in f["readers"])
    for name in READERS:
        lasting = sum(1 for f in found if name in f["readers"] and f["lasting"])
        print(f"{count[name]:5d}  {name}   ({lasting} L)")
    print(f"{sum(1 for f in found if f['inline']):5d}  inline: chosen inside the instruction that uses it")
    print(f"{sum(1 for f in found if not f['readers']):5d}  no reader found")
    print("\nBy kind:")
    kinds = Counter(k for f in found for k in f["kinds"])
    for kind, n in kinds.items():
        cells = Counter(r for f in found if kind in f["kinds"] for r in f["readers"])
        inline = sum(1 for f in found if kind in f["kinds"] and f["inline"])
        print(f"  {n:3d}  {kind}: " + ", ".join(f"{r} {c}" for r, c in cells.items()) + f"; inline {inline}")
    print("\nWho chooses:")
    for name, n in Counter(f["chooser"] for f in found).most_common():
        print(f"{n:5d}  {name}")


if __name__ == "__main__":
    main(sys.argv)
