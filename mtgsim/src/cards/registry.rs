use std::collections::HashMap;
use std::sync::Arc;

use crate::objects::card_data::CardData;

use super::fixtures;
use super::sets::{
    aer, ala, all, arn, avr, bbd, bfz, bok, c13, c14, c16, c17, chk, cmr, ddj, dgm, dis, dka, dmu, dom, drk, dst, ecl, eld, emn, eve, exo, fdn, fem,
    fic, fut, gpt, grn, gtc, hob, hou, ice, iko, inv, isd, jou, jud, khm, kld, ktk, lea, leb, lgn, lrw, m10, m11, m12, m13, m19, m20, m21, mbs, mh2, mir,
    mkm, mmq, mom, mrd, nem, ogw, ons, ori, p02, plc, pls, por, rav, rix, rna, rtr, s99, set_5dn, shm, snc, som, thb, tmp, tor, tsp, uds, ulg, usg,
    vis, war, wth, wwk, zen, znr,
};

/// The board an engine change is measured against — **representative, not
/// frozen** (revised 2026-09-01).
///
/// Stability across months buys the timing measurement nothing, since
/// `CLAUDE.md` mandates an **interleaved A/B in one sitting** and both arms
/// use this pool by construction; a stored ms/game number is machine drift,
/// not a baseline. What a freeze costs is *representativeness*: a phase that
/// adds a gated subsystem no pooled card can open has an A/B that measures
/// only the closed path.
///
/// **The rule: a phase that opens a new engine path adds one card here,
/// deliberately, and re-records the gameplay table in
/// `plans/engineering-practices.md#cards-and-pools`.** That table's
/// seed-deterministic rows — turns, spells cast, creatures died — are what an
/// addition invalidates and what still has to be re-measured. Registering a
/// card is still not the same act as adding one here.
const PERFORMANCE_POOL: [&str; 126] = [
    "Plains",
    "Island",
    "Swamp",
    "Mountain",
    "Forest",
    "Lightning Bolt",
    "Ancestral Recall",
    "Counterspell",
    "Burst of Energy",
    "Volcanic Upheaval",
    "Giant Growth",
    "Grizzly Bears",
    "Hill Giant",
    "Savannah Lions",
    "Earth Elemental",
    "Serra Angel",
    "Thornweald Archer",
    "Raging Cougar",
    "Wall of Stone",
    "Elvish Archers",
    "Ridgetop Raptor",
    "War Mammoth",
    "Knight of Meadowgrain",
    "Rhox War Monk",
    "Giant Spider",
    "Vampire Nighthawk",
    "Isamaru, Hound of Konda",
    "Night's Whisper",
    "Doom Blade",
    "Angel's Mercy",
    "Dark Ritual",
    "Glorious Anthem",
    "Cerulean Wisps",
    "Crimson Wisps",
    "Moonlace Fixture",
    "Blood Moon",
    "March of the Machines",
    "Tarmogoyf",
    "Culling Drone",
    "Humility",
    "Citanul Hierophants",
    "Act of Treason",
    "Sol Ring",
    "Darksteel Myr",
    "Merfolk Thaumaturgist",
    "Tundra",
    "Underground Sea",
    "Badlands",
    "Taiga",
    "Savannah",
    "Scrubland",
    "Volcanic Island",
    "Bayou",
    "Plateau",
    "Tropical Island",
    // The CR 101.2 restriction sweep. A pair, because neither is measurable
    // alone: Sigarda populates `restriction_ability_sources` (and nothing else
    // in the pool does), and the edict is the only resolution that asks her
    // anything.
    "Sigarda, Host of Herons",
    "Diabolic Edict",
    // Entering the battlefield as a replaceable event. A pair, because
    // CR 614.1c's two halves are two code paths: one writes a status before
    // anything can observe the permanent, the other allocates a CR 613.7c
    // timestamp per counter kind. The land is also the pool's first nonbasic
    // whose *own* ability CR 305.7 can strip, which is what makes Blood Moon's
    // effect on a tapland measurable here.
    "Idyllic Beachfront",
    "Chainbreaker",
    // The pool's replacement effect scoped by a *filter* rather than to its
    // own source, and so the board on which two replacement effects apply to
    // one event and CR 616.1 has to ask which one first. With Idyllic
    // Beachfront it is two `EnterWith(tapped)` rewrites on one land; with
    // Chainbreaker it is a filter and CR 122.6a on one artifact.
    "Root Maze",
    // The first card in the crate to propose a `GameAction::AddCounters`.
    // Chainbreaker is already here and enters with -1/-1 counters, so this is
    // also what makes CR 704.5q's annihilation happen in a measured game
    // rather than only in a fixture.
    "Battlegrowth",
    // The `AmountExpr::CountOf` in the layer walk: a CDA that enumerates the
    // battlefield, one frame per permanent per query — the quadratic
    // `replacement-architecture.md` §5a's enumeration row warned about,
    // measured rather than assumed. The look-ahead frame itself needs no card
    // here; Root Maze already routes every land drop through it.
    "Keldon Warlord",
    // The pool's only route to a layer 1 row, and so to the two gate legs and
    // the static re-registration path a copy adds. Mirrorweave is registered
    // but stays out: it opens the same engine path, and a second copy of one
    // path buys a slower fuzz run rather than a wider one.
    "Cytoshape",
    // The mana base. Not a new engine path — five printed mana abilities and
    // five basic land types, the same shape as the duals — but the land
    // `random_deck` fills every unassigned land slot with, so the pool cannot
    // build a deck without it, and every deck can pay for anything. The widest
    // board CR 305.7 has: Blood Moon strips five abilities here, not two.
    "Five-Color Land Fixture",
    // Two engine paths, one card each. Thunder-Thrash Elder is an application
    // that *prompts and mutates* (CR 614.13), a rewrite that is not a pure
    // function of the event and a non-`EnterWith` member a CR 616.1 entry step
    // can hold — so `ordering_cannot_change_outcome` has something it must
    // refuse to suppress; Sigarda is already here, which makes CR 101.2's
    // candidate filter live rather than fixtured. Master Biomancer is the
    // dynamic counter amount: a layer read per application, and §5b's
    // asymmetry on a board a random game reaches. Sutured Ghoul stays out at
    // `{4}{B}{B}{B}`.
    "Thunder-Thrash Elder",
    "Master Biomancer",
    // The pool's Aura, and so its `Host` row: membership is a `battlefield`
    // read per candidate per layer rather than a filter match, its own arm in
    // `board::affected_members`. Also the spell whose target comes from
    // `enchant_filter`, which is what puts CR 608.3b's fizzle and CR 704.5m/n
    // in front of a random game.
    "Holy Strength",
    "Bonesplitter",
    // The pool's effect that can depend on another's (CR 613.8): Blood Moon
    // strips the ability that generates Urborg's effect, so Urborg waits for it
    // whatever the timestamps say. Every other pooled layer is pairwise
    // independent under the static channel check, so this is what makes the
    // loop's slow path — the hypothetical, counted as `Dependency checks` —
    // live in a measured game. A land, so any deck that draws it drops it.
    "Urborg, Tomb of Yawgmoth",
    // The pool's cost ability that is the *spell's own* (CR 113.6d), so the
    // measured game in which CR 601.2f's gather has a second source, the
    // castability preview reads a hand card's own ability list, and an
    // `AmountExpr::CountOf` runs at cast time rather than inside a layer walk.
    // Colorless, so every deck can cast it; an artifact, so a second copy
    // counts the first. Frogmite is registered and stays out: it opens the
    // same path, and a second copy of one path buys a slower fuzz run rather
    // than a wider one.
    "Myr Enforcer",
    // The pool's row whose *existence* is a condition rather than an ability
    // lookup (CR 604.2), re-asked for every application in every layer of
    // every pass. One red mana, and the pool has nine lands with the Forest
    // subtype, so both answers happen in a measured game — including the one
    // Blood Moon takes away two layers earlier.
    "Kird Ape",
    // The pool's cost effect, and the card that populates
    // `cost_modification_ability_sources` and makes CR 601.2f's sweep run in
    // a measured game; she taxes both players' noncreature spells, and with
    // Humility already here the stripped-source path runs too. Electromancer
    // and Trinisphere are registered and stay out: the same engine path.
    "Thalia, Guardian of Thraben",
    // The pool's cost that is not mana. Its sacrifice is a payment routed
    // through the chokepoint from `pay_costs`, a mandatory additional cost
    // (CR 118.8b), a castability answer that turns on something other than
    // the mana cost, and a payment prompt that is not an allocation. Black,
    // and every deck in the pool has creatures, so it is both castable and
    // payable in a measured game.
    //
    // **Bone Splinters and not Altar's Reap, which opens the identical set.**
    // Altar's Reap draws two, so it made every measured game bigger — +20.2%
    // CPU/game against +13.8% for coverage the two share exactly, measured
    // 2026-09-08 on three arms. A pooled card is charged to every future A/B
    // sitting, so between two cards that open the same path the cheaper board
    // wins. Altar's Reap stays registered: it is CR 601.2h's own example.
    //
    // Thunderscape Familiar, Foundry Inspector and Mind Stone are registered
    // and stay out too: the Familiar and the Inspector open the path Thalia
    // already opens, and Mind Stone's puzzle is a test's board.
    "Bone Splinters",
    // The pool's static ability that watches a *damage* event, so the card
    // that opens the gather sweep on something combat proposes several times
    // a turn rather than once per entry. Also the pool's `Rewrite::Amount` and
    // its effect scoped to a **player**.
    //
    // Not legendary and mono-red at four mana, which is the point: CR 614.5's
    // own example is two of these multiplying damage by 4, and that branch is
    // reachable from a registered board only through a pooled pair
    // (`engineering-practices.md#how-many-cards-a-mechanic-owes`, tier 1).
    //
    // Ghosts of the Innocent, Gisela and Angel of Suffering stay out: Ghosts
    // opens the same path as Furnace at seven mana, Gisela is {4}{R}{W}{W} and
    // legendary, and Angel of Suffering's mill would make every measured game
    // bigger for coverage the stress pool already has. Loyalty Probe stays out
    // as a fixture; its reachability is a `--require` row instead.
    "Furnace of Rath",
    // {W} and "any target": the first registry row a pooled damage event
    // meets, and the first CR 615.7 `allocate` prompt a fuzz game can reach —
    // two attackers into a shielded player is a board every combat step
    // builds (`replacement-architecture.md` §9). Samite Healer, Safe
    // Passage and Samite Censer-Bearer stay out: the Healer opens the same
    // path from an activation the random agent will take anyway in `stress`,
    // Safe Passage is a whole-event prevention the pool already measures
    // through Angel of Suffering's shape, and the Censer-Bearer's rows are
    // one per creature at {W} plus itself — breadth, not a new cost.
    "Mending Hands",
    // The source in the pool whose gather evaluates an `ObjectFilter` on the
    // damage's **source** rather than on its target — a read per damage event
    // for as long as it is on the battlefield, which is what its A/B measured
    // (`replacement-architecture.md` §9). {2}{W}{W} for a
    // 3/4 flier is castable in a white deck, and the predicate is one an
    // opponent's every attack satisfies.
    //
    // The other seven stay out. Daunting Defender opens the same
    // `PreventUpTo` path at {4}{W}; Circle of Protection: Red's row needs a
    // `{1}` activation the random agent rarely affords, so its reachability is
    // a `--require` row instead; Reverse Damage and Dark Sphere are the same
    // chosen-source path as the Circle; Fog and Pyroclasm are one-shots whose
    // engine cost is a batch the pool already pays; Torbran is legendary at
    // {1}{R}{R}{R}.
    "Guardian Seraph",
    // The pool's `Rewrite::Retarget`, and its Aura whose static ability is a
    // replacement effect rather than a continuous one. Two engine paths
    // nothing else here opens: a rewrite that changes the event's *subject*,
    // so the CR 616.1 chooser and the next iteration's gather are asked about
    // a different object; and an `attached_to` read on a damage event.
    //
    // Castable at {2}{W} in a white deck that already runs Mending Hands and
    // Guardian Seraph, and it needs a creature to enchant, which the random
    // agent supplies — an Aura with no legal host is countered on resolution
    // (CR 608.3b) and costs nothing.
    //
    // The other three stay out. Palisade Giant is the same `Retarget` path at
    // {4}{W}{W}, and a 2/7 that the random agent will attack with distorts the
    // combat fixtures more than its redirect adds; Pinpoint Avalanche's flag is
    // a `bool` on an event the pool already proposes thousands of; Reflect
    // Damage is the chosen-source path Circle of Protection: Red already
    // measured, at {3}{R}{W}.
    "Pariah",
    // The pool's card whose effect is a *dropped* turn-structure proposal.
    // Every other replacement source here watches something a spell or a
    // combat step proposes; this one watches a unit the turn machinery
    // proposes on a fixed schedule, so from the moment it resolves it opens
    // the per-permanent gather sweep on every player's upkeep, every turn,
    // for the rest of the game.
    //
    // Colorless at five, so every deck can cast it, and its scope is
    // `PlayerSet::Everyone` — the only pooled effect that applies to a player
    // who is not its controller and is not about damage.
    //
    // The other four stay out, each for its own reason. Yawgmoth's Bargain
    // gives the random agent a use for its life total and would empty
    // libraries, a distortion of every fixture; Time Walk's extra turn moves
    // `Avg turns/game` by design; Meditate and Moment of Silence are one-shots
    // whose engine path this card already opens.
    "Eon Hub",
    // The pooled card that watches a *draw*, and so the one that opens the
    // gather sweep on both draw events: the instruction every turn's draw
    // step proposes, and each individual draw under it. Also the pool's
    // `Instead` whose output is decomposed, which is the path §3.2d's lineage
    // rule runs on — with one copy nothing can re-apply, but the sweep, the
    // substitution and the decomposition all run unforced.
    //
    // Seven mana is the most any pooled card has cost, so its reachability is
    // measured with `--require` rather than argued; see its row in
    // `engineering-practices.md#cards-and-pools`.
    //
    // The other three stay out. Alms Collector only ever watches an *opponent's*
    // instruction of two or more, and the pool's multi-card draws are Ancestral
    // Recall and Night's Whisper, so it would buy a creature slot to do nothing
    // in most games; Teferi's Ageless Insight and Notion Thief are the same
    // shape as this card with a narrower pattern.
    "Thought Reflection",
    // The pooled card that meets a proposal the pool already makes: lifelink's
    // contained `GainLife` is proposed in every measured game, and Knight of
    // Meadowgrain and Vampire Nighthawk are both here. So this is the one
    // replacement consumer whose gather sweep runs without a second card to
    // set it up — and, with lifelink of its own, it doubles the life its own
    // damage gains.
    "Rhox Faithmender",
    // The game's end. A three-drop whose static is a draw watcher gated on a
    // library state: the pool's conditional replacement, so the gather's "as
    // long as" leg runs on every draw while it is on the battlefield, and the
    // card that makes decking a *win* in a measured game. Fuzz games deck out
    // rarely, so the rows to read are `--require`'s and the "wins by effect"
    // outcome line. Platinum Angel is registered and stays out: a seven-drop
    // that turns every lethal board into a stall would move average turns by
    // design and not by engine.
    "Laboratory Maniac",
    // Plural creation and its `CreateTokens` watcher. Raise the Alarm is the
    // producer: `{1}{W}`, so any white deck casts it, and its two Soldiers are
    // the plural entry batch a measured game builds — every entry decided
    // against the board none has entered (CR 614.12), which is the engine
    // path `codebase-state.md` item 46 wanted measured. Parallel Lives is the
    // outer event's watcher, and the board on which the creation's CR 616.1
    // loop and the entries' loops both run in one resolution (CR 616.1g).
    // Kalitas's rider already makes single tokens in `stress`, so the
    // registered-but-unpooled arm moves by one gather per Zombie and nothing
    // else.
    //
    // Hordeling Outburst stays out: the Alarm's path at `{1}{R}{R}`. Hallowed
    // Moonlight stays out: its row is a substituted entry, and its one new
    // line — a token created in exile — needs a creation under the row in
    // the same turn, a `--require` question rather than a slot.
    "Parallel Lives",
    "Raise the Alarm",
    // The `EventPattern::AddCounters` watcher, at one mana. Hardened Scales
    // opens the sweep on every `AddCounters` (Battlegrowth is pooled) and on
    // every counter-bearing entry — Chainbreaker's, Master Biomancer's
    // grants — and beside a second Scales it is the pool's additive pair,
    // which the predicate asks about (`backlog.md` §2.29). Doubling Season
    // stays out: five mana, and its token half would double the pool's
    // Soldiers, a gameplay change the A/B should not carry with the engine
    // change. Vorinclex is six mana and legendary, Winding Constrictor two
    // colors, Primal Vigor the Season with `Everyone`, and Live Fast a cantrip
    // whose energy nothing pooled reads: each is the same path at more mana,
    // or a path with no second card to meet.
    "Hardened Scales",
    // The pool's discard outside the cleanup step, and its scry. Two cards
    // because they are two keyword actions with nothing in common: Mind Rot is
    // `{2}{B}` for one batch of two `ZoneChange { act: Discarded }` members
    // from a resolution, and Opt is `{U}` for a `GameAction::Scry` — a
    // proposal, a prompt and a library rewrite. Opt also draws, so it is the
    // cheapest card in the pool that puts two instructions in one resolution
    // (CR 608.2c) with a replacement watcher on the first of them.
    //
    // The other three stay out. Hymn to Tourach is a second two-card discard
    // and would double Mind Rot's measurement rather than add to it; Nephalia
    // Academy's row only ever fires against an *opponent's* discard spell, and
    // the pool has exactly one; Eligeth is a six-drop whose ability needs a
    // scry the pool can only get from Opt in the same deck.
    "Mind Rot",
    "Opt",
    // The pool's row that reaches a zone other than the battlefield, and the
    // only way a measured game exercises the working-set path at all. Every
    // other pooled card leaves `reachable_zones` at `BATTLEFIELD`, where the
    // seed's zone loop does not run — so without this card the A/B would
    // measure the guard and never the path it guards.
    //
    // `{1}{B}` for a 2/1, so a black deck casts it early and it stays out for
    // the rest of the game, which is the shape that matters: from the moment
    // it resolves, every graveyard card in the game is a member of every layer
    // pass, and graveyards only grow.
    //
    // Scarwood Treefolk is registered and stays out. It opens no engine path
    // — "enters tapped" is Idyllic Beachfront's row, already pooled — and its
    // job is to be the Jailer's partner in the atom, where a graveyard is
    // arranged rather than arrived at.
    "Yixlid Jailer",
    // CR 113.6, and the pool's static ability that functions **from** a zone
    // other than the battlefield. It opens the path in `move_object`: every
    // zone change asks `zone_function` whether the arriving object has an
    // ability that works there, and every registry row whose source is in a
    // graveyard sends the existence check down `compute_non_member` instead
    // of reading a live frame. `{3}{U}` for a 2/2 flier is castable, it dies
    // like any other creature, and from the moment it does the board carries
    // a row whose source is not on the battlefield — a cost shape no other
    // pooled card measures.
    "Wonder",
    // The pool's watcher of the mana production event, and a static on the
    // hottest path in the engine: from the turn it resolves, every land tap is
    // a gather with a match, where every tap of every game was already a
    // gather with none. The middle arm's `Replacement gathers` moves by one
    // per production whether or not anything watches, and this card is what
    // makes the sweep find something. Its pooled arm moves the gameplay rows
    // by design (twice the mana is more spells and bigger boards), so the
    // reading is `ms / 1,000 walks` beside `CPU/turn p50`.
    //
    // Nyxbloom Ancient stays out: the same path at seven mana with a 5/5
    // trample body that changes combat. Deep Water stays out: an activated
    // `{U}` the random agent would spend on nothing, so its reachability is a
    // `--require` row.
    "Mana Reflection",
    // The pool's source **off the battlefield** for the replacement sweep —
    // the gather's zone leg (RF, `replacement-architecture.md` §3.3 source
    // 2). At eleven mana it is almost never cast, and that is the point: what
    // it puts in front of every measured game is a card in a library or a
    // hand whose printed clause functions there, so the zone leg reads its
    // frame on every gather from the first turn — the cost shape this phase
    // opens, which no permanent can measure. Nexus of Fate stays out for
    // Time Walk's reason: an extra turn moves `Avg turns/game` by design.
    "Darksteel Colossus",
    // The pool's spell with **several instances of the word "target"**
    // (CR 601.2c, A4i) — the announcement loop, the CR 608.2b re-check per
    // instance, and the per-instance castability gate, in a measured game
    // rather than only in a fixture. Three clauses, identical criteria, so a
    // random agent that can reach one creature reaches all three of them and a
    // board with several makes each announcement a real prompt. Two mana and
    // two colors the pool's duals cover, so a random deck can cast it;
    // Incremental Growth, Jagged Lightning and Plague Spores stay out — the
    // first two walk this same loop and the third wants a board a random game
    // does not assemble.
    "Seeds of Strength",
    // The trigger spine's three, one per engine path (TR-1). Soul Warden is
    // the matcher at a batch's close: every creature entering asks it, and
    // two entering together are one window. Blood Artist is a placement
    // target chosen at CR 603.3d, and a look-back off the CR 603.10a frame
    // when it dies with the rest. Wild Growth is CR 605.1b's stackless path:
    // resolved at dispatch inside the mana window, never queued.
    "Soul Warden",
    "Blood Artist",
    "Wild Growth",
    // TR-2a's two. Vengeful Warchief reads the turn's history record by record
    // ("for the first time each turn"), which every life loss in a game
    // advances. Elvish Warmaster is "one or more" behind "triggers only once
    // each turn", and its pump is a one-shot over a filter; the pool's Elvish
    // Archers and Thornweald Archer are Elves for it to see.
    "Vengeful Warchief",
    "Elvish Warmaster",
    // TR-2b's two. Psychosis Crawler is a hand's count in the layer walk,
    // which every draw and discard moves, and the draw arm. Cosi's Trickster
    // is the shuffle arm and a "may" asked at resolution. Nykthos Paragon
    // stays out: its trigger is TR-2a's life-gain arm and its "may" is the
    // Trickster's, so it opens no path the two do not.
    "Psychosis Crawler",
    "Cosi's Trickster",
    // CV-2a's: the pool's only permanent that enters as a copy (CR 707.5), so
    // the only route to CR 616.1c's bucket, the entry copy in the frame and
    // the board pass, and registration filing a copied list. Its "you may"
    // is a `(0, 1)` pick, so a random game declines about half the time and
    // the 0/0 dies to CR 704.5f, which measures that path too.
    "Clone",
    // RG's: "enters untapped", a status the last applied effect sets, and
    // CR 616.1's order between opposite statuses, which beside Idyllic
    // Beachfront and Root Maze a random game now reaches. Tapped, it taps
    // every other permanent that enters, any player's.
    "Archelos, Lagoon Mystic",
    // CV-2b's: the only card that makes CR 707.9's exceptions, so the only
    // route to the applier, 707.9f's judgment on a look-ahead of the copy and
    // 707.9e's counter on the entry. Its "you may" is Clone's `(0, 1)` pick,
    // over its controller's own creatures only, since the pool has no
    // planeswalker; the tests carry that path.
    "Spark Double",
    // CV-1b's: the only copy row with no end but a move (CR 611.2a, 400.7),
    // so the only rows that outlive their turn and that CR 400.7's prune ends,
    // and a re-copy each upkeep over the last. Its target is "another", and
    // its "may" is asked at resolution.
    "Cryptoplasm",
    // MA-1's two paths no pooled card reaches (`mana-architecture.md` §3.3):
    // mana once for each permanent an outlet can sacrifice, with the cost's
    // own sacrifices taken first and the window's sacrifice prompt, and a
    // mana ability fed by mana, whose doubling the check reads as a transform
    // of the demand and the window now offers. Colorless, so every deck holds
    // them; the Cube beside Ironworks is the board where what is sacrificed
    // after the doubling pays undoubled.
    "Krark-Clan Ironworks",
    "Doubling Cube",
    // TR-3b's two paths: a delayed trigger that refers to the card its
    // creator exiled (CR 603.7c) and returns it, an Aura choosing its host as
    // it enters (CR 303.4f); and CR 610.3's "until" return, made as its
    // source's departure is dispatched, with no stack.
    "Flickerwisp",
    "Banishing Light",
    // TR-4a's: an intervening "if" read off the triggering object's last known
    // information at the trigger and at resolution, and a return with a
    // counter. It returns once, so a game gains a creature and not a loop.
    "Strangleroot Geist",
    // TR-4b's: "becomes attached", read after the event, an Aura entering
    // attached included (CR 303.4f). While it is on the battlefield every
    // attach a pooled Aura or Equipment makes is matched against it. Benign,
    // unlike Grafted Wargear, whose Equip {0} the random agent turns on its
    // own creatures.
    "Bramble Elemental",
    // TR-4c's: a trigger asked from a graveyard, the zone its intervening "if"
    // states. While it sits in one, every death is matched against it there.
    // It does nothing on the battlefield, so it ends no game.
    "Bridge from Below",
    // TR-4d's: a change of control, which no performer makes, announced by
    // the sweep; its steal ends at cleanup, where its delayed trigger repeats
    // the step. A steal until end of turn ends no game.
    "Ray of Command",
    // TR-5a's: a damage trigger whose instruction names the damage's source,
    // and a grant over a filter that counts counters. A counter after combat
    // damage, and reach and trample, end no game.
    "Sphere Grid",
    // TR-5e's: an attack trigger for each creature its controller attacks
    // with, and damage to what that creature is attacking. A point for each
    // attacker shortens games; how much it moves `stress` is read at the
    // close-out and reported (`triggers-architecture.md` §12, decision 7).
    "Hellrider",
    // TR-5f's: tokens put onto the battlefield attacking, each defender chosen
    // as it enters at four seats, and every removal from combat the end of
    // combat announces. Two more attackers shorten games; how much it moves
    // `stress` is read at the close-out and reported.
    "Hanweir Garrison",
    // TR-5b's: a removal of counters read once per counter, CR 704.5q's
    // annihilation in the state-based batch among them, each making a
    // delayed trigger; an entry's X (CR 107.3m); and a prevention whose rider
    // removes counters. Castable since MA-2 offers an X spell at the gate.
    "Protean Hydra",
    // TR-5b's other: a put of counters read once for each creature, that many
    // counters of a kind no rule reads, and a win behind an intervening "if"
    // over them. It ends some games (2.5 points of four-seat `stress` games,
    // natural decks), kept for the variety of ways a game ends.
    "Simic Ascendancy",
    // MA-2's: the plain X spell. The gate asks its least X, CR 601.2b offers
    // the X the board can pay, read in closed form, and the announced X
    // deals the damage. A burn spell, so it ends some games by damage; how
    // much it moves `stress` is read at the close-out and reported.
    "Blaze",
    // TR-5c's: the declaration of blockers read once for each block (CR
    // 509.3b), and one instruction on two objects, which destroys both in
    // one event. A random agent's block is what triggers it.
    "Loyal Sentry",
    // TR-5c's other: becoming the target of a spell or ability, whoever
    // controls it, once for each however many instances chose it. Every
    // targeted spell and ability aimed at it opens the path; it mills its
    // controller, which a long game can feel.
    "Cephalid Aristocrat",
    // TR-5d's: CR 615.13's event, one record each time a prevention effect
    // is applied to its controller's damage, read at the batch's close. Its
    // own entry makes the prevention, so any damage aimed at its controller
    // that turn opens the path.
    "Selfless Squire",
    // TR-5d's other: the excess damage a resolution's own effect dealt,
    // read in the same resolution after its damage's window, and X. Any
    // creature it hits for more than its toughness makes Goblins.
    "Goblin Negotiation",
    // TR-5g's: CR 603.2d's count, asked of every window that triggers
    // something while one is on the battlefield. No agent choice: it doubles
    // what its controller's permanents trigger on an artifact or creature
    // entering. Registered alone it moved `stress` games by about a turn
    // either way and no game ended differently in kind.
    "Panharmonicon",
    // TR-6's: CR 603.8's state check, asked after every dispatch
    // and before placement. No agent choice: alone on its controller's side
    // it sacrifices itself, which in a random deck is often.
    "Emperor Crocodile",
    // TR-7's: a triggered ability granted under a condition, which the
    // dispatcher asks off the frame of the Scarecrow that died. No agent
    // choice; without a black creature it is a 2/2 for three. Viscera Seer,
    // the loop's other new card, stays out: a random agent sacrifices its own
    // creatures with it at nearly every priority.
    "Wingrattle Scarecrow",
    // TR-7's entry counter over "not a Human", Master Biomancer's path at a
    // fixed amount. No agent choice.
    "Grumgully, the Generous",
    // CR 601.2d's division on a cast: a second target asks the first
    // `allocate` a spell's announcement reaches, and each share is a batch
    // member of its own size. Sorcery speed and three mana, so it comes up
    // on most boards without crowding out a turn.
    "Arc Lightning",
    // CR 700.2's modes: the offer of legal sets and the castability walk
    // that asks it for its first, at every priority question with it in
    // hand, and the modes resolved. Two mana and an instant, and the board
    // decides which of its two modes can be chosen. Cryptic Command and
    // Casualties of War are registered and stay out: the same path.
    "Abrade",
];

