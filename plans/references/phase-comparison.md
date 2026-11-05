# What the `phase-rs/phase` engine offers later steps

`phase-rs/phase` is another open-source Magic rules engine in Rust, built around
an Oracle text parser. It was compared with Ichor on 2026-11-04, at its commit
`3f8ec71a`, and paths and numbers below are from that commit. Each section is for
one step's author, read at that step's design stop (`/design-note` points here):
what phase does, and why it may apply.

These are inputs, not decisions, and none is rules authority: the Comprehensive
Rules stay the customer, and phase's habit of treating real cards as the
authority is one of the things not to take.

## `pausing-at-a-decision`

- **An answer names the prompt it answers.** phase digests each prompt it delivers
  (`delivered_prompt_identity`, `crates/engine/src/game/derived_views.rs`). A
  client answering late or twice can then be refused, where otherwise its answer
  would land on whatever prompt came next.
- **An entry for leaving outside a prompt.** `answer` serves only the seat asked,
  and CR 104.3a lets any player concede at any time (`backlog.md` 2.53).
- **phase's orphaned frames, as a test list.** phase keeps what a pause must resume
  in frames, and frames have been stranded: #7384 (a proliferate frame orphaned by
  a counter replacement), #5172 (pending triggers on an empty batch) and #8344
  (trigger ordering during a live resolution frame). Each is a board to try
  against a paused game.

## `information-model-design`

- **Build a seat's view by addition.** phase removes what a seat may not see, so a
  new field leaks until someone remembers to remove it (phase's #9542 and #6064).
  Starting empty and adding what a seat may see fails closed.
- **Keep the client's structure apart from internal state**, so an internal
  reshape is not a wire change.
- **Knowledge facts stamped with an epoch.** A shuffle calls
  `advance_library_knowledge_epoch`, which voids what was known about that library
  at once and spares a sweep over each fact.
- **Random draws named by kind**: `state.rng.draw(RandomDraw::Outcome)`. A draw
  says what it is at the call, which a replay or a loop check can read.

## `permission-design`

- **`CastingPermission` as a checklist** (`crates/phase-ast/src/types/ability.rs`).
  Its `ExileWithAltCost` variant carries who may cast, the cost's provenance, a
  cast-time constraint and how it is cleaned up. A permission type here should
  answer all four.

## `double-faced-cards` and the card model's builds

- **phase's extra copiable facts.** Room halves (`RoomCopiableHalves`), prepare
  faces and flip status each had to be carried by a copy. Check each against CR
  707.2 when the card model fixes what a copy holds.

## `tarkir-dragonstorm-decks`

- **Skills per kind of change.** Ichor's four skills are stages of a PR. Many of
  phase's 31 are checklists for one kind of change, such as add-keyword or
  add-trigger, each listing every registration point. Mass card work is where
  those pay.
- **Single-gap ranking on the deck census**: rank each missing feature by the
  cards it alone would unlock.
- **A per-card structural snapshot diff in PRs**, so a change to how cards lower
  shows up card by card.

## `mana-payment-solver`

- **A mode that won't auto-pay with sacrificial mana** (`CastPaymentMode::AutoExceptSacrificialMana`),
  so a solver never spends a permanent the player would not have chosen to lose.
- **Remembered replacement answers as a provider decorator**, so a repeated choice
  is asked once, with no engine change.
- **Cast reversal** (`SpellCastReversal`, `crates/engine/src/game/casting.rs`), to
  read beside the mana-ability gap, the ironworks-loop item.

## CR 113.11's "can't have"

- **Placement at the end of Layer 6, repeated after keyword counters**
  (`crates/engine/src/game/layers.rs`), for whichever step takes this rule.

## The first real AI agent

- **Paired-seed flip reports on non-mirror matchups**: `ai_duel` reports which
  seeds flipped between two arms, with a sign-test p-value. Count a panic as a
  failure, not a draw as phase does.
- **A verdict of reject or score** (`crates/phase-ai/src/config.rs`), so a candidate
  that breaks a rule is out, never merely scored low.
- **An LLM seat** that chooses among engine-issued options, revalidates the choice,
  falls back when it fails, and rejects a stale answer by fingerprint.
