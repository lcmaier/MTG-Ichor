"""Which kinds of event printed replacement effects watch, by family.

    python plans/references/replacement-kinds-census.py            # the counts
    python plans/references/replacement-kinds-census.py --names    # each family's cards and the clause that put them there

Behind item 356's census, the gather filed by the kind of action
(`replacement-architecture.md`, "Where replacement effects come from"; the
design note's table is `git show 7a2c8ae5:plans/handoffs/readiness-close.md`).

WHAT IT COUNTS
--------------
A card is in a family when one of its replacement or prevention effects
*watches* that event: the event inside a "would ... instead" sentence, inside a
"causes you to discard ... instead" one or an "is tapped for mana ... instead"
one, a prevention, a skip, an entry modification ("enters tapped", "enters
with", "as this enters"), or a keyword whose CR 702 definition is a replacement.
Reminder text is stripped first, so a keyword counts once, by its keyword.

A word anywhere in the text is not enough, which is what the first count got
wrong: `o:"causes you to discard"` returned 17 cards, 10 of them triggered
abilities (Ajani's Last Stand, Sand Golem). "If ... instead" with no event in
the "if" (kicker, "if you control a Mountain") is a spell modifying its own
effect, not a replacement, and is not counted.

The second column, "static on a permanent", is the battlefield file's
population: the clause is on a permanent card, not in a triggered ability, not
after an activation cost's colon, not inside a quoted grant, and not a one-shot
("this turn", "the next", "the first time"). It is read off the line's shape,
so it is approximate; the first column is the one that was read card by card.

Verified 2026-10-30 by reading every card of each family under 40 and a seeded
sample of 10 to 12 from each larger one. The classifier counts; it does not
judge, so a new family shows up as "would ... instead, unbucketed" for a human
to read.

INPUT
-----
`.census-cast.json` in this directory, `cast-census.py`'s cache of every
Commander-legal paper card (`legal:commander -t:token game:paper -is:funny`).
If it is missing, run `python plans/references/cast-census.py` first (about
seven minutes). Its fetch date is printed with the counts.
"""
import json
import os
import re
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, ".census-cast.json")

SELF_NOUN = re.compile(
    r"this (?:creature|land|artifact|enchantment|permanent|card|spell|aura|equipment|"
    r"vehicle|planeswalker|battle|saga|class|case|room|siege)",
    re.I,
)
TRIGGERED = re.compile(r"^(?:when|whenever|at the beginning|at end of)", re.I)
PERMANENT = re.compile(r"\b(?:Artifact|Creature|Enchantment|Land|Planeswalker|Battle)\b")

# The event a "would" clause names; first match wins, so the narrower comes first.
WOULD = [
    ("counters put on", r"would get[^,]*(?:\{e\}|counters?)|would (?:put|be put|have)[^,]*counters?|counters? would be put|would enter with[^,]*counters?|would (?:get|receive)[^,]*counters?"),
    ("life total reduced", r"life total would be reduced|would reduce your life total"),
    ("explore", r"would explore"),
    ("connive", r"would connive"),
    ("learn", r"would learn"),
    ("planeswalk", r"would planeswalk"),
    ("assemble", r"would assemble"),
    ("copy a spell", r"would copy"),
    ("mana empties", r"would lose unspent mana"),
    ("damage dealt", r"damage that [^,]*would deal|would (?:deal|be dealt)[^,]*damage|damage would be dealt|damage that would|would reduce[^,]*life total"),
    ("dies or put into a graveyard", r"would (?:die|be put into[^,]*graveyard|go to[^,]*graveyard)|would be (?:put|placed) into[^,]*graveyard|into a graveyard from anywhere"),
    ("leaves the battlefield", r"would leave (?:the battlefield|play)"),
    ("exiled", r"would be (?:exiled|put into exile)"),
    ("returned to hand or library", r"would be (?:returned|put) (?:to|into|on)[^,]*(?:hand|library)|would be shuffled"),
    ("discard", r"would discard|causes? [^,]* to discard"),
    ("tokens created", r"would create|tokens? would be created|would put[^,]*tokens?"),
    ("enters", r"would (?:enter|be put onto the battlefield|be put into play|come into play)|would put [^,]*onto the battlefield"),
    ("draw", r"would draw|would cause [^,]*to draw"),
    ("gain life", r"would cause [^,]*to gain life|would gain life|would gain [^,]*life"),
    ("lose life", r"would lose life|would lose [^,]*life|would pay life"),
    ("mana added", r"would (?:add|produce)"),
    ("untap", r"would (?:untap|become untapped)"),
    ("tap", r"would (?:tap|become tapped)"),
    ("destroyed", r"would be destroyed|would destroy"),
    ("sacrificed", r"would (?:be )?sacrifice"),
    ("scry", r"would scry"),
    ("surveil", r"would surveil"),
    ("mill", r"would mill"),
    ("search", r"would search"),
    ("roll or flip", r"would (?:roll|flip)"),
    ("lose the game", r"would lose the game"),
    ("win the game", r"would win the game"),
    ("countered", r"would be countered|would counter"),
    ("begin a turn, phase or step", r"would (?:begin|start|untap during)"),
    ("proliferate", r"would proliferate"),
]

