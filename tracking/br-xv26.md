+++
id = "br-xv26"
title = "Short ticket file names: a few words, not the whole title"
kind = "feature"
state = "integrated"
created_at = "2026-11-18T02:25:58.344Z"
updated_at = "2026-11-18T03:01:55.118768Z"
created_by = "external:aide@remus"
watchers = [
    "external:aide@remus",
    "agent:xv26",
]
branch = "bridle/xv26"
commit = "8319b4beab08eb68cd47a343c34115f250990124"
summary = "New ticket files are named <slug>-<id>.md with a short slug: --slug on ticket new (validated: lowercase ASCII/digits/-, no edge -, max 24), else the title before the first ':' else first 3 words, capped at 24 on a word boundary. Single mint point is ticket::create (submit makes no file; from-task goes through new). Existing files untouched. Rule tickets text, cli.md, CHANGELOG updated. Tests added in ticket.rs; one old name expectation updated."
ticket = "xv26"
+++

docs/tickets/open/short-ticket-file-names-a-few-words-not-the-whole-title-xv26.md

submitted by external:aide@remus

## The ask

Ticket file names should be short: a couple of words plus the ID (`flume-clone-pshd.md`), not
the title cut at 60 characters. Today `bridle ticket new` builds the slug from the whole title,
so names like `flume-clone-set-up-a-whole-workspace-from-the-tasks-repo-flu-pshd.md` are the norm.

The human, 2026-11-17 ~9:30 PM ET (via the flume aide), verbatim: "these ticket names are wildly
too long; what is the guidance on them? flume-clone-set-up-a-whole-workspace-from-the-tasks-repo-flu ???
it should just be flume-clone-[id]"

## Design (proposed; bridle's PM to settle)

- Add `--slug <words>` to `bridle ticket new` (and `submit`, when the file is minted) for an
  explicit short name; validate it (lowercase ASCII, digits, `-`, max ~20-25 chars).
- Without `--slug`, cut the derived slug much shorter: the first 2-3 words of the title, or
  everything before the first `:` when the title has one (`"flume clone: set up ..."` gives
  `flume-clone`).
- The rule `tickets` tells filers to pick a short slug.
- Existing files keep their names (names are set once); no mass rename.

## Discussion (for reference, not authoritative)

Current flume examples: `flume-guides-guide-files-with-front-matter-flume-guide-liste-x6xn.md`,
`flume-prime-and-agent-text-xml-sections-like-openspec-instru-87h9.md`. The human's own brief for
flume (its task files) had already asked for a 40-50 char cap; here they want far shorter.

## Thread

### note · external:aide@remus · 2026-11-18T02:25:58.346Z
submitted by external:aide@remus

### note · external:orchestrator · 2026-11-18T02:26:35.439Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-18T02:26:35.558Z
orchestrator: readied. The human asked for this result directly (quote above, via the flume aide / aide@remus); pm-1's design is a straight implementation of that ask, so no further question to the human.

### note · external:aide · 2026-11-18T02:27:05.729Z
Relayed by aide@bridle for aide@flume (remus), which cannot comment (br-za3j). The human, verbatim: "don't rename existing tickets". This confirms the proposal: existing ticket files keep their names.

### note · agent:xv26 · 2026-11-18T03:01:26.341Z
built: d3c47840 (branch bridle/xv26)

### note · agent:xv26 · 2026-11-18T03:01:26.382Z
done: short ticket file names (--slug, derived 3-word/colon slug, cap 24); just check exit 0, 2022 tests passed; d3c47840

### note · agent:manager-2 · 2026-11-18T03:01:38.429Z
integrated: 8319b4beab08eb68cd47a343c34115f250990124 (branch bridle/xv26)

### note · agent:manager-2 · 2026-11-18T03:01:38.624Z
ticket file moved to docs/tickets/resolved/short-ticket-file-names-a-few-words-not-the-whole-title-xv26.md

### note · agent:manager-2 · 2026-11-18T03:01:55.118Z
cleanup: removed agent xv26, origin branch bridle/xv26, branch bridle/xv26
