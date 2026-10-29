"""Forge need -> engine status, first pass (pilot, 2026-10-26).

B            built: the engine has a type and a path for it
F            free: Forge bookkeeping with no rules content
D            digital-only or out of scope (Alchemy, Planechase)
M:<owner>    missing; <owner> is the roadmap step that builds it
U            unclear: needs a closer look before it can be called

Anything not listed is U. The verdicts read the engine's enums and a grep,
not a test: a B says a surface exists, not that every card using it works.
"""

B, F, D, U = "B", "F", "D", "U"


def M(owner):
    return f"M:{owner}"


MAP = {
    # abilities
    "ability:activated": B, "ability:spell": B,
    # triggers (TR-1..TR-7)
    "trig:ChangesZone[enters]": B, "trig:ChangesZone[dies]": B, "trig:ChangesZone[leaves]": B,
    "trig:ChangesZone[Any->Graveyard]": B, "trig:ChangesZone[Hand->Graveyard]": B,
    "trig:SpellCast": B, "trig:Attacks": B, "trig:AttackersDeclared": B, "trig:AttackerBlocked": B,
    "trig:Blocks": B, "trig:Phase[Upkeep]": B, "trig:Phase[End of Turn]": B, "trig:Phase[BeginCombat]": B,
    "trig:Phase[Draw]": B, "trig:Phase[Main1]": B, "trig:Phase[Main2]": B, "trig:Phase[Cleanup]": B,
    "trig:DamageDone": B, "trig:DamageDoneOnce": B, "trig:Drawn": B, "trig:LifeGained": B, "trig:LifeLost": B,
    "trig:BecomesTarget": B, "trig:Discarded": B, "trig:Taps": B, "trig:Untaps": B,
    "trig:CounterAdded": B, "trig:CounterAddedOnce": B, "trig:CounterRemoved": B, "trig:TokenCreated": B,
    "trig:Countered": B, "trig:AbilityCast": B, "trig:Attached": B, "trig:Unattach": B,
    "trig:ChangesController": B, "trig:Scry": B, "trig:Shuffled": B, "trig:ChangesZoneAll": B,
    "trig:NewTurn": B, "trig:LosesGame": B, "trig:Always": B,
    "trig:ChaosEnsues": D, "trig:PlanarDice": D, "trig:PlaneswalkedTo": D,
    "trig:Transformed": M("double-faced-cards"), "trig:TurnFaceUp": M("face-down-and-morph"),
    "trig:Exploited": U, "trig:Cycled": M("remaining-costs"), "trig:RolledDie": M("backlog: dice"),
    # effects
    "api:Draw": B, "api:PutCounter": B, "api:PutCounterAll": B, "api:RemoveCounter": B,
    "api:DealDamage": B, "api:DamageAll": B, "api:Cleanup": F, "api:Token": B, "api:Pump": B,
    "api:Pump[keyword]": B, "api:PumpAll": B, "api:PumpAll[keyword]": B, "api:Mana": B,
    "api:Mana[any_or_chosen_color]": M("any-color-plus-painlands-plus-signets"),
    "api:GainLife": B, "api:LoseLife": B, "api:SetLife": B, "api:Tap": B, "api:TapAll": B,
    "api:Untap": B, "api:UntapAll": B, "api:Sacrifice": B, "api:SacrificeAll": B,
    "api:Destroy": B, "api:DestroyAll": B, "api:Discard": B, "api:Animate": B, "api:AnimateAll": B,
    "api:Mill": B, "api:Scry": B, "api:Surveil": B, "api:Counter": B, "api:Regenerate": B,
    "api:DelayedTrigger": B, "api:ImmediateTrigger": B, "api:Clone": B, "api:GainControl": B,
    "api:ReplaceEffect": B, "api:Fight": B, "api:Attach": B, "api:AddTurn": B, "api:AddPhase": B,
    "api:LosesGame": B, "api:WinsGame": B, "api:ChangeZoneAll[Battlefield->Exile]": B,
    "api:ChangeZoneAll[Graveyard->Exile]": B, "api:ChangeZoneAll[Battlefield->Hand]": B,
    "api:ChangeZoneAll[Battlefield->Graveyard]": B, "api:ChangeZoneAll[Hand->Graveyard]": B,
    "api:ChangeZone[Battlefield->Exile]": B, "api:ChangeZone[Battlefield->Hand]": B,
    "api:ChangeZone[Battlefield->Library]": B, "api:ChangeZone[Battlefield->Graveyard]": B,
    "api:ChangeZone[Graveyard->Battlefield]": B, "api:ChangeZone[Graveyard->Battlefield,hidden]": B,
    "api:ChangeZone[Graveyard->Hand]": B, "api:ChangeZone[Graveyard->Hand,hidden]": B,
    "api:ChangeZone[Graveyard->Exile]": B, "api:ChangeZone[Graveyard->Exile,hidden]": B,
    "api:ChangeZone[Graveyard->Library]": B, "api:ChangeZone[Exile->Battlefield]": B,
    "api:ChangeZone[Exile->Hand]": B,
    # hidden-zone reads: search, look at the top N
    "api:ChangeZone[Library->Hand,hidden]": M("information-model"),
    "api:ChangeZone[Library->Battlefield,hidden]": M("information-model"),
    "api:ChangeZone[Library->Graveyard,hidden]": M("information-model"),
    "api:Dig": M("information-model"), "api:DigUntil": M("information-model"),
    "api:RevealHand": M("information-model"), "api:Reveal": M("information-model"),
    "api:PeekAndReveal": M("information-model"), "api:LookAt": M("information-model"),
    "api:RearrangeTopOfLibrary": M("information-model"),
    # copies, faces, casting from elsewhere
    "api:CopyPermanent": M("token-copies"), "api:CopySpellAbility": M("spell-copies"),
    "api:SetState": M("double-faced-cards"), "api:Play": M("casting-from-other-zones"),
    "api:Manifest": M("face-down-and-morph"), "api:Cloak": M("face-down-and-morph"),
    # unscheduled or the route review's
    "api:Charm": M("modes-plus-choice-kinds: modal"), "api:GenericChoice": M("modes-plus-choice-kinds: modal"),
    "api:RepeatEach": M("backlog: for-each"), "api:Repeat": M("backlog: repeat"),
    "api:Goad": M("combat-requirements"), "api:RollDice": M("backlog: dice"),
    "api:FlipACoin": M("backlog: coins"), "api:NameCard": M("backlog: choose a name"),
    "api:Vote": M("backlog: voting"), "api:Venture": M("backlog: dungeons"),
    "api:Monarch": M("multiplayer-rules"), "api:Initiative": M("backlog: dungeons"),
    "api:MakeCard": D, "api:Planeswalk": D, "api:Conjure": D, "api:Seek": D,
    # statics
    "static:Continuous[pt]": B, "static:Continuous[keywords]": B, "static:Continuous[grant_ability]": B,
    "static:Continuous[types]": B, "static:Continuous[color]": B, "static:Continuous[control]": B,
    "static:Continuous[cda]": B, "static:ReduceCost": B, "static:RaiseCost": B, "static:SetCost": B,
    "static:Continuous[may_play]": M("casting-from-other-zones"),
    "static:CantTarget": M("cant-be-targeted"),
    "static:CantBlockBy": M("combat-restrictions"), "static:CantAttack": M("combat-restrictions"),
    "static:CantBlock": M("combat-restrictions"), "static:CantAttackUnless": M("combat-restrictions"),
    "static:CantBlockUnless": M("combat-restrictions"), "static:MinMaxBlocker": M("combat-restrictions"),
    "static:MustAttack": M("combat-requirements"), "static:MustBlock": M("combat-requirements"),
    "static:CantBeCast": M("cant-cast-play-or-activate"), "static:CantBeActivated": M("cant-cast-play-or-activate"),
    "static:CantPlayLand": M("cant-cast-play-or-activate"),
    "static:CantGainLosePayLife": B, "static:CantPutCounter": B, "static:CantDraw": B,
    "static:CastWithFlash": M("as-though-effects"), "static:MayPlay": M("casting-from-other-zones"),
    # replacement (RA-RH)
    "repl:Moved": B, "repl:Counter": B, "repl:Untap": B, "repl:Tap": B, "repl:DamageDone": B,
    "repl:GameLoss": B, "repl:GameWin": B, "repl:Draw": B, "repl:DrawCards": B, "repl:GainLife": B,
    "repl:LifeReduced": B, "repl:Destroy": B, "repl:CreateToken": B, "repl:Scry": B,
    "repl:ProduceMana": B, "repl:BeginPhase": B, "repl:BeginTurn": B, "repl:AddCounter": B,
    "repl:RemoveCounter": B, "repl:Discard": B, "repl:Mill": B,
    # keywords
    "kw:Flying": B, "kw:Trample": B, "kw:Haste": B, "kw:Vigilance": B, "kw:First Strike": B,
    "kw:Double Strike": B, "kw:Flash": B, "kw:Reach": B, "kw:Lifelink": B, "kw:Menace": B,
    "kw:Deathtouch": B, "kw:Defender": B, "kw:Hexproof": B, "kw:Shroud": B, "kw:Indestructible": B,
    "kw:Intimidate": B, "kw:Enchant": B, "kw:Equip": B, "kw:MayEffectFromOpeningHand": B,
    "kw:etbCounter": B, "kw:ETBReplacement": B, "kw:Prowess": B, "kw:Persist": B, "kw:Undying": B,
    "kw:Kicker": B, "kw:Multikicker": B, "kw:Changeling": B, "kw:Devoid": B, "kw:Exalted": B,
    "kw:Annihilator": B, "kw:Evolve": B, "kw:Affinity": B, "kw:Buyback": B, "kw:Entwine": B,
    "kw:Casualty": B, "kw:Bargain": B, "kw:Strive": B, "kw:Storm": M("spell-copies"),
    "kw:Ward": M("modes-plus-choice-kinds: unless pays"),
    # the route names protection's targeting half in cant-be-targeted; its
    # blocking half and landwalk are combat's
    "kw:Protection": M("cant-be-targeted"), "kw:Landwalk": M("combat-restrictions"),
    "kw:Horsemanship": M("combat-restrictions"), "kw:Fear": M("combat-restrictions"), "kw:Shadow": M("combat-restrictions"),
    "kw:Cycling": M("remaining-costs"), "kw:TypeCycling": M("remaining-costs"),
    "kw:Flashback": M("casting-from-other-zones"), "kw:Escape": M("casting-from-other-zones"),
    "kw:Madness": M("casting-from-other-zones"), "kw:Suspend": M("casting-from-other-zones"),
    "kw:Unearth": M("casting-from-other-zones"), "kw:Cascade": M("casting-from-other-zones"),
    "kw:Jump-start": M("casting-from-other-zones"), "kw:Retrace": M("casting-from-other-zones"),
    "kw:Disturb": M("double-faced-cards"), "kw:Daybound": M("double-faced-cards"), "kw:Nightbound": M("double-faced-cards"),
    "kw:Convoke": M("convoke-delve-improvise"), "kw:Delve": M("convoke-delve-improvise"), "kw:Improvise": M("convoke-delve-improvise"),
    "kw:Morph": M("face-down-and-morph"), "kw:Megamorph": M("face-down-and-morph"), "kw:Disguise": M("face-down-and-morph"),
    "kw:Foretell": M("face-down-and-morph"),
    "kw:Partner": M("game-formats"), "kw:Partner with": M("game-formats"),
    "kw:Infect": M("backlog: infect and wither"), "kw:Wither": M("backlog: infect and wither"),
    "kw:Bestow": M("backlog: bestow"), "kw:Mutate": M("merging-and-meld"),
    # costs
    "cost:Tap": B, "cost:Untap": B, "cost:Sac": B, "cost:PayLife": B,
    "cost:SubCounter": M("remaining-costs"), "cost:AddCounter": M("remaining-costs"),
    "cost:Discard": M("remaining-costs"), "cost:ExileFromGrave": M("remaining-costs"),
    "cost:tapXType": M("remaining-costs"), "cost:Exile": M("remaining-costs"),
    "cost:Return": M("remaining-costs"), "cost:Mill": M("remaining-costs"),
    "cost:Reveal": M("information-model"), "cost:Energy": M("backlog: energy"), "cost:PayEnergy": M("backlog: energy"),
    # mana-cost shapes
    "mana:X": B, "mana:hybrid": M("hybrid-and-phyrexian-payment"),
    "mana:phyrexian": M("hybrid-and-phyrexian-payment"), "mana:snow": M("backlog: snow"),
    # layouts
    "layout:DoubleFaced": M("double-faced-cards"), "layout:Modal": M("double-faced-cards"),
    "layout:Adventure": M("split-and-adventure-style-cards"), "layout:Split": M("split-and-adventure-style-cards"),
    "layout:Flip": M("split-and-adventure-style-cards"), "layout:Omen": M("split-and-adventure-style-cards"),
    "layout:Meld": M("merging-and-meld"), "layout:Specialize": D,
    # parameters
    "param:may": B, "param:remember": B, "param:intervening_if": B, "param:condition_present": B,
    "param:condition_count": B, "param:sorcery_speed": B, "param:loyalty_ability": B, "param:imprint": B,
    "param:duration_Permanent": B, "param:duration_UntilHostLeavesPlay": B,
    "param:duration_UntilHostLeavesPlayOrEOT": B, "param:duration_UntilYourNextTurn": B,
    "param:duration_UntilLoseControlOfHost": B, "param:duration_Perpetual": D,
    "param:unless_pays": M("modes-plus-choice-kinds: unless pays"), "param:divided": M("divided-among-targets"),
    "param:chosen_color": M("chosen-color-plus-could-produce"),
    # amounts
    "amount:Valid": B, "amount:xPaid": B, "amount:ValidGraveyard": B, "amount:ValidHand": B,
    "amount:ValidExile": B, "amount:ValidLibrary": B, "amount:CardPower": B, "amount:CardToughness": B,
    "amount:CardCounters": B, "amount:CardManaCost": B, "amount:LifeYouGainedThisTurn": B,
    "amount:LifeYouLostThisTurn": B, "amount:ThisTurnCast_Card": B, "amount:TriggeredCard": B,
    "amount:TriggerCount": B, "amount:CardTypes": B,
}