# Keywords whose CR 702 definition is a replacement effect.
FROM_THE_STACK = "kw: exiled or returned as it leaves the stack (flashback, jump-start, aftermath, harmonize, rebound, buyback)"
SELF_ENTRY_KW = "kw: its own entry (amplify, modular, sunburst, bloodthirst, graft, vanishing, fading, devour, unleash, riot, ravenous, compleated, impending, tribute)"
KEYWORDS = {
    "protection": "kw: protection (702.16e, a prevention)",
    "absorb": "kw: absorb (702.64a)",
    "umbra armor": "kw: umbra armor (702.89a)",
    "totem armor": "kw: umbra armor (702.89a)",
    "madness": "kw: madness (702.35a)",
    "dredge": "kw: dredge (702.52a)",
    "unearth": "kw: unearth (702.84a)",
    **{k: FROM_THE_STACK for k in ("flashback", "jump-start", "aftermath", "harmonize", "rebound", "buyback")},
    **{k: SELF_ENTRY_KW for k in ("amplify", "modular", "sunburst", "bloodthirst", "graft", "vanishing", "fading",
                                  "devour", "unleash", "riot", "ravenous", "compleated", "impending", "tribute")},
}

SELF_ENTRY = {"self: enters tapped", "self: enters with counters", "self: as it enters", SELF_ENTRY_KW}
# Families that are not one of a card's own replacement abilities watching an
# event: the counters and copies that bring one, the grants, the "can't"s.
NOT_A_WATCHER = ("a copy", "grant", "shield counter", "stun counter", "finality counter", "can't", "regenerate")


def normalize(card):
    """Reminder text stripped, and the card's own name and "this <type>" as `~`."""
    text = re.sub(r"\([^)]*\)", "", card.get("text") or "")
    names = set()
    for face in card["name"].split(" // "):
        names |= {face, face.split(",")[0], face.split(" the ")[0]}
        if "Legendary" in (card.get("type") or "") and len(face.split(" ")[0]) > 3:
            names.add(face.split(" ")[0].rstrip(","))
    for name in sorted(names, key=len, reverse=True):
        if len(name) > 2:
            text = text.replace(name, "~")
    return SELF_NOUN.sub("~", text)


def sentences(text):
    for line in text.split("\n"):
        line = line.strip()
        if line:
            for s in re.split(r"(?<=[.!?])\s+(?=[A-Z•])", line):
                yield s.strip(), line


def entry_subject_is_self(subject):
    return subject in ("", "~") or bool(re.search(r"(?:^|\s)~\s*$|^~ and", subject))


def entry_subject_is_resolution(subject):
    """"It enters tapped", "those tokens enter with": a resolution's own object."""
    return bool(re.search(r"\b(?:it|they|those|that|the token|the tokens|these|each of them)\b", subject))


