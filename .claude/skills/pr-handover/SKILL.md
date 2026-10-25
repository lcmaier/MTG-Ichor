---
name: pr-handover
description: Close a build and hand the PR to the owner - size it, run only the gates that can fail, pre-review it with the owner-review agent, open the PR from the template, and give the owner the list of changes by concept. Use when a build's last code commit is in.
---

# Handing a PR over

The owner reviews every line, and a handover is good when that review is
spent on rules and design. Do these in order.

## 1. Size

`git diff --numstat <base> HEAD`: report code plus tests (`mtgsim/src`,
`mtgsim/tests`), the band's measure, and the total with docs. Past 2,500 in
code plus tests, say so before anything else
(`plans/engineering-practices.md#sizing-a-phase`).

## 2. The gates that can fail here

Run what the change can break, and name each gate skipped with why it cannot
fail:

- **Engine code changed:** `cargo build --all-targets` with zero warnings,
  `cargo test`, `cargo clippy --all-targets -- -D warnings`, and the
  close-out's measurements (`plans/engineering-practices.md#the-close-out`).
- **Docs or tooling only:** the `check_*.py` family in `CLAUDE.md`'s Commands.
- **The dev GUI:** only when `devgui/` changed, an engine type it matches on
  changed, the A/B moved two-seat games, or a card was registered or pooled.
- **A fix of a few lines:** its own test shown failing first, then the
  module's tests; CI runs the rest.

## 3. Pre-review

Launch the `owner-review` agent with the base commit and the design note's
path, and nothing about why the code is the way it is. For each finding:
fix a defect (its own commit when it is its own concept), answer a false
alarm in a line, and put a genuine choice to the owner with its tradeoffs.
Record all of it in the PR body's "Review before handover" section.

## 4. Open the PR

Push the code commits, then fill `.github/pull_request_template.md` section by
section and open the PR with `gh pr create --body-file`. Write the PR number
into any doc only after the PR exists. Lead with engine changes and their
`file:line` when the PR also touches the dev GUI.

## 5. In chat

The same list of changes by concept as the PR body: for each, what was wrong
with a concrete card or board, what changed, and why, with item numbers only
in brackets at the end. Then the size, the gates, what the pre-review caught,
and the PR URL. Merging is the owner's call; do not offer to watch CI.