# Calibration, round 1: needs that only registered, working cards carry, so the
# engine has them. Each line names a card that proves it.
CALIBRATED = {
    "api:ReplaceToken": "Doubling Season, Parallel Lives",
    "api:ReplaceCounter": "Hardened Scales, Winding Constrictor",
    "api:ReplaceMana": "Mana Reflection, Nyxbloom Ancient",
    "api:ReplaceDamage": "Gisela, Guardian Seraph",
    "api:PreventDamage": "Samite Healer, Mending Hands",
    "api:Fog": "Fog",
    "api:ChooseSource": "Circle of Protection: Red",
    "api:Effect": "Skullcrack, Safe Passage (Forge's catch-all: the pilot's largest optimism)",
    "api:SkipTurn": "Meditate",
    "api:SkipPhase": "Moment of Silence",
    "api:PermanentNoncreature": "Bridge from Below",
    "api:ChangeZone[All->Exile,hidden]": "Rest in Peace, Leyline of the Void",
    "api:ChangeZone[All->Library,hidden]": "Darksteel Colossus, Nexus of Fate",
    "api:ChangeZone[Hand->Battlefield]": "Leyline of the Void (opening hand)",
    "api:ChangeZone[Stack->Exile]": "Stunning Reversal",
    "api:ChooseCard": "Sutured Ghoul",
    "param:choose_from_zone": "Sutured Ghoul",
    "amount:YourLifeTotal": "Ali from Cairo",
    "amount:YourStartingLife": "Exquisite Archangel",
    "trig:Unattached": "Grafted Wargear",
    "trig:DamagePreventedOnce": "Selfless Squire",
    "trig:TapsForMana": "Wild Growth",
    "static:Panharmonicon": "Panharmonicon",
    "static:CantSacrifice": "Sigarda, Host of Herons",
    "kw:Devour": "Thunder-Thrash Elder",
    "kw:Ingest": "Culling Drone",
    "param:duration_NextThisTurn": "Moment of Silence",
    # authoring is unaffected: partner is a deck-construction rule (CR 702.124)
    "kw:Partner": "Eligeth, Crossroads Augur",
    # round 3, once an Effect's contents were parsed
    "static:CantGainLife": "Skullcrack",
    "static:CantPreventDamage": "Skullcrack",
}
MAP.update({k: B for k in CALIBRATED})
MAP["api:ChangeZone[Command->Exile]"] = F  # Forge cleans up its own effect objects

