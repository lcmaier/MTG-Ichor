"""Parse Forge card scripts into each card's set of needs.

usage: parse_forge.py <cardsfolder> <out.json>

A need is a "category:name" token:
  api:<Api>        an effect (SP$/AB$/DB$), a few refined by their parameters
  trig:<Mode>      a trigger mode, ChangesZone and Phase refined
  static:<Mode>    a static ability mode; Continuous split by what it changes
  repl:<Event>     a replacement event
  kw:<Keyword>     a keyword ability
  cost:<Part>      a non-mana cost part of an activated ability or spell
  mana:<Kind>      a mana-cost shape: X, hybrid, phyrexian, snow
  layout:<Mode>    a multi-face layout
  param:<Name>     a parameter that names a mechanism of its own ("unless", divided, "may", ...)
  amount:<Count>   a Count$ dynamic amount
"""
import json
import os
import re
import sys

root, out = sys.argv[1], sys.argv[2]

FEATURE_PARAMS = {
    "UnlessCost": "unless_pays",
    "DividedAsYouChoose": "divided",
    "OptionalDecider": "may",
    "Optional": "may",
    "XChoice": "x_choice",
    "AnnounceType": "announce",
    "ChoiceZone": "choose_from_zone",
    "Chooser": "other_player_chooses",
    "Planeswalker": "loyalty_ability",
    "Ultimate": "loyalty_ability",
    "ActivationLimit": "activation_limit",
    "SorcerySpeed": "sorcery_speed",
    "IsPresent": "condition_present",
    "ConditionPresent": "condition_present",
    "CheckSVar": "condition_count",
    "ConditionCheckSVar": "condition_count",
    "RememberObjects": "remember",
    "RememberChanged": "remember",
    "Imprint": "imprint",
    "ChosenColor": "chosen_color",
    "ChooseColor": "chosen_color",
    "Duration": None,  # refined below
}
CONTINUOUS_PARTS = [
    (("AddPower", "AddToughness", "SetPower", "SetToughness"), "pt"),
    (("AddKeyword", "RemoveKeyword", "RemoveAllAbilities"), "keywords"),
    (("AddAbility", "AddTrigger", "AddStaticAbility", "AddReplacementEffects", "GainTextOf"), "grant_ability"),
    (("AddType", "RemoveType", "AddAllCreatureTypes", "RemoveCardTypes", "RemoveSubTypes", "RemoveSuperTypes", "RemoveCreatureTypes", "RemoveLandTypes"), "types"),
    (("SetColor", "AddColor"), "color"),
    (("GainControl",), "control"),
    (("CharacteristicDefining",), "cda"),
    (("MayPlay", "MayLookAt", "MayPlayWithoutManaCost", "MayPlayAltManaCost"), "may_play"),
    (("AddHiddenKeyword",), "hidden_keyword"),
]
COST_PARTS = re.compile(r"\b([A-Za-z]+)<")


def params(body):
    d = {}
    for part in body.split("|"):
        part = part.strip()
        if "$" in part:
            k, v = part.split("$", 1)
            d[k.strip()] = v.strip()
    return d


def mana_symbol(t, needs):
    """Forge writes hybrid as `BG`, twobrid as `2W`, Phyrexian with a `P`."""
    if t == "X":
        needs.add("mana:X")
    elif re.fullmatch(r"(P[WUBRG]{1,2}|[WUBRG]{1,2}P)", t):
        needs.add("mana:phyrexian")
    elif re.fullmatch(r"[WUBRG]{2}|2[WUBRG]|C[WUBRG]", t):
        needs.add("mana:hybrid")
    elif t == "S":
        needs.add("mana:snow")
    else:
        return False
    return True


def cost_tokens(cost, needs):
    for m in COST_PARTS.finditer(cost):
        needs.add(f"cost:{m.group(1)}")
    toks = re.sub(r"[A-Za-z]+<[^>]*>", " ", cost).split()
    for t in toks:
        if t == "T":
            needs.add("cost:Tap")
        elif t == "Q":
            needs.add("cost:Untap")
        elif mana_symbol(t, needs):
            pass
        elif t == "E":
            needs.add("cost:Energy")


