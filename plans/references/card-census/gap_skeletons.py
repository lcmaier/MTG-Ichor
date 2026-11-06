"""Each deck card's printed face as a Rust builder, for the gap census's crate.

    python -I plans/references/card-census/gap_skeletons.py <cards.json> <out.rs>

`cards.json` is `pilot_fetch.py`'s output. For each unregistered card this writes
`b_<slug>()`, a `CardDataBuilder` holding everything but abilities, and
`T_<SLUG>`, the card's paragraphs, so a card's own code is its abilities alone.
A subtype the engine's enums lack fails the crate's compile, which is a finding.
"""

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent))
from check_state_of_play import deck_cards, registered_cards  # noqa: E402

COLORS = {"W": "White", "U": "Blue", "B": "Black", "R": "Red", "G": "Green"}
MANA = {**COLORS, "C": "Colorless"}
FLAGS = ["Deathtouch", "Defender", "Double strike", "First strike", "Flash", "Flying", "Haste", "Hexproof",
         "Indestructible", "Intimidate", "Lifelink", "Menace", "Reach", "Shroud", "Trample", "Vigilance"]
ENGINE_TYPES = {"Artifact", "Battle", "Creature", "Enchantment", "Instant", "Kindred", "Land", "Planeswalker", "Sorcery"}
SUPERTYPES = {"Basic", "Legendary", "Snow", "World"}


def slug(name):
    return re.sub(r"[^a-z0-9]+", "_", name.split(" // ")[0].lower().replace("'", "").replace(",", "")).strip("_")


def symbol(sym):
    if sym.isdigit():
        return ["ManaSymbol::Generic"] * int(sym)
    if sym == "X":
        return ["ManaSymbol::X"]
    if sym in MANA:
        return ["ManaSymbol::Colorless" if sym == "C" else f"ManaSymbol::Colored(ManaType::{MANA[sym]})"]
    parts = sym.split("/")
    if len(parts) == 2 and all(p in COLORS for p in parts):
        return [f"ManaSymbol::Hybrid(ManaType::{COLORS[parts[0]]}, ManaType::{COLORS[parts[1]]})"]
    if len(parts) == 2 and parts[0].isdigit():
        return [f"ManaSymbol::MonoHybrid(ManaType::{COLORS[parts[1]]})"]
    if len(parts) == 2 and parts[1] == "P":
        return [f"ManaSymbol::Phyrexian(ManaType::{COLORS[parts[0]]})"]
    raise ValueError(sym)


def subtype(card_types, word):
    camel = re.sub(r"[^A-Za-z]", "", word.title())
    if "Planeswalker" in card_types:
        return f"Subtype::Planeswalker(PlaneswalkerType::{camel})"
    if "Creature" in card_types or "Kindred" in card_types:
        return f"Subtype::Creature(CreatureType::{camel})"
    if "Land" in card_types:
        return f"Subtype::Land(LandType::{camel})"
    if "Artifact" in card_types:
        return f"Subtype::Artifact(ArtifactType::{camel})"
    if "Enchantment" in card_types:
        return f"Subtype::Enchantment(EnchantmentType::{camel})"
    return f"Subtype::Spell(SpellType::{camel})"


def printed(text):
    """Oracle text less reminder text, which has no game function (CR 207.2) and
    no registered card's rules text carries; a paragraph of nothing else goes."""
    text = re.sub(r"\s*\([^()]*\)", "", text)
    return "\n".join(line.strip() for line in text.split("\n") if line.strip())


def paragraphs(text):
    """`card_data::paragraphs`: a modal ability's bullets stay with it."""
    return re.split(r"\n(?!•)", text) if text else []


def skeleton(card):
    face = (card.get("card_faces") or [card])[0]
    name = face["name"]
    left, _, right = face["type_line"].partition(" — ")
    words = left.split()
    types = [w for w in words if w in ENGINE_TYPES]
    lines = [f'CardDataBuilder::new({json.dumps(name, ensure_ascii=False)})']
    symbols = [s for sym in re.findall(r"{([^}]+)}", face.get("mana_cost", "")) for s in symbol(sym)]
    if symbols:
        lines.append(f".mana_cost(ManaCost::from_symbols(vec![{', '.join(symbols)}]))")
    for color in face.get("colors", card.get("colors", [])):
        lines.append(f".color(Color::{COLORS[color]})")
    for word in words:
        if word in SUPERTYPES:
            lines.append(f".supertype(Supertype::{word})")
    for word in types:
        lines.append(f".card_type(CardType::{word})")
    for word in right.split():
        lines.append(f".subtype({subtype(types, word)})")
    if "power" in face and face["power"].lstrip("-").isdigit():
        lines.append(f".power_toughness({face['power']}, {face['toughness']})")
    elif "power" in face:
        lines.append(".power_toughness(0, 0)")
    if "loyalty" in face:
        lines.append(f".loyalty({face['loyalty']})")
    for flag in FLAGS:
        if flag in card.get("keywords", []):
            lines.append(f".keyword_flag(KeywordFlag::{flag.title().replace(' ', '')})")
    text = printed("\n".join(f.get("oracle_text", "") for f in (card.get("card_faces") or [card])[:1]))
    lines.append(f".rules_text({json.dumps(text, ensure_ascii=False)})")
    parts = paragraphs(text)
    upper = slug(name).upper()
    consts = f"pub const T_{upper}: [&str; {len(parts)}] = [{', '.join(json.dumps(p, ensure_ascii=False) for p in parts)}];"
    return f"{consts}\npub fn b_{slug(name)}() -> CardDataBuilder {{\n    {chr(10).join('    ' + l if i else l for i, l in enumerate(lines))}\n}}\n"


HEADER = """// GENERATED by gap_skeletons.py from Scryfall's Oracle text.
#![allow(dead_code)]
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::types::card_types::*;
use mtgsim::types::colors::Color;
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaSymbol, ManaType};

"""


def main():
    cards = json.load(open(sys.argv[1], encoding="utf-8"))
    names = sorted(deck_cards() - registered_cards())
    body = "\n".join(skeleton(cards[n]) for n in names)
    Path(sys.argv[2]).write_text(HEADER + body, encoding="utf-8")
    print(f"{len(names)} skeletons")


if __name__ == "__main__":
    main()