/// Cards in development (`setup-architecture.md` §7a): a card a developer is
/// building, which a scenario can name before the engine plays it. Only
/// scenarios read this list: `card_names`, and so `random_deck`, the fuzz
/// pools and `determinism_test`, never see it. A card leaves the list in the
/// commit that registers it.
const IN_DEVELOPMENT: [(&str, fn() -> Arc<CardData>); 0] = [];

/// Card registry: maps card names to factory functions that produce CardData.
///
/// Contributors add new cards by:
/// 1. Creating a function that returns CardData (using CardDataBuilder)
/// 2. Registering it here with `register()`
///
/// This keeps card definitions purely data-driven — no engine code needed.
///
/// Two pools are built from these definitions — `default_registry` (everything)
/// and `performance_pool` (representative). `plans/engineering-practices.md#cards-and-pools`
/// says which to reach for.
pub struct CardRegistry {
    cards: HashMap<String, fn() -> Arc<CardData>>,
    /// [`IN_DEVELOPMENT`]'s cards, or a test's.
    in_development: HashMap<String, fn() -> Arc<CardData>>,
}

impl CardRegistry {
    pub fn new() -> Self {
        CardRegistry {
            cards: HashMap::new(),
            in_development: HashMap::new(),
        }
    }

    /// Register a card factory function
    pub fn register(&mut self, name: &str, factory: fn() -> Arc<CardData>) {
        self.cards.insert(name.to_string(), factory);
    }

