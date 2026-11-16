"""What the printed cards ask of the information model: who is shown a card, and what stays hidden.

    python plans/references/information-census.py                  # the table
    python plans/references/information-census.py --names FAMILY   # one row's cards
    python plans/references/information-census.py --names FAMILY --sample 7   # twelve of them, seeded

Behind `information-model-design`'s note (`plans/handoffs/information-model-design.md`).
The corpus is `cast-census.py`'s cache, `.census-cast.json`: every
Commander-legal paper card, fetched 2026-10-05. The `tdc` column is the cards
of `tdc-names.txt`, the five Tarkir: Dragonstorm Commander lists.

WHAT IT COUNTS
--------------
A card by the clause that does the thing, reminder text stripped, never by a
word anywhere in its text. The families are grouped by what the model must
do, which is the note's table:

- shown: a card's face shown to every player (reveal), or to one (look at),
  once as an effect applies or for as long as an effect lasts;
- moved unseen: a card that goes from one hidden place to another without
  any other player seeing which card it was;
- face down: a card whose face is hidden while it sits in a public zone;
- secret: a choice made that the other players are not told yet;
- random: a draw from the game's random stream, by its kind.

A keyword is counted off Scryfall's keyword list for the card (`kw`), since a
keyword's own clause is its reminder text. A word match is approximate: every
family under 40 was read by hand at the design (2026-11-15), and twelve of
each larger one, seeded.
"""
import json
import os
import random
import re
import sys
from collections import OrderedDict

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, ".census-cast.json")
DECK = os.path.join(HERE, "tdc-names.txt")
REMINDER = re.compile(r"\([^()]*\)")

R = lambda p: re.compile(p, re.I)


def kw(*names):
    wanted = {n.lower() for n in names}
    return lambda c, t: any(k.lower() in wanted for k in c["kw"])


def text(pattern, unless=None):
    found, refused = R(pattern), R(unless) if unless else None
    return lambda c, t: bool(found.search(t)) and not (refused and refused.search(t))


def either(*tests):
    return lambda c, t: any(test(c, t) for test in tests)


def both(*tests):
    return lambda c, t: all(test(c, t) for test in tests)


HIDDEN_PLACE = r"(?:hand|library|libraries|hands)"

FAMILIES = OrderedDict()

# --- shown to every player ---------------------------------------------------
FAMILIES["reveal: any card shown to all as an effect applies or a cost is paid"] = text(r"\breveals?\b")
FAMILIES["...a hand revealed"] = text(r"\breveals? (?:their|your|his or her|its controller's|that player's) hand\b|\breveal (?:their|your) hands\b|\beach (?:player|opponent) reveals (?:their|his or her) hand\b")
FAMILIES["...the top of a library revealed, once"] = text(r"\breveals? the top (?:\w+ )?cards? of\b|\breveals? cards from the top of\b")
FAMILIES["...revealed until a card matches (a library walk)"] = text(r"\breveals? cards from the top of [^.]*? until\b|\breveals? cards from [^.]*? library until\b")
FAMILIES["...a searched-for card revealed"] = text(r"\bsearch(?:es)? [^.]*? library for [^.]*?\breveal")
FAMILIES["...revealed as a cost or as it is cast (behold, forecast, 'reveal ... from your hand')"] = text(r"\breveal [^.:]*? from your hand\b|\bas an additional cost[^.]*\breveal\b")
FAMILIES["plays with a hand revealed (for as long as an effect lasts)"] = text(r"\bplays? with (?:their|your|his or her) hands? revealed\b")
FAMILIES["plays with the top card of a library revealed"] = text(r"\bplays? with the top card of (?:their|your|his or her) librar(?:y|ies) revealed\b")
FAMILIES["a card revealed as it is drawn (miracle, and 'reveal it' after a draw)"] = either(
    kw("Miracle"), text(r"\bdraws? a card[^.]*?(?:\.|,| and) (?:then )?(?:you may )?reveals? (?:it|that card)\b|\breveals? (?:each|the first) card (?:they|you) draws?\b|\bas (?:you|they) draw[^.]*reveal"))

# --- shown to one player -----------------------------------------------------
FAMILIES["look at: any card shown to one player"] = either(
    text(r"\blooks? at\b"), kw("Scry", "Surveil", "Hideaway", "Explore", "Fateseal", "Clash"))
