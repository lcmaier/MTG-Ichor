"""The history census (`triggers-architecture.md` §12, TR-5h): how many
printed cards read the past by filtering or totalling over what the things
were at the time, which no count per player can answer.

Run: PYTHONIOENCODING=utf-8 python plans/references/history-census.py

Each phrasing is a Scryfall search; the union is taken locally, since one
query holding all eleven is refused. Every number is a lower bound: a card
that words the same reading differently is missed, never counted twice.
Counted 2026-10-19: 124 cards in the union.
"""
import json
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

UA = "MTG-Ichor-Research/1.0 (contact: maiercluke@gmail.com)"

PHRASINGS = [
    # spells cast, narrowed by what they were
    'o:/(noncreature|instant|sorcery|creature|artifact|enchantment|historic|legendary|other) spells? you.ve cast this turn/',
    # deaths counted by kind ("each Zubera that died this turn")
    'o:/(each|for each|number of) [a-z]+ that died this turn/',
    # an act counted by kind ("three or more Clues")
    'o:/(sacrificed|discarded|cycled|milled) (a|an|two|three|one or more|two or more|three or more) [a-z]+ this turn/',
    # damage counted by its sources' kind ("three or more Pirates")
    'o:/damage by (three|two) or more/',
    # tokens counted by kind ("Treasure tokens you created")
    'o:/tokens? you created this turn/',
    # objects put into a zone, by kind
    'o:/(artifacts|creatures|cards|permanents|lands) (that were )?put into [a-z ]+ this turn/',
    # totals and aggregates ("the total mana value of", "card types among")
    'o:/(total|greatest|among)[a-z ]+(spells|creatures|permanents|cards)[a-z ]+this turn/',
    # entries counted by kind (celebration)
    'o:/(two or more|three or more) [a-z ]*(permanents|creatures|lands|artifacts) entered/',
    # one event of a kind ("if an artifact entered ... this turn")
    'o:/(an artifact|a land|an enchantment|a nontoken|a nonland permanent) (entered|died|left)/ o:"this turn"',
    # a damage source as it was ("a Giant, Wizard, or spell you controlled")
    'o:/that dealt damage to [a-z ]+ this turn/ o:/(controlled|instant|sorcery|named)/',
    # an object's own first time ("the first time ... put on that permanent")
    'o:/first time [a-z +\\/1]+ (put on|become|been)/',
]


def names(query):
    url = "https://api.scryfall.com/cards/search?q=" + urllib.parse.quote(query)
    found = set()
    while url:
        request = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
        try:
            with urllib.request.urlopen(request) as response:
                page = json.load(response)
        except urllib.error.HTTPError as error:
            if error.code == 404:  # no card matches
                return found
            raise
        found |= {card["name"] for card in page["data"]}
        url = page.get("next_page")
        time.sleep(0.11)
    return found


def main():
    union = set()
    for query in PHRASINGS:
        cards = names(query)
        union |= cards
        print(f"{len(cards):5d}  {query}")
    print(f"{len(union):5d}  the union")
    if "--names" in sys.argv:
        for name in sorted(union):
            print("       " + name)


if __name__ == "__main__":
    main()