    /// Look up a card by name and create a fresh Arc<CardData>
    pub fn create(&self, name: &str) -> Result<Arc<CardData>, String> {
        self.cards.get(name)
            .map(|factory| factory())
            .ok_or_else(|| format!("Card '{}' not found in registry", name))
    }

    /// List `name` among the cards in development, which scenarios can name
    /// and nothing else draws from.
    pub fn register_in_development(&mut self, name: &str, factory: fn() -> Arc<CardData>) {
        self.in_development.insert(name.to_string(), factory);
    }

    /// A card in development by name, for a scenario.
    pub fn create_in_development(&self, name: &str) -> Option<Arc<CardData>> {
        self.in_development.get(name).map(|factory| factory())
    }

    /// The cards in development, alphabetically: the names a scenario can use
    /// besides [`Self::card_names`], which never lists them.
    pub fn names_in_development(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.in_development.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    /// Get all registered card names, alphabetically.
    ///
    /// Sorted, not raw `HashMap` key order: `fuzz_games` builds its decks by
    /// drawing from this list with a seeded RNG, so an order that changes per
    /// process makes the same seed build a different deck.
    pub fn card_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.cards.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    /// Every card the crate can build. `fuzz_games --pool stress` plays this
    /// one, and it grows with each phase.
    pub fn default_registry() -> Self {
        let mut registry = CardRegistry::new();

        // Basic lands
        registry.register("Plains", lea::plains);
        registry.register("Island", lea::island);
        registry.register("Swamp", lea::swamp);
        registry.register("Mountain", lea::mountain);
        registry.register("Forest", lea::forest);

        // The first spells
        registry.register("Lightning Bolt", lea::lightning_bolt);
        registry.register("Ancestral Recall", lea::ancestral_recall);
        registry.register("Counterspell", lea::counterspell);
        registry.register("Burst of Energy", ulg::burst_of_energy);
        registry.register("Volcanic Upheaval", bfz::volcanic_upheaval);
        registry.register("Giant Growth", lea::giant_growth);

        // Vanilla creatures
        registry.register("Grizzly Bears", lea::grizzly_bears);
        registry.register("Hill Giant", lea::hill_giant);
        registry.register("Savannah Lions", lea::savannah_lions);
        registry.register("Earth Elemental", lea::earth_elemental);

        // Keyword creatures
        registry.register("Serra Angel", lea::serra_angel);
        registry.register("Thornweald Archer", fut::thornweald_archer);
        registry.register("Raging Cougar", por::raging_cougar);
        registry.register("Wall of Stone", lea::wall_of_stone);
        registry.register("Elvish Archers", lea::elvish_archers);
        registry.register("Ridgetop Raptor", lgn::ridgetop_raptor);
        registry.register("War Mammoth", lea::war_mammoth);
        registry.register("Knight of Meadowgrain", lrw::knight_of_meadowgrain);
        registry.register("Rhox War Monk", ala::rhox_war_monk);
        registry.register("Giant Spider", lea::giant_spider);
        registry.register("Vampire Nighthawk", zen::vampire_nighthawk);

        // The cards that came before the layer system.
        registry.register("Isamaru, Hound of Konda", chk::isamaru_hound_of_konda);
        registry.register("Night's Whisper", set_5dn::nights_whisper);
        registry.register("Doom Blade", m10::doom_blade);
        registry.register("Angel's Mercy", m10::angels_mercy);
        registry.register("Dark Ritual", lea::dark_ritual);
        registry.register("Glorious Anthem", usg::glorious_anthem);
        registry.register("Arc Lightning", usg::arc_lightning);

        // Layer-system cards.
        //
        // Only the ones whose definition is faithful to the printed card, because
        // this registry feeds the dev GUI as well as `fuzz_games`. The layer
        // phases' other cards stay out and each has a reason:
        //
        // - Chromatic Ward and Windswept Heights are invented cards.
        // - Urborg Effect is a deliberate stand-in (an Enchantment carrying the
        //   land's ability) rather than Urborg itself.
        // - The four `*_spell_fixture` cards are an Aura's or an Artifact's
        //   effect cast as an Instant, sparing machinery their phase did not
        //   need.
        // - Inside Out Fixture is Inside Out at a cost with no hybrid pip.
        // - Cloudspire Mesa and Moonlit Steppe are invented Lands; the ten
        //   real dual lands below are what `fuzz_games::random_deck` draws its
        //   nonbasic land slots from.
        //
        // Moonlace Fixture is the one fixture this block registers: the
        // pool's Layer 5 colorless setter, Moonlace's shape narrowed to a
        // target creature until end of turn.
        registry.register("Cerulean Wisps", shm::cerulean_wisps);
        registry.register("Crimson Wisps", shm::crimson_wisps);
        registry.register("Moonlace Fixture", fixtures::moonlace_fixture);
        registry.register("Blood Moon", drk::blood_moon);
        registry.register("March of the Machines", mrd::march_of_the_machines);
        registry.register("Tarmogoyf", fut::tarmogoyf);
        registry.register("Culling Drone", bfz::culling_drone);
        registry.register("Humility", tmp::humility);
        registry.register("Citanul Hierophants", usg::citanul_hierophants);
        registry.register("Act of Treason", m10::act_of_treason);
        // Layer 7b's and 7c's spells from the layer phase, written whole. Not
        // pooled: Humility and March of the Machines already reach 7b, and
        // Giant Growth reaches 7c from a resolution.
        registry.register("Zhalfirin Shapecraft", mom::zhalfirin_shapecraft);
        registry.register("Bull Rush", wwk::bull_rush);

        // Layer 7b and 7d's random-play coverage: March of the Machines needs an
        // artifact to animate, and nothing else switches a creature's P/T. Sol
        // Ring is colorless, so every deck gets it; the Thaumaturgist rides in
        // blue decks, which is where March lives too.
        registry.register("Sol Ring", lea::sol_ring);
        registry.register("Darksteel Myr", som::darksteel_myr);
        registry.register("Merfolk Thaumaturgist", plc::merfolk_thaumaturgist);

        // The ten original duals. Nonbasic lands with basic land types, which is
        // what gives `fuzz_games` a mana base Blood Moon can actually affect.
        registry.register("Tundra", lea::tundra);
        registry.register("Underground Sea", lea::underground_sea);
        registry.register("Badlands", lea::badlands);
        registry.register("Taiga", lea::taiga);
        registry.register("Savannah", lea::savannah);
        registry.register("Scrubland", lea::scrubland);
        registry.register("Volcanic Island", leb::volcanic_island);
        registry.register("Bayou", lea::bayou);
        registry.register("Plateau", lea::plateau);
        registry.register("Tropical Island", lea::tropical_island);
        // The eleventh mana-base land, and a fixture: the registry's only
        // source of every color, which is what lets `fuzz_games --require`
        // stop seeding a deck's colors from the required card's. Everywhere,
        // whose shape it has, is a token, which no deck holds.
        registry.register("Five-Color Land Fixture", fixtures::five_color_land_fixture);

        // The first replacement effect with printed card text.
        registry.register("Kalitas, Traitor of Ghet", ogw::kalitas_traitor_of_ghet);
        // The second and third replacement sources, which make CR 616.1's "two
        // or more" branch reachable in a game
        // (`engineering-practices.md#how-many-cards-a-mechanic-owes`).
        registry.register("Rest in Peace", rtr::rest_in_peace);
        registry.register("Leyline of the Void", gpt::leyline_of_the_void);

        // A CR 101.2 "can't" with printed card text, and
        // the resolution that makes its *absence of a prompt* observable.
        registry.register("Sigarda, Host of Herons", avr::sigarda_host_of_herons);
        registry.register("Diabolic Edict", tmp::diabolic_edict);

        // Cards that modify how a permanent enters
        // (CR 614.1c/d). One per half of `EnterMods`: status and counters.
        registry.register("Idyllic Beachfront", dmu::idyllic_beachfront);
        registry.register("Chainbreaker", shm::chainbreaker);
        registry.register("Adaptive Shimmerer", iko::adaptive_shimmerer);

        // The registered `ObjectSet::Filter` replacement, and the card that makes
        // CR 616.1's multi-candidate branch reachable in a fuzz game.
        registry.register("Root Maze", tmp::root_maze);
        // Containment Priest is the entry replacement that does *not* commute
        // with an `EnterWith`, so CR 616.1's multi-candidate branch stays
        // reachable while the commuting case does not prompt; Dryad Arbor is the
        // only road a fuzz game has to a creature that "wasn't cast". Keldon
        // Warlord counts the battlefield, and does not count itself while
        // entering (its doc says why).
        registry.register("Containment Priest", c14::containment_priest);
        registry.register("Dryad Arbor", fut::dryad_arbor);
        registry.register("Keldon Warlord", lea::keldon_warlord);

        // CR 614.13's auxiliary zone changes, one card per candidate
        // zone (the battlefield and a player's graveyard are two enumerations),
        // plus the first entry replacement whose counter amount is read off the
        // board rather than printed on the card.
        registry.register("Thunder-Thrash Elder", ala::thunder_thrash_elder);
        registry.register("Sutured Ghoul", jud::sutured_ghoul);
        registry.register("Master Biomancer", gtc::master_biomancer);

        // The cards that put a row in layer 1. A pair,
        // because `CopyRoles` has two arms and each card is one of them:
        // Cytoshape targets the permanent that becomes a copy and *chooses* its
        // donor, Mirrorweave targets the donor and affects everything else.
        registry.register("Cytoshape", dis::cytoshape);
        registry.register("Mirrorweave", shm::mirrorweave);
        // The third arm-consumer: Mirrorform prints Mirrorweave's shape without
        // the word "other", which is why `exclude_donor` is a field.
        registry.register("Mirrorform", ecl::mirrorform);
        // CR 707.5: the permanent that enters as a copy, CR 616.1c's producer.
        registry.register("Clone", lea::clone);
        // CR 707.9: the entry copy that makes exceptions, over a creature or
        // planeswalker its controller controls.
        registry.register("Spark Double", war::spark_double);
        // CR 611.2a: a copy with no stated duration, which a move alone ends.
        registry.register("Cryptoplasm", mbs::cryptoplasm);
        // CR 110.5b's other status: a permanent that enters untapped because
        // an effect says so (`replacement-architecture.md` §3.5).
        registry.register("Archelos, Lagoon Mystic", cmr::archelos_lagoon_mystic);

        // The +1/+1 half of CR 704.5q. Its -1/-1 half is Chainbreaker above,
        // and until this card the annihilation had never happened in a fuzz
        // game — `engineering-practices.md#how-many-cards-a-mechanic-owes`.
        registry.register("Battlegrowth", mrd::battlegrowth);

        // An Aura whose static ability lowers to `ObjectSet::Host`, and a spell
        // whose recipient comes from `enchant_filter` (CR 303.4a) rather than a
        // spell ability — which is what makes CR 608.3b and CR 704.5m/n
        // reachable from a fuzz game at all.
        registry.register("Holy Strength", lea::holy_strength);
        registry.register("Bonesplitter", mrd::bonesplitter);
        registry.register("Cobbled Wings", isd::cobbled_wings);
        registry.register("Loxodon Warhammer", mrd::loxodon_warhammer);

        // The CR 613.8 boards the rulings walk. Urborg is the existence
        // dependency and pooled; Ashaya is the applies-to dependency on a
        // printed card, with a CR-derived answer; Opalescence is the CR 613.6
        // test with the Humility rulings' answers, and the first "each other"
        // (`ObjectFilter::NotSource`). The Rootpath Purifier ruling's board is
        // `fixtures::purifier_clause`, a fixture registered nowhere.
        registry.register("Urborg, Tomb of Yawgmoth", plc::urborg_tomb_of_yawgmoth);
        registry.register("Opalescence", uds::opalescence);
        registry.register("Ashaya, Soul of the Wild", znr::ashaya_soul_of_the_wild);

        // Conditional statics. Kird Ape is the cheapest printed one
        // and pooled; `fixtures::flight_clause` (Rune of Flight's third
        // line) and `fixtures::simian_clause` (a layer-4 condition
        // another layer-4 effect flips) are fixtures registered nowhere.
        registry.register("Kird Ape", arn::kird_ape);

        // Cost modification, one card per position in CR 601.2f's
        // order: an increase (pooled), a reduction, and the one printed card
        // that "directly affects the total cost". The cost fixtures — the
        // reducers, the lessons, the locked sphere — are registered nowhere.
        registry.register("Thalia, Guardian of Thraben", dka::thalia_guardian_of_thraben);
        registry.register("Goblin Electromancer", ddj::goblin_electromancer);
        registry.register("Trinisphere", dst::trinisphere);

        // Affinity for artifacts (CR 702.41a): the spell's own cost
        // ability, the gather's second source and its first dynamic amount.
        // Myr Enforcer is pooled; Frogmite is the second copy of the same
        // path, which buys a wider stress board and not a slower measured
        // one (`engineering-practices.md#cards-and-pools`).
        registry.register("Myr Enforcer", mrd::myr_enforcer);
        registry.register("Frogmite", mrd::frogmite);

        // Lock-in's payment side (CR 601.2h, 118.8b, 732.1)
        registry.register("Altar's Reap", isd::altars_reap);
        registry.register("Bone Splinters", ala::bone_splinters);
        registry.register("Thunderscape Familiar", pls::thunderscape_familiar);
        registry.register("Krark-Clan Ironworks", set_5dn::krark_clan_ironworks);
        registry.register("Foundry Inspector", kld::foundry_inspector);
        registry.register("Mind Stone", wth::mind_stone);

        // The damage event's two subjects and its results. Four printed
        // cards on two axes: which subject an effect is about, and what it does
        // to the amount. Furnace of Rath is pooled; the other three are the
        // stress pool's breadth — Ghosts of the Innocent is the halving arm and
        // the non-commuting board beside Furnace, Gisela carries a `PlayerSet`
        // twice on one card with both rounding directions, and Angel of
        // Suffering is the only prevention here and the only rider that rides
        // on a player subject.
        registry.register("Furnace of Rath", tmp::furnace_of_rath);
        registry.register("Ghosts of the Innocent", rav::ghosts_of_the_innocent);
        registry.register(
            "Gisela, Blade of Goldnight",
            avr::gisela_blade_of_goldnight,
        );
        registry.register("Angel of Suffering", snc::angel_of_suffering);
        // The CR 120.3c fixture, on the `graveyard_probe` convention: a
        // planeswalker with printed loyalty 3 and no abilities, because a
        // printed one would be a real name wearing three dead abilities.
        // Registered so a random agent's Lightning Bolt can find it — which is
        // what makes CR 704.5i reachable from a game — and pooled nowhere.
        registry.register("Loyalty Probe", fixtures::loyalty_probe);

        // CR 615.7 prevention shields. One card per shape a
        // resolution-created prevention effect takes: Mending Hands' targeted
        // count (pooled), Samite Healer's tapped-for count, Safe Passage's
        // filter asked at each event, and Samite Censer-Bearer's one row per
        // creature at resolution (CR 615.11).
        registry.register("Mending Hands", bok::mending_hands);
        registry.register("Samite Healer", lea::samite_healer);
        registry.register("Safe Passage", m10::safe_passage);
        registry.register("Samite Censer-Bearer", fut::samite_censer_bearer);

        // Sources. The axis is which half of CR 609.7's source
        // predicate each card writes: the Circle writes both, Reverse Damage
        // and Dark Sphere the chosen object alone, Guardian Seraph and Torbran
        // the property alone, Daunting Defender neither. Fog is the `combat`
        // flag's only consumer, and Pyroclasm is the other half of CR 615.10's
        // own example — the pool's first "each creature" damage, which is one
        // batch. Guardian Seraph is pooled; the rest are the stress pool's
        // breadth.
        registry.register(
            "Circle of Protection: Red",
            lea::circle_of_protection_red,
        );
        registry.register("Reverse Damage", lea::reverse_damage);
        registry.register("Dark Sphere", drk::dark_sphere);
        registry.register("Guardian Seraph", m10::guardian_seraph);
        registry.register("Daunting Defender", ons::daunting_defender);
        registry.register("Pyroclasm", ice::pyroclasm);
        registry.register("Fog", lea::fog);

        // Triggered abilities (CR 603), TR-1: the five cards of the spine.
        registry.register("Soul Warden", exo::soul_warden);
        registry.register("Blood Artist", avr::blood_artist);
        registry.register("Verdant Force", tmp::verdant_force);
        registry.register("Wild Growth", lea::wild_growth);
        registry.register("Felidar Sovereign", zen::felidar_sovereign);

        // TR-2a: the histories, the gates and each player.
        registry.register("Paladin of Atonement", rix::paladin_of_atonement);
        registry.register("Vengeful Warchief", m20::vengeful_warchief);
        registry.register("Elvish Warmaster", khm::elvish_warmaster);
        registry.register("Temple Bell", m11::temple_bell);

        // TR-2b: "may", CR 118.12's answer, and the departed frames.
        registry.register("Nykthos Paragon", mh2::nykthos_paragon);
        registry.register("Psychosis Crawler", mbs::psychosis_crawler);
        registry.register("Cosi's Trickster", zen::cosis_trickster);

        // TR-3a: the delayed-trigger registry. Neither is pooled: Final
        // Fortune for Time Walk's reason, an extra turn moves `Avg turns/game`
        // by design; Blessed Wine because §12 pools the registry with TR-3b.
        registry.register("Final Fortune", mir::final_fortune);
        registry.register("Blessed Wine", ice::blessed_wine);

        // TR-3b: the returns, and "until". Both pooled (§12).
        registry.register("Flickerwisp", eve::flickerwisp);
        registry.register("Banishing Light", jou::banishing_light);

        // TR-3c: the reflexive trigger. Not pooled: no pooled card has a
        // "when you do", so the performance A/B stays IDENTICAL.
        registry.register("Cornered Crook", mkm::cornered_crook);

        // TR-4a: the frame's facts. Strangleroot Geist is pooled (above);
        // Demonic Vigor and Compost are the two sides of CR 603.10's look-back,
        // which the tests carry.
        registry.register("Strangleroot Geist", dka::strangleroot_geist);
        registry.register("Demonic Vigor", dom::demonic_vigor);
        registry.register("Compost", uds::compost);

        // TR-4b: attachment's records. Grafted Wargear reads "becomes
        // unattached" on every route CR 701.3d counts, Bramble Elemental
        // "becomes attached". Bramble is pooled (above); the Wargear's doc
        // says why it is not.
        registry.register("Grafted Wargear", set_5dn::grafted_wargear);
        registry.register("Bramble Elemental", rav::bramble_elemental);

        // TR-4c: the other two look-back classes and the zone statement.
        // Bridge from Below states its graveyard zone and is pooled (above),
        // Guerrilla Tactics triggers on its own discard where it went, and
        // Angelic Destiny finds the card it became (CR 400.7f) beside Rancor,
        // whose trigger is on itself going to the graveyard (400.7e).
        registry.register("Bridge from Below", fut::bridge_from_below);
        registry.register("Guerrilla Tactics", all::guerrilla_tactics);
        registry.register("Angelic Destiny", m12::angelic_destiny);
        registry.register("Rancor", ulg::rancor);

        // TR-4d: control, countering and loss. Multani's Presence looks back at
        // who cast a countered spell (CR 603.10e); Ray of Command's delayed
        // trigger is on losing control, which its own steal ending at cleanup
        // makes (CR 603.10d, 514.3a).
        registry.register("Multani's Presence", ulg::multanis_presence);
        registry.register("Ray of Command", ice::ray_of_command);

        // TR-5a: the binding's roles. Sphere Grid's "that creature" is the
        // damage's source, which its instruction names (CR 608.2c), and its
        // grant counts the counters on each creature.
        registry.register("Sphere Grid", fic::sphere_grid);

        // TR-5e: the attack declaration. Hellrider triggers for each attacker
        // and deals its damage to what that attacker is attacking (CR 506.3).
        registry.register("Hellrider", dka::hellrider);

        // TR-5f: creatures put onto the battlefield attacking, and the combat's
        // history. Hanweir Garrison's tokens are attacking and never attacked
        // (CR 508.4); Relentless Assault untaps the creatures that attacked this
        // turn, read off the history's rows, for a second combat.
        registry.register("Hanweir Garrison", emn::hanweir_garrison);
        registry.register("Relentless Assault", vis::relentless_assault);

        // TR-5b: counters. Simic Ascendancy's put is read once for each
        // creature, and its win is an intervening "if" over its own counters;
        // Protean Hydra's X is its spell's (CR 107.3m), and each counter
        // removed from it triggers, CR 704.5q's annihilation included.
        registry.register("Simic Ascendancy", rna::simic_ascendancy);
        registry.register("Protean Hydra", m10::protean_hydra);

        // MA-2: the legal values of X. Blaze is the plain X spell, offered
        // at its least X and announced from what the board can pay.
        registry.register("Blaze", por::blaze);

        // TR-5c: blocks. Loyal Sentry triggers once for each creature it
        // blocks (CR 509.3b) and destroys that creature and itself together.
        registry.register("Loyal Sentry", s99::loyal_sentry);
        // Cephalid Aristocrat mills two each time it becomes the target of a
        // spell or ability, its controller's own included.
        registry.register("Cephalid Aristocrat", tor::cephalid_aristocrat);

        // TR-5d: prevention. Selfless Squire's entry prevents the damage dealt
        // to its controller this turn, and it grows by each prevention to them
        // (CR 615.13), its own or another's.
        registry.register("Selfless Squire", c16::selfless_squire);
        // The damage and excess a resolution dealt "this way": Goblin
        // Negotiation makes a Goblin for each point past lethal.
        registry.register("Goblin Negotiation", fdn::goblin_negotiation);

        // TR-5g: CR 603.2d's count. Panharmonicon doubles every triggered
        // ability of its controller's permanents that an artifact or
        // creature entering causes, its own entry and one beside it included.
        registry.register("Panharmonicon", kld::panharmonicon);
        // TR-6: CR 603.8's state check. "When you control no other
        // creatures" is asked of the board, never of an event.
        registry.register("Emperor Crocodile", uds::emperor_crocodile);
        // TR-7: the persist loop. With Blood Artist, sacrificing the
        // Scarecrow to the Seer under Grumgully returns it with two counters
        // that CR 704.5q removes, so it can persist again.
        registry.register("Grumgully, the Generous", eld::grumgully_the_generous);
        registry.register("Wingrattle Scarecrow", shm::wingrattle_scarecrow);
        registry.register("Viscera Seer", m11::viscera_seer);
        registry.register(
            "Torbran, Thane of Red Fell",
            eld::torbran_thane_of_red_fell,
        );

        // Redirection and unpreventable damage. The axis is where a
        // `Rewrite::Retarget` reads its destination: Pariah off the Aura's
        // host, Palisade Giant off its own source, Reflect Damage off the
        // *damage's* source's controller. Pinpoint Avalanche is the other
        // feature and shares none of that — CR 615.12's per-event flag, and the
        // only one of its nine printed cards the engine can play whole.
        // Pariah is pooled; the rest are the stress pool's breadth.
        registry.register("Pariah", usg::pariah);
        registry.register("Palisade Giant", rtr::palisade_giant);
        registry.register("Pinpoint Avalanche", ons::pinpoint_avalanche);
        registry.register("Reflect Damage", mir::reflect_damage);

        // Skips, and the turn queue. The axis is which CR 614.10 unit a
        // skip names and where the effect comes from: a static on you
        // (Yawgmoth's Bargain), a static on everyone (Eon Hub), a consumable
        // row on you (Meditate), a targeted row (Moment of Silence) — plus
        // Time Walk, which makes the turn the others skip. Eon Hub is pooled;
        // the rest are registered for the stress pool, and its comment in
        // `PERFORMANCE_POOL` says why each stays out.
        registry.register("Yawgmoth's Bargain", uds::yawgmoths_bargain);
        registry.register("Eon Hub", set_5dn::eon_hub);
        registry.register("Meditate", tmp::meditate);
        registry.register("Time Walk", lea::time_walk);
        registry.register("Moment of Silence", mmq::moment_of_silence);
        registry.register("Thought Reflection", shm::thought_reflection);
        registry.register(
            "Teferi's Ageless Insight",
            m21::teferis_ageless_insight,
        );
        registry.register("Alms Collector", c17::alms_collector);
        registry.register("Notion Thief", dgm::notion_thief);

        // Life. Six cards over three events: a gain doubled (Rhox
        // Faithmender, Alhammarret's Archive), a gain turned into a loss
        // (Tainted Remedy), a draw turned into a gain (Words of Worship), the
        // loss inside damage clamped (Ali from Cairo), and the gain refused
        // outright (Skullcrack). Rhox Faithmender is pooled; Leyline of
        // Punishment is deliberately unregistered and Skullcrack's doc says why.
        registry.register("Rhox Faithmender", m13::rhox_faithmender);
        registry.register("Tainted Remedy", ori::tainted_remedy);
        registry.register("Words of Worship", ons::words_of_worship);
        registry.register("Ali from Cairo", arn::ali_from_cairo);
        registry.register(
            "Alhammarret's Archive",
            ori::alhammarrets_archive,
        );
        registry.register("Skullcrack", gtc::skullcrack);

        // The game's end. Four cards on one axis: a draw turned into a
        // win (Laboratory Maniac), a loss replaced by a static (Exquisite
        // Archangel) or by a one-turn row (Stunning Reversal), and a loss
        // refused outright (Platinum Angel). Laboratory Maniac is pooled;
        // Platinum Angel is registered and not, and the Maniac's comment in
        // `PERFORMANCE_POOL` says why.
        registry.register("Laboratory Maniac", isd::laboratory_maniac);
        registry.register("Exquisite Archangel", aer::exquisite_archangel);
        registry.register("Stunning Reversal", bbd::stunning_reversal);
        registry.register("Platinum Angel", mrd::platinum_angel);

        // Tokens. Four cards on two axes: what makes a plural creation
        // (Raise the Alarm, Hordeling Outburst), what doubles one (Parallel
        // Lives), and what substitutes a token's entry (Hallowed Moonlight).
        // Parallel Lives and Raise the Alarm are pooled; `PERFORMANCE_POOL`
        // says why the other two stay out.
        registry.register("Parallel Lives", isd::parallel_lives);
        registry.register("Raise the Alarm", mrd::raise_the_alarm);
        registry.register("Hordeling Outburst", ktk::hordeling_outburst);
        registry.register("Hallowed Moonlight", ori::hallowed_moonlight);
        // The kind-changing substitution over a creation (Divine Visitation) and
        // a card whose two halves the draw and token watchers already cover
        // (Bard, King of Dale). Neither pooled.
        registry.register("Divine Visitation", grn::divine_visitation);
        registry.register("Bard, King of Dale", hob::bard_king_of_dale);

        // Counters. Six cards on two axes: which subject the effect
        // is around (a permanent, a player, or either) and what it does to
        // the count. Hardened Scales is pooled; `PERFORMANCE_POOL` says why
        // the other five stay out.
        registry.register("Doubling Season", rav::doubling_season);
        registry.register("Hardened Scales", ktk::hardened_scales);
        registry.register(
            "Vorinclex, Monstrous Raider",
            khm::vorinclex_monstrous_raider,
        );
        registry.register("Winding Constrictor", aer::winding_constrictor);
        registry.register("Live Fast", kld::live_fast);
        registry.register("Primal Vigor", c13::primal_vigor);

        // Discard and scry. Five cards on three axes: who picks a
        // discarded card (Mind Rot, Hymn to Tourach), what caused the discard
        // (Nephalia Academy, the one `ReplacementDef::by` in the registry),
        // and the scry both ways round (Opt makes one, Eligeth replaces one).
        // Mind Rot and Opt are pooled; `PERFORMANCE_POOL` says why the other
        // three stay out.
        registry.register("Mind Rot", por::mind_rot);
        registry.register("Hymn to Tourach", fem::hymn_to_tourach);
        registry.register("Nephalia Academy", emn::nephalia_academy);
        registry.register("Opt", inv::opt);
        registry.register(
            "Eligeth, Crossroads Augur",
            cmr::eligeth_crossroads_augur,
        );

        // Extra phases. Aggravated Assault is CR 500.8's only producer in
        // reach whole, and unpooled: eight mana to use once, and every
        // activation makes the game bigger.
        registry.register("Aggravated Assault", ons::aggravated_assault);

        // Mana. Three cards on two axes: CR 106.6a's multipliers (Mana
        // Reflection, pooled; Nyxbloom Ancient) and CR 106.12b's retyping
        // (Deep Water, the one of six printed type-changers that registers
        // whole).
        registry.register("Mana Reflection", shm::mana_reflection);
        registry.register("Nyxbloom Ancient", thb::nyxbloom_ancient);
        registry.register("Deep Water", drk::deep_water);
        registry.register("Pale Moon", nem::pale_moon);
        registry.register("Doubling Cube", set_5dn::doubling_cube);

        // The zone-reaching set. Yixlid Jailer is the printed consumer and
        // is pooled; Scarwood Treefolk is its partner in ATOM-614.12-001 and
        // opens no engine path, so it is registered and unpooled.
        // `fixtures::graveyard_painter` and `graveyard_reveler` — the
        // pair that proves a rule can *read* a zone-reaching change — are
        // fixtures registered nowhere, and so are `teferi_flash_clause` and
        // `lattice_colorless_clause`, the two rows item 181's measurement
        // places into a game: registering either would move every stress game.
        registry.register("Yixlid Jailer", fut::yixlid_jailer);
        registry.register("Scarwood Treefolk", tsp::scarwood_treefolk);

        // CR 113.6. Wonder is the registered card whose static
        // ability functions off the battlefield, and it is pooled: it opens
        // `move_object`'s registration path and puts a graveyard source in
        // front of the existence check.
        //
        // `fixtures::exiled_ancestor` — CR 113.6c's complement, which
        // needs no second mechanism — is a fixture registered nowhere, for the
        // reason `registry.rs` gives elsewhere: the printed 113.6c population
        // is thin and every member of it wants something else as well.
        registry.register("Wonder", jud::wonder);

        // RF — the gather's zone leg (`replacement-architecture.md` §3.3
        // source 2): a static *replacement* ability functioning off the
        // battlefield. Two shapes: a permanent card whose clause the library,
        // the hand and the stack all see, and an instant that is never a
        // permanent at all. Darksteel Colossus's doc says why neither is
        // Blightsteel.
        // The fixtures beside them — `timid_golem`, `hollow_hands`,
        // `sealing_ward` — are the CR 113.6 default, the effective-list read
        // and the affected side's zone check, and are registered nowhere.
        registry.register("Darksteel Colossus", dst::darksteel_colossus);
        registry.register("Nexus of Fate", m19::nexus_of_fate);

        // A4i — CR 601.2c's three shapes of "several instances of target",
        // each card's doc saying which it presses on; Seat of the Synod is
        // here because the CR's own example of one object satisfying two
        // clauses is an artifact land.
        registry.register("Seeds of Strength", rav::seeds_of_strength);
        registry.register("Incremental Growth", lrw::incremental_growth);
        registry.register("Jagged Lightning", p02::jagged_lightning);
        registry.register("Plague Spores", inv::plague_spores);
        registry.register("Seat of the Synod", mrd::seat_of_the_synod);
        // CR 700.2's modes: a choice of one the board narrows, a choice of
        // two, and one or more of five.
        registry.register("Abrade", hou::abrade);
        registry.register("Cryptic Command", lrw::cryptic_command);
        registry.register("Casualties of War", war::casualties_of_war);

        for (name, factory) in IN_DEVELOPMENT {
            registry.register_in_development(name, factory);
        }

        registry
    }

    /// The subset an engine change is A/B'd against — see [`PERFORMANCE_POOL`]
    /// for why it grows rather than stays frozen.
    ///
    /// Panics if a name in `PERFORMANCE_POOL` is no longer registered. That is
    /// the point: a rename that silently shrank the pool would leave two runs
    /// incomparable without saying so, which is the one failure a *deliberate*
    /// addition does not have.
    pub fn performance_pool() -> Self {
        let all = Self::default_registry();
        let mut registry = CardRegistry::new();
        for name in PERFORMANCE_POOL {
            let factory = all.cards.get(name).unwrap_or_else(|| {
                panic!("performance pool names {name:?}, which is no longer registered")
            });
            registry.register(name, *factory);
        }
        registry
    }
}

impl CardRegistry {
    /// Every registered card and the castable fixtures: `fuzz_games --pool
    /// fixtures`, which the checked sitting plays, where the debug checks the
    /// fixtures' builds carry fire
    /// (`engineering-practices.md#cards-and-pools`). A fixture is no printed
    /// card, so it is listed apart and never joins `default_registry`, whose
    /// numbers stay real cards'.
    pub fn fixture_pool() -> Self {
        let mut pool = Self::default_registry();
        for (name, fixture) in crate::cards::fixtures::CASTABLE {
            pool.register(name, fixture);
        }
        pool
    }
}

impl Default for CardRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::card_types::CardType;