# Round 2, read off the residue: Forge writes most library searches with
# `ChangeType$` and no `Hidden$`; the rest are plain reads of an existing type.
MAP.update({
    "api:ChangeZone[Library->Hand]": M("information-model"),
    "api:ChangeZone[Library->Battlefield]": M("information-model"),
    "api:ChangeZone[Library->Graveyard]": M("information-model"),
    "trig:Sacrificed": B,
    "api:RemoveFromCombat": B,
    "static:CantAttack,CantBlock": M("combat-restrictions"),
    # Forge's HIDDEN keywords: rules text a pump or a static grants
    "hidden:~ can't block": M("combat-restrictions"),
    "hidden:~ can't attack or block": M("combat-restrictions"),
    "hidden:~ can't attack": M("combat-restrictions"),
    "hidden:Unblockable": M("combat-restrictions"),
    "hidden:~ must be blocked if able": M("combat-requirements"),
    "hidden:All creatures able to block ~ do so": M("combat-requirements"),
    "hidden:This card doesn't untap during your next untap step": B,  # the event chokepoint's "can't"
    "hidden:~'s power and toughness are switched": B,
})

# Round 4, the corpus residue by card count. `phase-8-breadth` is a mechanic no
# named step builds: a new primitive, keyword, amount, duration or trigger arm,
# each small, which the roadmap leaves to Phase 8's breadth work.
BREADTH = "phase-8-breadth"
MAP.update({
    # expressible today by composition of existing types
    "param:other_player_chooses": B,  # EffectRecipient::ChosenBy
    "amount:Compare": B, "api:Investigate": B, "api:ChoosePlayer": B, "api:Branch": B,
    "api:ChangeZone[Hand->Exile]": B, "api:ChangeZone[Battlefield->Hand,hidden]": B,
    "api:ChangeZone[Hand->Library]": B, "api:ChangeZone[Graveyard,Exile->Hand,hidden]": B,
    "api:ChangeZoneAll[Graveyard->Battlefield]": B, "api:ChangeZoneAll[Exile->Battlefield]": B,
    "api:ChangeZoneAll[Graveyard->Library]": B, "api:Shuffle": B, "api:Debuff": B,
    "api:DamageResolve": B, "api:PermanentCreature": B, "api:StoreSVar": F,
    "amount:Kicked": B, "amount:TimesKicked": B, "amount:RememberedSize": B, "amount:RememberedNumber": B,
    "amount:AttackersDeclared": B, "amount:Morbid": B, "amount:YouDrewThisTurn": B,
    "amount:LifeOppsLostThisTurn": B, "amount:CardsInYourHand": B, "amount:ResolvedThisTurn": B,
    "trig:AttackerBlockedByCreature": B, "trig:DamageDealtOnce": B, "trig:AttackersDeclaredOneTarget": B,
    "trig:ChangesZone[Exile->Any]": B, "trig:LandPlayed": B,  # ZoneChangeAct::PlayedAsLand
    "kw:Bushido": B, "kw:Flanking": B, "kw:Soulshift": B,
    "param:duration_AsLongAsControl": B, "static:AlternativeCost": B,  # AlternativeCost::Custom, CR 118.9
    "kw:Evoke": B,  # AlternativeCost::Evoke
    # named steps
    "param:activated_from_Graveyard": M("casting-from-other-zones"),  # activatable_abilities walks the battlefield only
    "param:activated_from_Hand": M("casting-from-other-zones"),
    "param:activated_from_Exile": M("casting-from-other-zones"),
    "param:activation_limit": M("permission-design"),  # ActivationRestriction is None | OnlyAsSorcery
    "kw:Crew": M("remaining-costs"), "cost:Draw": M("remaining-costs"), "cost:RemoveAnyCounter": M("remaining-costs"),
    "kw:Ninjutsu": M("remaining-costs"),
    "api:ChooseType": M("chosen-color-plus-could-produce"), "api:ChooseColor": M("chosen-color-plus-could-produce"),
    "api:ManaReflected": M("chosen-color-plus-could-produce"),
    "api:ChangeZone[Library->Library]": M("information-model"), "api:ChangeZone[Library->Exile]": M("information-model"),
    "api:ChangeZoneAll[Library->Library]": M("information-model"),
    "kw:Cumulative upkeep": M("modes-plus-choice-kinds: unless pays"), "kw:Echo": M("modes-plus-choice-kinds: unless pays"),
    "api:TapOrUntap": M("modes-plus-choice-kinds: modal"),
    "api:BecomeMonarch": M("multiplayer-rules"),
    "static:CanAttackDefender": M("as-though-effects"),
    "api:Protection": M("cant-be-targeted"),
    "kw:Toxic": M("backlog: infect and wither"), "api:Poison": M("backlog: infect and wither"),
    "api:FlipCoin": M("backlog: dice"), "api:Clash": M(BREADTH),
    "api:MustBlock": M("combat-requirements"),
    "kw:CARDNAME can't block.": M("combat-restrictions"), "static:AssignNoCombatDamage": M("combat-restrictions"),
    "static:CantAttack,CantBlock,CantBeActivated": M("combat-restrictions"),
    "static:OptionalAttackCost": M("combat-requirements: costs to attack"),
    "kw:Rebound": M("casting-from-other-zones"), "kw:Plot": M("casting-from-other-zones"),
    "kw:Warp": M("casting-from-other-zones"), "api:Discover": M("casting-from-other-zones"),
    "api:Incubate": M("double-faced-cards"), "api:ManifestDread": M("face-down-and-morph"),
    "trig:Mutates": M("merging-and-meld"), "kw:Aftermath": M("split-and-adventure-style-cards"),
    "kw:Myriad": M("token-copies"), "kw:Encore": M("token-copies"), "trig:SpellCastOrCopy": M("spell-copies"),
    "amount:Converge": M("mana-provenance"), "amount:CastTotalManaSpent": M("mana-provenance"),
    "kw:Choose a Background": M("game-formats"), "kw:Doctor's companion": M("game-formats"),
    "kw:StickerSheet": D, "api:PutSticker": D,
    # Phase 8 breadth: one small mechanic each
    **{k: M(BREADTH) for k in (
        "kw:Chapter", "api:AlterAttribute", "param:duration_UntilTheEndOfYourNextTurn", "api:Proliferate",
        "api:Amass", "amount:ChosenNumber", "api:ChooseNumber", "amount:Domain", "api:Phases",
        "api:MultiplyCounter", "api:Explore", "api:RingTemptsYou", "api:Connive", "amount:Devotion",
        "kw:You may choose not to untap CARDNAME during your untap step.", "param:duration_UntilEndOfCombat",
        "api:ChangeTargets", "trig:AttackerUnblocked", "kw:Start your engines", "amount:Party", "kw:Banding",
        "api:Earthbend", "api:Empower", "kw:Firebending", "kw:Class", "api:MoveCounter",
        "kw:Station", "kw:Saddle", "api:ExchangeControl", "kw:Splice", "api:TwoPiles",
        "param:duration_UntilUntaps", "kw:Sneak", "kw:Ascend", "kw:Overload", "kw:Backup", "kw:Level up",
        "kw:Gift", "kw:Soulbond",
    )},
})