def ability_tokens(kind, body, needs):
    p = params(body)
    api = p.get(kind)
    if api:
        name = api
        if api in ("ChangeZone", "ChangeZoneAll"):
            name = f"{api}[{p.get('Origin', '?')}->{p.get('Destination', '?')}{',hidden' if p.get('Hidden') else ''}]"
        elif api in ("Pump", "PumpAll") and "KW" in p and not any(k in p for k in ("NumAtt", "NumDef")):
            name = f"{api}[keyword]"
        elif api == "Mana" and re.search(r"Any|Combo|Chosen", p.get("Produced", "")):
            name = "Mana[any_or_chosen_color]"
        needs.add(f"api:{name}")
    if "Cost" in p:
        cost_tokens(p["Cost"], needs)
    # A token's own abilities live in tokenscripts/: a Treasure's needs are its maker's.
    for t in p.get("TokenScript", "").split(","):
        if t.strip():
            needs.update(token_needs(t.strip()))
    granted_keywords(p, ("KW", "Keywords"), needs)
    for k, v in p.items():
        if k in FEATURE_PARAMS and FEATURE_PARAMS[k]:
            needs.add(f"param:{FEATURE_PARAMS[k]}")
        if k == "Duration" and v not in ("UntilEndOfTurn",):
            needs.add(f"param:duration_{v}")
        if k == "Defined" and "Remembered" in v:
            needs.add("param:remember")
        if v.startswith("Count$"):
            needs.add("amount:" + re.split(r"[ .<>/]", v[6:])[0])


def trig_tokens(body, needs):
    p = params(body)
    mode = p.get("Mode", "?")
    if mode == "ChangesZone":
        o, d = p.get("Origin", "Any"), p.get("Destination", "Any")
        if d == "Battlefield":
            mode = "ChangesZone[enters]"
        elif o == "Battlefield" and d == "Graveyard":
            mode = "ChangesZone[dies]"
        elif o == "Battlefield":
            mode = "ChangesZone[leaves]"
        else:
            mode = f"ChangesZone[{o}->{d}]"
    elif mode == "Phase":
        mode = f"Phase[{p.get('Phase', '?')}]"
    needs.add(f"trig:{mode}")
    if p.get("OptionalDecider"):
        needs.add("param:may")
    if "IsPresent" in p or "CheckSVar" in p:
        needs.add("param:intervening_if")


KW_FAMILY = re.compile(
    r"^(Protection|Enchant|Landwalk|Hexproof|Ward|Equip|Cycling|Kicker|Flashback|Bushido|Bands with other|"
    r"Partner with|Affinity|Champion|Splice|Typecycling|Dredge|Madness|Morph|Megamorph|Suspend|Prototype|Offspring)\b.*")


def keyword_name(kw):
    kw = re.split(r"[:]", kw)[0]
    kw = KW_FAMILY.sub(r"\1", kw)
    kw = re.sub(r"\s+\d+$", "", kw)
    if kw.endswith("walk"):
        kw = "Landwalk"
    return kw.strip()


def granted_keywords(p, keys, needs):
    """A keyword a pump or a static grants needs the engine to know it, as a
    printed one does; Forge's `HIDDEN` keywords are rules text ("can't block")."""
    for key in keys:
        for kw in re.split(r"\s*&\s*", p.get(key, "")):
            kw = kw.strip()
            if not kw:
                continue
            if kw.startswith("HIDDEN "):
                text = re.sub(r"\s+", " ", kw[7:].replace("CARDNAME", "~")).strip(" .")
                needs.add(f"hidden:{text}")
            else:
                needs.add(f"kw:{keyword_name(kw)}")


def static_tokens(body, needs):
    p = params(body)
    granted_keywords(p, ("AddKeyword", "AddHiddenKeyword"), needs)
    if "Cost" in p:
        cost_tokens(p["Cost"], needs)  # an alternative or optional cost's parts
    mode = p.get("Mode", "?")
    if mode == "Continuous":
        parts = [label for keys, label in CONTINUOUS_PARTS if any(k in p for k in keys)]
        for label in parts or ["other"]:
            needs.add(f"static:Continuous[{label}]")
    else:
        needs.add(f"static:{mode}")