    /// The board editor's search lists both lists, and only scenarios read
    /// the second, so neither holds the other's names.
    #[test]
    fn the_names_in_development_are_listed_apart_and_sorted() {
        // Not named `registry`: `check_state_of_play.py` counts that name's
        // `register` calls as the cards registered.
        let mut lists = CardRegistry::new();
        lists.register("Forest", lea::forest);
        lists.register_in_development("Zebra Bear", lea::forest);
        lists.register_in_development("Ape Bear", lea::forest);
        assert_eq!(lists.names_in_development(), ["Ape Bear", "Zebra Bear"]);
        assert_eq!(lists.card_names(), ["Forest"]);
    }

    #[test]
    fn test_default_registry_has_basic_lands() {
        let registry = CardRegistry::default_registry();

        for name in &["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            let card = registry.create(name).unwrap();
            assert_eq!(card.name, *name);
            assert!(card.types.contains(&CardType::Land));
        }
    }

    #[test]
    fn test_registry_unknown_card() {
        let registry = CardRegistry::default_registry();
        assert!(registry.create("Black Lotus").is_err());
    }

    #[test]
    fn test_every_performance_pool_name_still_resolves() {
        // `performance_pool` panics on a name it cannot find, so building it is
        // the check. The count is asserted against the array's own length
        // rather than a literal: the pool grows by deliberate act now, and a
        // hardcoded number would fail the *addition* instead of failing the
        // thing worth catching, which is a name that stopped resolving.
        let pool = CardRegistry::performance_pool();
        assert_eq!(pool.card_names().len(), PERFORMANCE_POOL.len());
    }