# Round 5, the next tail down.
MAP.update({
    "api:ChangeZone[Exile->Battlefield,hidden]": B, "api:ChangeZoneAll[Exile->Library]": B,
    "api:ChangeZone[Battlefield->Exile,hidden]": B, "api:ChangeZoneAll[Graveyard->Hand]": B,
    "api:ChangeZoneAll[Exile->Hand]": B, "api:ChangeZoneAll[Hand->Exile]": B,
    "api:ChangeZone[Hand->Exile,hidden]": B, "api:ChangeZone[Exile->Graveyard]": B,
    "api:RemoveCounterAll": B, "api:ChangeZone[Exile->Hand,hidden]": B,
    "api:ChangeZoneAll[Hand->Library]": B, "api:ChangeZoneAll[Hand,Graveyard->Library]": B,
    "trig:Phase[EndCombat]": B, "trig:Phase[Main]": B, "trig:DiscardedAll": B,
    "amount:TriggerRememberAmount": B, "amount:ThisTurnActivated_Activated": B,
    "param:duration_AsLongAsInPlay": B, "param:duration_EndOfTurn": B,
    "static:CantRegenerate": B,  # Restriction::ApplyReplacement
    "kw:Prevent all damage that would be dealt to CARDNAME.": B,
    "cost:ExileFromHand": M("remaining-costs"), "cost:CollectEvidence": M("remaining-costs"),
    "cost:ExileAnyGrave": M("remaining-costs"), "kw:AlternateAdditionalCost": M("remaining-costs"),
    "kw:Skulk": M("combat-restrictions"),
    "kw:CARDNAME must be blocked if able.": M("combat-requirements"),
    "kw:All creatures able to block CARDNAME do so.": M("combat-requirements"),
    "kw:Prototype": M("split-and-adventure-style-cards"), "kw:Fuse": M("split-and-adventure-style-cards"),
    "trig:UnlockDoor": M("split-and-adventure-style-cards"), "trig:FullyUnlock": M("split-and-adventure-style-cards"),
    "kw:Craft": M("double-faced-cards"), "kw:More Than Meets the Eye": M("double-faced-cards"),
    "kw:Living metal": M("double-faced-cards"), "api:DayTime": M("double-faced-cards"),
    "kw:Embalm": M("token-copies"), "kw:Eternalize": M("token-copies"),
    "kw:Replicate": M("spell-copies"), "kw:Conspire": M("spell-copies"), "kw:Cipher": M("spell-copies"),
    "kw:Spree": M("modes-plus-choice-kinds: modal"),
    "kw:Transmute": M("information-model"), "kw:Hideaway": M("information-model"),
    "api:TakeInitiative": M("backlog: dungeons"), "amount:YourCountersEnergy": M("backlog: energy"),
    "amount:TotalCommanderCastFromCommandZone": M("command-zone-casting"),
    "amount:UniqueManaColorsProduced": M("mana-provenance"), "trig:ManaExpend": M("mana-provenance"),
    "amount:Monarch": M("multiplayer-rules"), "static:ManaConvert": M("as-though-effects"),
    "api:ChangeZone[Sideboard->Hand,hidden]": M("game-formats"),
    "api:OpenAttraction": D,
    **{k: M(BREADTH) for k in (
        "api:ChangeZone[Stack->Hand]", "api:ChangeZone[Stack->Library]", "api:EachDamage", "api:Radiation",
        "api:Learn", "api:Airbend", "api:VillainousChoice", "api:ChangeText", "api:Detain",
        "cost:Exert", "cost:Waterbend", "cost:Blight", "cost:ExiledMoveToGrave",
        "static:CombatDamageToughness", "static:TapPowerValue", "static:AssignCombatDamageAsUnblocked",
        "static:UntapOtherPlayer", "trig:Exploited", "trig:BecomeMonstrous", "trig:CommitCrime",
        "trig:Surveil", "trig:ClassLevelGained", "amount:YourCountersExperience", "amount:Void",
        "amount:DevotionDual", "amount:Teamwork", "param:duration_UntilTheEndOfYourNextUntap",
        "param:duration_UntilYourNextEndStep", "alt:Cleave",
    )},
})


def status(need):
    if need in MAP:
        return MAP[need]
    if need.startswith("amount:ThisTurnEntered") or need.startswith("amount:ThisTurnCast"):
        return B
    # A keyword ability no row above names is a mechanic the engine has not
    # built: KeywordFlag holds 16, and the rest are each an engine addition.
    if need.startswith("kw:"):
        return M(BREADTH)
    return U
