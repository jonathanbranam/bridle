---
id: br-4mbd
title: "Reserve xxxx: never mint it as a ticket id, and the link-check hook ignores br-xxxx"
kind: fix
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-17 ~4:05 PM ET, to advisor (tj7q), after the link-check Stop hook flagged the
placeholder `br-xxxx` three times in one session, verbatim:

> Also are you getting hooks for that br-xxxx? Instead use br-oooo and also file a ticket that we
> should exclude xxxx as a valid ticket id, then update the book to ignore that ticket id

("book" is "hook".)

---

## Proposal

### Why

Agents write `br-xxxx` as a placeholder ticket id in examples ("br-xxxx: design unblocked"). The
`bridle hook link-check` Stop hook flags every `<prefix>-<4 id chars>` word as a ticket that needs a
link, whether or not it exists (`crates/bridle/src/link_check.rs`, `unlinked_ids`: "A task ID of
the project's `prefix` always counts"), and `x` is in the id alphabet, so `xxxx` looks like a real
id. Each flag costs the human a turn of noise. `br-oooo` is the workaround for now: `o` is not in
the alphabet (`abcdefghjkmnpqrstuvwxyz23456789`), so it is never matched.

### What changes

- `xxxx` is reserved: id minting never produces it (the ticket and task id generators:
  `store.rs` `random_id_chars`, the `bridle ticket new` path; one shared "reserved ids" list so
  they agree), and `bridle ticket check` reports a ticket file or row with that id.
- The link-check hook ignores `xxxx`, prefixed (`br-xxxx`) or bare, in any project.
- A unit test in `link_check.rs` for each: `br-xxxx` is not flagged; minting skips `xxxx`.

### What does not change

The id alphabet, the 4-character length, and how the hook treats every other id.

### Risk and impact

None in practice: no ticket or task has the id `xxxx` today (check with `bridle ticket check`
before landing). One in 31^4 mints is redrawn.

### Out of scope

A general placeholder syntax, or making the hook check that a prefixed id exists.

### Open questions

None.

---