    #[test]
    fn test_stress_pool_is_a_superset_of_the_performance_pool() {
        // The two pools share definitions rather than duplicating them, so the
        // only way to fail this is to unregister a card the performance list
        // names.
        let all = CardRegistry::default_registry();
        for name in CardRegistry::performance_pool().card_names() {
            assert!(all.create(name).is_ok(), "{name:?} left the full registry");
        }
        assert!(
            all.card_names().len() > PERFORMANCE_POOL.len(),
            "the stress pool has stopped being a strict superset"
        );
    }

    /// `AbilityType::Triggered` and `Effect::Triggered` are one fact written
    /// twice: the tag is what priority and display key on, the effect is what
    /// the trigger dispatcher reads, through `Effect::as_trigger`, which sees
    /// past a statement of where it functions. `authoring::triggered_ability`
    /// writes both and is the only constructor of either, so the dispatcher
    /// asks only the effect. This is what makes that safe for every card,
    /// printed or nested.
    #[test]
    fn a_triggered_tag_and_a_triggered_effect_always_come_together() {
        use crate::objects::card_data::{AbilityDef, AbilityType};

        fn check(what: &str, def: &AbilityDef) {
            assert_eq!(
                def.ability_type == AbilityType::Triggered,
                def.effect.as_trigger().is_some(),
                "{what}: the ability's tag and its effect disagree"
            );
        }

        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        for name in names {
            let card = (registry.cards[name.as_str()])();
            let mut copy = (*card).clone();
            for (i, ability) in Arc::make_mut(&mut copy.abilities).iter_mut().enumerate() {
                check(&format!("{name} ability #{i}"), ability);
                let mut nested = 0usize;
                ability.effect.for_each_ability_def_mut(&mut |def| {
                    check(&format!("{name} ability #{i}, nested def #{nested}"), def);
                    nested += 1;
                });
            }
        }
    }