FAMILIES["...scry or surveil (the top of your own library)"] = either(kw("Scry", "Surveil"), text(r"\bscry \w+\b|\bsurveil \w+\b"))
FAMILIES["...look at the top of your own library, other than scry and surveil"] = text(
    r"\blooks? at the top (?:\w+ )?cards? of (?:your|their) library\b")
FAMILIES["...look at another player's hand"] = text(
    r"\blooks? at (?:target (?:player|opponent)'s|that player's|an opponent's|each opponent's|their|his or her) hand\b")
FAMILIES["...look at the top of another player's library"] = text(
    r"\blooks? at the top (?:\w+ )?cards? of (?:target (?:player|opponent)'s|that player's|an opponent's|each opponent's|each player's|its owner's) library\b")
FAMILIES["...look at a face-down card someone else controls or owns"] = text(
    r"\blooks? at (?:any |target |all |each )?face-down (?:creatures?|permanents?|cards?|spells?)\b[^.]*?(?:you don't control|opponents? controls?|target|any time)|\blooks? at (?:that|those|the exiled) cards?\b")
FAMILIES["may look at the top card of a library any time (for as long as an effect lasts)"] = text(
    r"\bmay look at the top card of (?:your|their) library any time\b|\blook at the top card of your library at any time\b")
FAMILIES["search: a whole library or hand looked at (CR 701.23a)"] = text(r"\bsearch(?:es)? ")
FAMILIES["...search another player's library, hand or graveyard-and-hand"] = text(
    r"\bsearch(?:es)? (?:target (?:player|opponent)'s|that player's|an opponent's|its controller's|their|each opponent's|its owner's) (?:graveyard, hand,? and )?" + HIDDEN_PLACE)

# --- moved unseen: hidden to hidden -------------------------------------------
FAMILIES["a card put from a hand into a library, not revealed"] = text(
    r"\bputs? [^.]*?\bfrom (?:your|their|his or her) hand on(?:to)? (?:the )?(?:top|bottom) of\b|\bputs? (?:\w+ |any number of )?cards? from (?:your|their) hand on (?:top|the bottom)\b"
    r"|\bshuffles? (?:\w+ )?cards? from (?:your|their) hand into\b|\bshuffles? (?:your|their) hand into\b|\bputs? (?:your|their) hand on (?:top|the bottom)\b",
    unless=r"\breveal [^.]*?\bfrom your hand")
FAMILIES["a card put from a library into a hand, not revealed (tutor, 'put one into your hand')"] = text(
    r"\bputs? (?:it|that card|them|one of them|one|those cards|the rest|\w+ of them) into (?:your|their|its owner's) hand\b",
    unless=r"\breveals?\b")
FAMILIES["the order of cards put into a library is the owner's secret (CR 401.4): 'in any order'"] = text(
    r"(?:top|bottom) of (?:your|their|its owner's|that player's|his or her) library in any order\b|\bin any order\b[^.]*\blibrary\b")
FAMILIES["cards put on the bottom of a library in a random order"] = text(r"\bbottom of (?:your|their|its owner's|that player's) library in a random order\b")

# --- face down ----------------------------------------------------------------
FAMILIES["cast face down: morph, megamorph, disguise"] = kw("Morph", "Megamorph", "Disguise")
FAMILIES["put onto the battlefield face down: manifest, manifest dread, cloak"] = either(
    kw("Manifest", "Manifest dread", "Cloak"), text(r"\bmanifests?\b|\bcloaks?\b"))
FAMILIES["another way onto the battlefield or the stack face down, or turned face down"] = text(
    r"\b(?:onto the battlefield|enters?) face down\b|\bturn(?:s|ed)? [^.]*? face down\b|\bface-down (?:creature|permanent|spell)s?\b",
    unless=r"\bmanifests?\b|\bcloaks?\b")
FAMILIES["exiled face down: any"] = either(kw("Foretell", "Hideaway"), text(r"\bexiles? [^.]*?\bface down\b|\bface-down exiled\b|\bexiled face down\b"))
FAMILIES["...foretell"] = kw("Foretell")
FAMILIES["...hideaway"] = kw("Hideaway")
FAMILIES["...exiled face down, then 'you may look at' or 'may play/cast' it (CR 406.3's grant)"] = text(
    r"\bexiles? [^.]*?\bface down\b[^|]*?\b(?:may look at|may play|may cast|look at (?:that|those|the exiled))")