TOKEN_ROOT = os.path.join(os.path.dirname(os.path.normpath(root)), "tokenscripts")
_token_cache = {}


def token_needs(name):
    if name not in _token_cache:
        _token_cache[name] = set()  # a token that makes itself stops here
        path = os.path.join(TOKEN_ROOT, name + ".txt")
        if os.path.exists(path):
            _token_cache[name] = {n for n in parse(path)["needs"] if not n.startswith("ability:")}
        else:
            _token_cache[name] = {"token:unknown_script"}
    return _token_cache[name]


def parse(path):
    text = open(path, encoding="utf8", errors="replace").read()
    card = {"name": None, "needs": set(), "faces": 1, "spells_on_face": 0}
    for raw in text.splitlines():
        line = raw.strip()
        if line == "ALTERNATE":
            card["faces"] += 1
            card["spells_on_face"] = 0
            continue
        if ":" not in line:
            continue
        key, rest = line.split(":", 1)
        needs = card["needs"]
        if key in ("Oracle", "Types", "ManaCost", "PT") and card["faces"] == 1:
            card[key.lower()] = rest
        if key == "Name" and card["name"] is None:
            card["name"] = rest
        elif key == "ManaCost":
            for t in rest.split():
                mana_symbol(t, needs)
        elif key == "AlternateMode":
            needs.add(f"layout:{rest}")
        elif key == "K":
            kw = re.split(r"[:]", rest)[0]
            kw = re.sub(r"^(Protection|Enchant|Landwalk|Hexproof|Ward|Equip|Cycling|Kicker|Flashback|Bushido|Bands with other|Partner with|Affinity|Champion|Splice|Typecycling|Dredge|Madness|Morph|Megamorph|Suspend|Prototype|Offspring)\b.*", r"\1", kw)
            kw = re.sub(r"\s+\d+$", "", kw)
            if kw.endswith("walk"):
                kw = "Landwalk"
            needs.add(f"kw:{kw.strip()}")
        elif key == "A":
            for kind in ("SP", "AB"):
                if rest.startswith(kind + "$"):
                    ability_tokens(kind, rest, needs)
                    needs.add("ability:activated" if kind == "AB" else "ability:spell")
            if rest.startswith("SP$"):
                # A second spell ability on one face is another way to cast it:
                # cleave, overload, and the like, named by its PrecostDesc.
                card["spells_on_face"] += 1
                if card["spells_on_face"] > 1:
                    alt = params(rest).get("PrecostDesc", "").strip(" .:") or "second_spell_ability"
                    needs.add(f"alt:{alt}")
            zone = params(rest).get("ActivationZone")
            if zone and zone != "Battlefield":
                needs.add(f"param:activated_from_{zone}")
        elif key == "T":
            trig_tokens(rest, needs)
        elif key == "S":
            static_tokens(rest, needs)
        elif key == "R":
            needs.add(f"repl:{params(rest).get('Event', '?')}")
            ability_tokens("ReplaceWith", "", needs)
        elif key == "SVar":
            if ":" in rest:
                _, body = rest.split(":", 1)
                if body.startswith("DB$") or body.startswith("AB$"):
                    ability_tokens(body[:2], body, needs)
                elif body.startswith("Count$"):
                    needs.add("amount:" + re.split(r"[ .<>/]", body[6:])[0])
                elif body.startswith("Mode$"):
                    # A trigger or static written as an SVar: what an `Effect`
                    # creates, or what a static or an animation grants.
                    if re.search(r"Execute\$|TriggerZones\$|TriggerDescription\$", body):
                        trig_tokens(body, needs)
                    else:
                        static_tokens(body, needs)
                elif body.startswith("Event$"):
                    needs.add(f"repl:{params(body).get('Event', '?')}")
    card["needs"] = sorted(card["needs"])
    del card["spells_on_face"]
    return card


cards = []
for dirpath, _, files in os.walk(root):
    for f in files:
        if f.endswith(".txt"):
            c = parse(os.path.join(dirpath, f))
            c["path"] = os.path.relpath(os.path.join(dirpath, f), root).replace(os.sep, "/")
            cards.append(c)
json.dump(cards, open(out, "w", encoding="utf8"), indent=0)
print(f"parsed {len(cards)} scripts")