def classify(card):
    """Yield (family, clause) for every replacement clause on the card."""
    for s, line in sentences(normalize(card)):
        low = s.lower()
        if (re.search(r"\bwould\b", low) and re.search(r"\binstead\b", low)) or re.search(r"causes? [^.]* to discard [^.]*instead", low):
            yield next((fam for fam, rx in WOULD if re.search(rx, low)), "would ... instead, unbucketed"), s
        elif "instead" in low and re.search(r"tap[^,.]* for mana|tapped for mana", low):
            yield "mana added", s
        if re.search(r"\bprevent\b", low) and "damage" in low and "can't be prevented" not in low:
            yield "damage prevented", s
        if re.search(r"\bskips?\b", low) and re.search(r"turn|step|phase|combat", low) and not TRIGGERED.match(line):
            yield "skip a turn, phase or step", s
        m = re.search(r"([^,.]*?)\benters? tapped\b", low)
        if m and "attacking" not in low[m.start():m.end() + 20] and not TRIGGERED.match(line):
            subject = m.group(1).strip()
            # A shockland's "As this enters, you may pay 2 life. If you don't, it enters tapped."
            if entry_subject_is_self(subject) or (subject.endswith("it") and line.lower().startswith("as ~ enters")):
                yield "self: enters tapped", s
            elif not entry_subject_is_resolution(subject):
                yield "others enter tapped", s
        m = re.search(r"([^,.]*?)\benters? with\b[^.]*counters?", low)
        if m:
            subject = m.group(1).strip()
            if entry_subject_is_self(subject):
                yield "self: enters with counters", s
            elif not entry_subject_is_resolution(subject):
                yield "others enter with counters", s
        if re.search(r"\bas ~ enters\b|\bas ~ is turned face up\b|\b~ escapes with\b", low):
            yield "self: as it enters", s
        for phrase, fam in (("can't win the game", "can't win the game"), ("can't lose the game", "can't lose the game")):
            if phrase in low:
                yield fam, s
        if re.search(r"can't enter|can't be put onto the battlefield", low):
            yield "can't enter", s
        if re.search(r"\bregenerate\b", low) and "can't be regenerated" not in low:
            yield "regenerate", s
        for counter in ("shield", "stun", "finality"):
            if re.search(rf"\b{counter} counters?\b", low):
                yield f"{counter} counter", s
        if re.search(r"\b(?:becomes?|become) a copy of\b", low):
            yield "a copy: becomes a copy (a named copy row)", s
        elif re.search(r"\benters? as a copy of\b|\benter as a copy\b", low):
            yield "a copy: enters as a copy (filed as printed)", s
        elif re.search(r"\btokens? (?:that's|that are) (?:a )?cop(?:y|ies) of\b", low):
            yield "a copy: a token copy (filed as printed)", s
        if re.search(r"\b(?:has|have|gains?|gain)\b[^\"]*\"[^\"]*\b(?:instead|prevent)\b", s, re.I):
            yield "grant: a quoted replacement or prevention", s
        if re.search(r"\b(?:gains?|has|have)\b[^.]*\bprotection from\b", low):
            if re.search(r"^(?:creatures|other|each|all|permanents|artifacts|slivers)\b|\b(?:creatures|permanents) you control\b|\beach creature\b", low):
                yield "grant: protection, over a filter", s
            else:
                yield "grant: protection, to named objects", s
    for keyword in card.get("kw") or []:
        fam = KEYWORDS.get(keyword.lower())
        if fam:
            yield fam, "[keyword] " + keyword


def static_on_permanent(card, clause):
    if not PERMANENT.search(card.get("type") or ""):
        return False
    if clause.startswith("[keyword]"):
        return True
    for s, line in sentences(normalize(card)):
        if s != clause:
            continue
        at = line.find(s)
        if TRIGGERED.match(line) or '"' in line[:at] or re.search(r":\s", line[:at + len(s)].split('"')[0]):
            return False
        return not re.search(r"\bthis turn\b|\buntil end of turn\b|\bthe next\b|\bthe first time\b", s, re.I)
    return False


def main():
    if not os.path.exists(CORPUS):
        sys.exit("no corpus: run `python plans/references/cast-census.py` first (about seven minutes)")
    corpus = json.load(open(CORPUS, encoding="utf-8"))
    cards = {c["name"]: c for c in corpus["cards"]}
    families = defaultdict(dict)
    for card in cards.values():
        for fam, clause in classify(card):
            families[fam].setdefault(card["name"], clause)

    print(f"{corpus['query']}, fetched {corpus['fetched']}: {len(cards)} cards")
    print(f"{'cards':>6} {'static':>6}  family")
    for fam, hits in sorted(families.items()):
        static = sum(static_on_permanent(cards[n], c) for n, c in hits.items())
        print(f"{len(hits):6d} {static:6d}  {fam}")
        if "--names" in sys.argv:
            for name, clause in sorted(hits.items()):
                print(f"{'':15}{name} | {clause}")

    watchers = defaultdict(set)
    for fam, hits in families.items():
        if not fam.startswith(NOT_A_WATCHER):
            for name in hits:
                watchers[name].add(fam)
    self_only = sum(1 for fams in watchers.values() if fams <= SELF_ENTRY)
    any_self = sum(1 for fams in watchers.values() if fams & SELF_ENTRY)
    print(f"{len(watchers):6d}         cards with a replacement that watches an event")
    print(f"{any_self:6d}         of them, with one that modifies only its own entry")
    print(f"{self_only:6d}         of them, whose every replacement modifies only its own entry")


if __name__ == "__main__":
    main()