FAMILIES["...exiled face down from another player's hand or library"] = text(
    r"\bexiles? [^.]*?(?:target (?:player|opponent)'s|that player's|an opponent's|each opponent's|each player's|their) (?:hand|library)[^.]*?\bface down\b"
    r"|\bexiles? [^.]*?\bface down\b[^.]*?(?:target (?:player|opponent)'s|that player's|an opponent's|each opponent's) (?:hand|library)"
    r"|\beach (?:player|opponent) exiles [^.]*?\bface down\b")
FAMILIES["a face-down pile chosen from at random (CR 406.4)"] = text(r"\bface down\b[^|]*?\bat random\b")

# --- secret -------------------------------------------------------------------
FAMILIES["a choice made in secret, then revealed"] = text(r"\bsecretly\b")
FAMILIES["a vote (CR 701.38): open, in turn order"] = text(r"\bvotes?\b")
FAMILIES["a card name chosen (CR 201.4)"] = text(r"\b(?:choose|name)s? a (?:[\w-]+ )*card name\b|\bname a (?:[\w-]+ )*card\b")

# --- random -------------------------------------------------------------------
FAMILIES["shuffle"] = text(r"\bshuffles?\b")
FAMILIES["a card or object taken at random (discard, exile, return, choose)"] = text(r"\bat random\b", unless=r"\b(?:player|opponent)s? (?:chosen )?at random\b")
FAMILIES["a player or opponent chosen at random"] = text(r"\b(?:player|opponent)s? (?:chosen )?at random\b|\bchoose an? (?:player|opponent) at random\b")
FAMILIES["a random order"] = text(r"\brandom order\b")
FAMILIES["a coin flipped"] = text(r"\bflips? (?:a|\w+) coins?\b|\bcoin flip\b")
FAMILIES["a die rolled"] = text(r"\brolls? (?:a|\w+|x) (?:d\d+|six-sided|twenty-sided|\w+-sided|dice|die)\b|\brolls? (?:a |\w+ )?d\d+\b")

# --- the rules that read sight --------------------------------------------------
FAMILIES["an ability that works from a hand or a library and is seen as it does (CR 603.2f, 602.2a)"] = text(
    r"\bwhen you reveal\b|\bwhen (?:you|a player) (?:draws?|cycles?|discards?) (?:this card|~)\b|\bwhile (?:this card|~) is in your (?:hand|library)\b|\bfrom your hand\b[^.]*:\s|\b(?:reveal|discard) (?:this card|~) from your hand\b[^.:]*:")
FAMILIES["a discard put into a library instead of the graveyard (CR 701.9c's board)"] = text(
    r"\bcauses you to discard a card\b[^.]*?\bon top of your library instead\b")
FAMILIES["a lasting 'may look at cards exiled with' this permanent (sight by a link, CR 406.3, 607)"] = text(
    r"\bmay look at (?:the )?cards exiled with\b")
FAMILIES["a card put a counted place from the top of a library"] = text(
    r"\b(?:second|third|fourth|fifth|sixth|seventh|\w+th) from the top\b")


def load():
    with open(CORPUS, encoding="utf-8") as f:
        data = json.load(f)
    return data["fetched"], data["cards"]


def main(argv):
    fetched, cards = load()
    with open(DECK, encoding="utf-8") as f:
        deck = {line.strip() for line in f if line.strip()}
    texts = [(c, REMINDER.sub("", c.get("text") or "")) for c in cards]
    if "--names" in argv:
        want = argv[argv.index("--names") + 1]
        hits = [(c, t) for c, t in texts if FAMILIES[want](c, t)]
        if "--tdc" in argv:
            hits = [(c, t) for c, t in hits if c["name"] in deck]
        if "--sample" in argv:
            hits = random.Random(int(argv[argv.index("--sample") + 1])).sample(hits, min(12, len(hits)))
        for c, t in hits:
            print(c["name"], "|", t.replace("\n", " / ")[:260])
        return
    print(f"corpus fetched {fetched}, {len(cards)} cards; tdc lists {len(deck)} names")
    print(f"{'cards':>6} {'tdc':>4}  family")
    for name, test in FAMILIES.items():
        hits = [c["name"] for c, t in texts if test(c, t)]
        in_deck = sum(1 for n in hits if n in deck)
        print(f"{len(hits):>6} {in_deck:>4}  {name}")


if __name__ == "__main__":
    main(sys.argv[1:])
