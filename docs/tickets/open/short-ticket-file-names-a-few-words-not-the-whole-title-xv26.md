---
id: br-xv26
title: "Short ticket file names: a few words, not the whole title"
kind: feature
created: 2026-11-18
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Ticket file names are short: a couple of words plus the ID (`flume-clone-pshd.md`), not the title cut at 60 characters. Today `bridle ticket new` builds the slug from the whole title, so names like `flume-clone-set-up-a-whole-workspace-from-the-tasks-repo-flu-pshd.md` are the norm.

The human, 2026-11-17 ~9:30 PM ET (via the flume aide, submitted by aide@remus), verbatim: "these ticket names are wildly too long; what is the guidance on them? flume-clone-set-up-a-whole-workspace-from-the-tasks-repo-flu ??? it should just be flume-clone-[id]"

## Design (pm-1 settles the proposal; the worker builds this)

- `bridle ticket new` (and `ticket new --from-task`, and wherever else a file is minted, e.g. `submit` accept path) takes `--slug <words>`: an explicit short name. Validate: lowercase ASCII letters, digits and `-`, no leading or trailing `-`, at most 24 characters; refuse otherwise with the rule in the message.
- Without `--slug`, derive a much shorter slug from the title: everything before the first `:` when the title has one ("flume clone: set up ..." gives `flume-clone`); otherwise the first 3 words. Cap the derived slug at 24 characters on a word boundary. The file is `<slug>-<id>.md`.
- Existing files keep their names (a name is set once). No rename, no migration: this changes only how new files are named. Say so in the CHANGELOG.
- Rule `tickets` (workflow/base/rules/tickets.md) tells filers to pass a short `--slug`; edit that existing text, do not add a rule (the human on 2026-11-17: modify the existing rule, don't add one). The `ticket-references` rule leads with the file name's first letters, which a short slug serves better; no change needed there.
- Not now (yagni): a command to rename old files; a title-length limit.

## Files (the worker confirms)

`bridle ticket new` in crates/bridle/src (cli.rs, the ticket command module, and the slug function), the daemon's ticket-file code (crates/bridle-daemon/src/ticket_file.rs, since ticket files are minted there), docs/design/cli.md, the tickets rule. Check callers of the slug function and tests that assert file names.

## Acceptance

Tests: derived slug from a title with a colon, without, and a long title; `--slug` accepted, and refused when uppercase, too long or with a bad character; an existing file name is untouched. `just check` with BRIDLE_* env unset. Model: Sonnet.