    /// Every registered trigger multiplier is one the count can read (CR
    /// 603.2d): an event or none, and at least one more time.
    #[test]
    fn every_trigger_multiplier_names_an_event_and_a_count() {
        use crate::objects::card_data::AbilityDef;
        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        for name in names {
            let card = (registry.cards[name.as_str()])();
            let mut copy = (*card).clone();
            for (i, ability) in Arc::make_mut(&mut copy.abilities).iter_mut().enumerate() {
                let check = |def: &AbilityDef| {
                    for (_, multiplier) in def.effect.trigger_multipliers() {
                        assert_eq!(multiplier.refusal(), None, "{name} ability #{i}");
                    }
                };
                check(ability);
                ability.effect.for_each_ability_def_mut(&mut |def: &mut AbilityDef| check(def));
            }
        }
    }

    /// CR 113.6b's statement of where a static ability functions wraps the
    /// ability's whole effect, so every reader finds it by one peel
    /// (`Effect::with_its_condition`): an `Effect::FunctionsIn` anywhere but
    /// the root of an ability's effect would be read by none of them.
    #[test]
    fn every_zone_statement_is_at_its_abilitys_root() {
        use crate::objects::card_data::AbilityDef;
        use crate::types::effects::Effect;

        fn inner_statements(effect: &Effect) -> usize {
            let mut found = 0;
            let mut stack: Vec<&Effect> = vec![effect];
            while let Some(effect) = stack.pop() {
                match effect {
                    Effect::FunctionsIn(_, inner) => {
                        found += 1;
                        stack.push(inner);
                    }
                    Effect::Sequence(subs) => stack.extend(subs.iter()),
                    Effect::Modal(node) => stack.extend(node.modes.iter().map(|mode| &mode.effect)),
                    Effect::Conditional(_, inner)
                    | Effect::Optional { effect: inner, .. }
                    | Effect::Remember(inner)
                    | Effect::ForEach(_, inner)
                    | Effect::Repeat(_, inner) => stack.push(inner),
                    Effect::Triggered(def) => stack.push(&def.effect),
                    Effect::Atom(..)
                    | Effect::Replacement(_)
                    | Effect::Restriction(_)
                    | Effect::CostModification(_)
                    | Effect::TriggerMultiplier(_) => {}
                }
            }
            found
        }
        fn check(what: &str, def: &AbilityDef) {
            let below_the_root = match &def.effect {
                Effect::FunctionsIn(_, body) => inner_statements(body),
                effect => inner_statements(effect),
            };
            assert_eq!(below_the_root, 0, "{what}: a zone statement below its ability's root");
        }

        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        for name in names {
            let card = (registry.cards[name.as_str()])();
            let mut copy = (*card).clone();
            for (i, ability) in Arc::make_mut(&mut copy.abilities).iter_mut().enumerate() {
                check(&format!("{name} ability #{i}"), ability);
                let mut nested = 0usize;
                ability.effect.for_each_ability_def_mut(&mut |def: &mut AbilityDef| {
                    check(&format!("{name} ability #{i}, nested def #{nested}"), def);
                    nested += 1;
                });
            }
        }
    }

