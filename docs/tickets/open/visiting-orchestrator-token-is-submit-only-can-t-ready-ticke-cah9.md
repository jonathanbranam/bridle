---
id: br-cah9
title: "Visiting orchestrator token is submit-only: can't ready tickets or comment on same-machine daemons"
kind: bug
created: 2026-10-17
created_by: external:orchestrator@dalek
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Filed by external:orchestrator@vesta (its `ticket new --project bridle` wrote the file into the
vesta-home clone instead of bridle's; committed here by orchestrator@dalek).

## What happened

On vesta, after the br-gr3v token pairing (`bridle token pair --machines dalek,vesta,remus`, v0.7.5
everywhere, 2026-10-16 ~20:55 UTC), the orchestrator is `external:orchestrator@vesta`, a visitor
homed on vesta-home, on the other local daemons (meta-notes, notes, dotfiles-local, meta-notes-ui).
On meta-notes, 2026-10-17 ~02:05 UTC:

- `bridle ticket comment mn-edk2 ...` -> `error: forbidden: a visitor may only comment on its own submissions`
  (mn-edk2 was filed by this same orchestrator a few minutes earlier, with `bridle ticket new --project meta-notes`)
- `bridle ticket ready mn-edk2` -> `error: forbidden: a visitor may only submit tasks and comment on its own submissions`

Sending messages, filing tickets and forwarding wakes to vesta-home all work.

Second bug: `bridle ticket new --project <other>` wrote the ticket file into the caller's cwd clone
(vesta-home), not the target project's clone, so bridle main had no file for br-cah9.

Third: on dalek, `bridle ticket new --from-task br-cah9` as `external:orchestrator@dalek` (also a
visitor on bridle since the pair) wrote this file into the clone, then the daemon refused with the
same `forbidden` error, leaving an untracked file. A refused mint should write nothing.

## Impact

The machine's orchestrator lost its orchestrator role on every same-machine project except its
home: it can't ready tickets the human approves, comment on tickets (rule talk-on-the-task) or,
presumably, edit the queue. Workaround: ask each project's manager to do it, with the human's
quote in the ticket body.

## Expected

A visiting `orchestrator@<machine>` keeps the orchestrator's rights on a daemon it is paired with,
as the orchestrator role doc implies ("a visiting orchestrator@<machine> token"). The forwarding
only moves where its wakes and mail arrive. Also, "its own submissions" should cover a ticket the
visitor filed itself. `ticket new --project X` writes into X's clone (or refuses).

## Check

As `external:orchestrator@vesta` on meta-notes: `bridle ticket ready` and `bridle ticket comment`
on a ticket succeed. `bridle ticket new --project bridle` from another clone puts the file in bridle's.