    /// A modal node is all of its ability (`Effect::modal`): the announcement
    /// asks its chosen modes' targets after the effect's own, and resolution
    /// meets them where the node sits, so a node beside other instructions or
    /// inside a mode would be announced in one order and resolved in another.
    /// A spell is asked as a whole, since its paragraphs are one effect.
    #[test]
    fn every_modal_node_is_at_its_abilitys_root() {
        use crate::objects::card_data::AbilityDef;
        use crate::types::effects::Effect;

        fn nodes(effect: &Effect) -> usize {
            let mut found = 0;
            let mut stack: Vec<&Effect> = vec![effect];
            while let Some(effect) = stack.pop() {
                match effect {
                    Effect::Modal(node) => {
                        found += 1;
                        stack.extend(node.modes.iter().map(|mode| &mode.effect));
                    }
                    Effect::Sequence(subs) => stack.extend(subs.iter()),
                    Effect::Conditional(_, inner)
                    | Effect::FunctionsIn(_, inner)
                    | Effect::Optional { effect: inner, .. }
                    | Effect::Remember(inner)
                    | Effect::ForEach(_, inner)
                    | Effect::Repeat(_, inner) => stack.push(inner),
                    Effect::Triggered(def) => stack.push(&def.effect),
                    Effect::Atom(..)
                    | Effect::Replacement(_)
                    | Effect::Restriction(_)
                    | Effect::CostModification(_)
                    | Effect::TriggerMultiplier(_) => {}
                }
            }
            found
        }
        fn check(what: &str, effect: &Effect) {
            let at_the_root = usize::from(effect.modal().is_some());
            assert_eq!(nodes(effect), at_the_root, "{what}: a modal node below its ability's root");
        }

        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        for name in names {
            let card = (registry.cards[name.as_str()])();
            if let Some(effect) = crate::objects::card_data::spell_effect(&card.abilities) {
                check(&format!("{name} as a spell"), &effect);
            }
            let mut copy = (*card).clone();
            for (i, ability) in Arc::make_mut(&mut copy.abilities).iter_mut().enumerate() {
                check(&format!("{name} ability #{i}"), &ability.effect);
                let mut nested = 0usize;
                ability.effect.for_each_ability_def_mut(&mut |def: &mut AbilityDef| {
                    check(&format!("{name} ability #{i}, nested def #{nested}"), &def.effect);
                    nested += 1;
                });
            }
        }
    }

    /// Every event role a registered trigger names is one every arm of its
    /// condition has (TR-5a's decision 2): "that creature" after a dies
    /// trigger is the card that moved, and no dies record has a recipient. A
    /// delayed trigger's def, which its creating instruction carries, is
    /// checked against its own arms.
    #[test]
    fn every_trigger_names_event_roles_its_arms_have() {
        use crate::objects::card_data::AbilityDef;
        use crate::types::effects::{Effect, Primitive};
        use crate::types::triggers::TriggerDef;

        fn check(what: &str, def: &TriggerDef) {
            assert!(
                def.event_roles_its_arms_lack().is_empty(),
                "{what}: names {:?}, which an arm of its condition does not have",
                def.event_roles_its_arms_lack()
            );
            assert!(
                def.readings_its_arms_cannot_give().is_empty(),
                "{what}: asks {:?}",
                def.readings_its_arms_cannot_give()
            );
        }
        fn walk(what: &str, effect: &Effect) {
            match effect {
                Effect::Triggered(def) => {
                    check(what, def);
                    walk(what, &def.effect);
                }
                Effect::Atom(Primitive::CreateDelayedTrigger(template), _) => {
                    check(&format!("{what}, its delayed trigger"), &template.def);
                    walk(what, &template.def.effect);
                }
                Effect::Sequence(effects) => effects.iter().for_each(|e| walk(what, e)),
                Effect::Modal(node) => node.modes.iter().for_each(|mode| walk(what, &mode.effect)),
                Effect::Conditional(_, inner)
                | Effect::FunctionsIn(_, inner)
                | Effect::Optional { effect: inner, .. }
                | Effect::Remember(inner)
                | Effect::ForEach(_, inner)
                | Effect::Repeat(_, inner) => walk(what, inner),
                Effect::Atom(..)
                | Effect::Replacement(_)
                | Effect::Restriction(_)
                | Effect::CostModification(_)
                | Effect::TriggerMultiplier(_) => {}
            }
        }

        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        for name in names {
            let card = (registry.cards[name.as_str()])();
            let mut copy = (*card).clone();
            for (i, ability) in Arc::make_mut(&mut copy.abilities).iter_mut().enumerate() {
                walk(&format!("{name} ability #{i}"), &ability.effect);
                let mut nested = 0usize;
                ability.effect.for_each_ability_def_mut(&mut |def: &mut AbilityDef| {
                    walk(&format!("{name} ability #{i}, nested def #{nested}"), &def.effect);
                    nested += 1;
                });
            }
        }
    }

    /// Every ability a registered card carries says what its card says. A
    /// printed ability's text is exactly one paragraph of the card's rules
    /// text (CR 113.2c), a modal ability's bullets with it (CR 700.2), and
    /// the paragraph the builder found for it. Several abilities may share
    /// one: CR 603.11 prints a static ability and the triggered abilities
    /// linked to it as one paragraph. A granted ability or a
    /// token's is the text its card quotes, and a granted keyword ability is
    /// its name, which the card names (CR 702.1: Wingrattle Scarecrow's "has
    /// persist"). The exception
    /// is CR 305.6's ability, which a land has from its basic land type
    /// whatever its text box prints. A card's rules text carries no reminder
    /// text, which has no game function (CR 207.2); every parenthesis in a
    /// registered card's Oracle text is reminder text.
    #[test]
    fn every_ability_carries_its_cards_text() {
        use crate::engine::layers::land_types::intrinsic_mana_ability;
        use crate::objects::card_data::paragraphs;
        use crate::types::card_types::Subtype;
        use crate::types::ids::new_object_id;

        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        for name in names {
            let card = (registry.cards[name.as_str()])();
            let printed = &card.rules_text;
            assert!(!printed.contains('('), "{name}'s rules text carries reminder text: {printed:?}");
            let intrinsic: Vec<&str> = card
                .subtypes
                .iter()
                .filter_map(|subtype| match subtype {
                    Subtype::Land(land_type) if card.types.contains(&CardType::Land) => {
                        intrinsic_mana_ability(new_object_id(), *land_type).map(|a| a.rules_text.words)
                    }
                    _ => None,
                })
                .collect();
            let check = |what: String, text: &str, belongs: bool| {
                assert!(!text.is_empty(), "{what} has no text");
                assert!(belongs, "{what}'s text {text:?} is not its card's: {:?}", card.rules_text);
            };
            let mut copy = (*card).clone();
            for (i, ability) in Arc::make_mut(&mut copy.abilities).iter_mut().enumerate() {
                let text = ability.rules_text.words;
                let belongs = match ability.rules_text.paragraph() {
                    Some(at) => paragraphs(printed).nth(usize::from(at)) == Some(text),
                    None => intrinsic.contains(&text),
                };
                check(format!("{name} ability #{i}"), text, belongs);
                let mut nested = 0usize;
                ability.effect.for_each_ability_def_mut(&mut |def| {
                    let text = def.rules_text.words;
                    let quoted = printed.contains(&format!("\"{text}\""));
                    // A keyword's name is not a sentence, so it carries no period.
                    let named_keyword = !text.contains('.') && printed.to_lowercase().contains(&text.to_lowercase());
                    check(format!("{name} ability #{i}, nested def #{nested}"), text, quoted || named_keyword);
                    nested += 1;
                });
            }
        }
    }
}

#[cfg(test)]
mod instance_references {
    use super::*;
    use crate::types::effects::{Effect, EffectRecipient};

    /// Every `EffectRecipient::SameInstanceAs(ix)` a registered card writes names an
    /// instance of "target" that the same effect declares (CR 115.3).
    ///
    /// **An authoring check, not a rules one.** A card that writes
    /// `Instance(1)` where its effect declares one clause resolves to an error
    /// deep inside `resolve_effect` — at the moment the spell is already on the
    /// stack and paid for. Caught here it is a typo; caught there it is a game
    /// that cannot finish. The walk is `Effect::instances`' own, so the two
    /// cannot disagree about what counts as a declaration.
    #[test]
    fn every_instance_back_reference_names_a_declared_clause() {
        fn check(what: &str, effect: &Effect) {
            // A mode is its own instance scope, and a modal node is all of its
            // ability (`every_modal_node_is_at_its_abilitys_root`).
            if let Some(node) = effect.modal() {
                for (place, mode) in node.indexed_modes() {
                    check(&format!("{what}, mode {place}"), &mode.effect);
                }
                return;
            }
            let declared = effect.instances().len();
            for_each_atom(effect, &mut |_, recipient| {
                if let EffectRecipient::SameInstanceAs(ix) = recipient {
                    assert!(*ix < declared, "{what} refers to instance {ix} of \"target\" but declares {declared}");
                }
            });
        }

        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        for name in names {
            let card = (registry.cards[name.as_str()])();
            for (i, ability) in card.abilities.iter().enumerate() {
                if ability.ability_type != crate::objects::card_data::AbilityType::Spell {
                    check(&format!("{name} ability #{i}"), &ability.effect);
                }
            }
            // A spell's instances are counted across its abilities, so a
            // later paragraph names an earlier one's target by that count.
            if let Some(effect) = crate::objects::card_data::spell_effect(&card.abilities) {
                check(&format!("{name} as a spell"), &effect);
            }
        }
    }

    /// Every instruction of `effect`, with its recipient.
    fn for_each_atom<'a>(effect: &'a Effect, f: &mut impl FnMut(&'a crate::types::effects::Primitive, &'a EffectRecipient)) {
        let mut stack = vec![effect];
        while let Some(e) = stack.pop() {
            match e {
                Effect::Atom(primitive, recipient) => f(primitive, recipient),
                Effect::Sequence(subs) => stack.extend(subs.iter()),
                Effect::Modal(node) => stack.extend(node.modes.iter().map(|mode| &mode.effect)),
                Effect::Conditional(_, inner)
                | Effect::FunctionsIn(_, inner)
                | Effect::Optional { effect: inner, .. }
                | Effect::Remember(inner)
                | Effect::ForEach(_, inner)
                | Effect::Repeat(_, inner) => stack.push(inner),
                Effect::Triggered(def) => stack.push(&def.effect),
                Effect::Replacement(_) | Effect::Restriction(_) | Effect::CostModification(_) | Effect::TriggerMultiplier(_) => {}
            }
        }
    }

    /// CR 601.2d's total has two homes: the clause carries it to the
    /// announcement and the atom prints it. Nothing reads the atom's copy, so
    /// this is what keeps them one number, on every card and castable fixture.
    /// A shield prints none, each row's count being its target's share.
    #[test]
    fn every_divided_atom_prints_what_its_clause_divides() {
        use crate::types::effects::{Primitive, TargetCount};
        let pool = CardRegistry::fixture_pool();
        let mut names: Vec<String> = pool.cards.keys().cloned().collect();
        names.sort();
        let mut divided = 0;
        for name in names {
            for ability in (pool.cards[name.as_str()])().abilities.iter() {
                for_each_atom(&ability.effect, &mut |primitive, recipient| {
                    let EffectRecipient::Target(_, TargetCount::Divided(clause)) = recipient else { return };
                    divided += 1;
                    match primitive {
                        Primitive::DealDamage { amount, .. } | Primitive::AddCounters { amount, .. } => {
                            assert_eq!(amount, &clause.total, "{name} prints one amount and divides another");
                        }
                        Primitive::CreateReplacement(..) => {}
                        other => panic!("{name} divides {other:?}, which reads no share"),
                    }
                });
            }
        }
        assert!(divided >= 8, "the walk found {divided} divided atoms, and Arc Lightning and seven fixtures print one each");
    }

    /// A second change in one sentence ("gets +1/+0 and gains first strike
    /// until end of turn") acts on the target its first change declared.
    ///
    /// `authoring::ContinuousChange` writes the second change's back-reference as the
    /// effect's first instance of "target", which is right only when the
    /// chain's own target is that first one. A chain after another target
    /// names the wrong object and still names a declared one, so
    /// `every_instance_back_reference_names_a_declared_clause` passes it:
    /// this one reads the shape the chain leaves, a
    /// declaring change and then a change of the same duration on
    /// `SameInstanceAs`, and asks that the reference is to the instance the
    /// change before it declared.
    #[test]
    fn a_chained_change_refers_to_the_target_its_first_change_declared() {
        use crate::types::effects::{Duration, Primitive};

        fn duration(primitive: &Primitive) -> Option<Duration> {
            match primitive {
                Primitive::SetPowerToughness(_, _, d)
                | Primitive::ModifyPowerToughness(_, _, d)
                | Primitive::SwitchPowerToughness(d)
                | Primitive::GrantKeywordFlag(_, d)
                | Primitive::RemoveKeywordFlag(_, d)
                | Primitive::GrantAbility(_, d)
                | Primitive::LoseAbility(_, d)
                | Primitive::LoseAllAbilities(d)
                | Primitive::ChangeColor(_, d)
                | Primitive::ChangeType(_, d)
                | Primitive::GainControl { duration: d, .. }
                | Primitive::Copy { duration: d, .. }
                | Primitive::Restrict(_, d)
                | Primitive::CreateReplacement(_, d, _) => Some(*d),
                _ => None,
            }
        }
        fn check(what: &str, whole: &Effect, effect: &Effect, chains: &mut usize) {
            let declared_at = |recipient: &EffectRecipient| {
                (0..).map_while(|ix| whole.instance(ix).map(|r| (ix, r))).find(|(_, r)| std::ptr::eq(*r, recipient)).map(|(ix, _)| ix)
            };
            match effect {
                Effect::Sequence(subs) => {
                    for pair in subs.windows(2) {
                        if let [Effect::Atom(first, recipient), Effect::Atom(second, EffectRecipient::SameInstanceAs(ix))] = pair
                            && let Some(own) = declared_at(recipient)
                            && duration(first).is_some()
                            && duration(first) == duration(second)
                        {
                            assert_eq!(*ix, own, "{what}: a change chained to instance {own} of \"target\" names instance {ix}");
                            *chains += 1;
                        }
                    }
                    subs.iter().for_each(|sub| check(what, whole, sub, chains));
                }
                Effect::Conditional(_, inner)
                | Effect::FunctionsIn(_, inner)
                | Effect::Optional { effect: inner, .. }
                | Effect::Remember(inner) => check(what, whole, inner, chains),
                Effect::Triggered(def) => check(what, &def.effect, &def.effect, chains),
                _ => {}
            }
        }

        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        let mut chains = 0;
        for name in names {
            let card = (registry.cards[name.as_str()])();
            for (i, ability) in card.abilities.iter().enumerate() {
                if ability.ability_type != crate::objects::card_data::AbilityType::Spell {
                    check(&format!("{name} ability #{i}"), &ability.effect, &ability.effect, &mut chains);
                }
            }
            if let Some(effect) = crate::objects::card_data::spell_effect(&card.abilities) {
                check(&format!("{name} as a spell"), &effect, &effect, &mut chains);
            }
        }
        // Crimson Wisps' "becomes red and gains haste" is one.
        assert!(chains > 0, "no registered card chains a change to a target, so this check reads nothing");
    }

    /// For every registered card, the stored lists of instances of "target"
    /// (CR 115.3) — `AbilityDef::instances` on each def, `CardData::spell_instances`
    /// on the card — equal what `Effect::instances` derives from the effect tree.
    ///
    /// **The gate on a derived field.** `AbilityDef::instances` and
    /// `CardData::spell_instances` are written once by `CardDataBuilder::build`
    /// and read on every castability check; a def that reached a card by some
    /// route the builder does not walk would carry an empty list and announce
    /// nothing, silently. So every def a registered card carries — printed and
    /// nested — is re-derived here and compared, and the spell-level list is
    /// re-derived by the Aura rule (CR 303.4a) independently of the builder.
    #[test]
    fn every_stored_instance_list_matches_the_walk() {
        use crate::types::effects::TargetCount;

        fn check(what: &str, def: &crate::objects::card_data::AbilityDef) {
            assert_eq!(
                def.instances(),
                def.effect.instances(),
                "{what}: the stored instance list is not the walk's"
            );
            for (place, mode) in def.effect.modal().into_iter().flat_map(|node| node.indexed_modes()) {
                assert_eq!(mode.instances, mode.effect.instances(), "{what}, mode {place}: the stored instance list is not the walk's");
            }
        }

        let registry = CardRegistry::default_registry();
        let mut names: Vec<String> = registry.cards.keys().cloned().collect();
        names.sort();
        for name in names {
            let card = (registry.cards[name.as_str()])();
            for (i, ability) in card.abilities.iter().enumerate() {
                check(&format!("{name} ability #{i}"), ability);
            }
            // The nested defs — `for_each_ability_def_mut` is the only walk over
            // them, so it runs over a copy.
            let mut nested = 0usize;
            let mut copy = (*card).clone();
            for (i, ability) in Arc::make_mut(&mut copy.abilities).iter_mut().enumerate() {
                ability.effect.for_each_ability_def_mut(&mut |def| {
                    check(&format!("{name} ability #{i}, nested def #{nested}"), def);
                    nested += 1;
                });
            }
            let expected = match &card.enchant_filter {
                Some(filter) => {
                    vec![EffectRecipient::Target(filter.clone(), TargetCount::Exactly(1))]
                }
                None => crate::objects::card_data::spell_effect(&card.abilities)
                    .map(|effect| effect.instances())
                    .unwrap_or_default(),
            };
            assert_eq!(
                card.spell_instances(), expected,
                "{name}: the stored spell-level instance list is not the card's"
            );
        }
    }
}

