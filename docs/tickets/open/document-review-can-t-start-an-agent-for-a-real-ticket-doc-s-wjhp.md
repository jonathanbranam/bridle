---
id: wjhp
title: "Document review can't start an agent for a real ticket: doc-<stem> exceeds the 40-char agent name"
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [x8jt, jrm2]
tasks: [br-wjhp]
---

## The ask

The human, verbatim (2026-10-04, via advisor doc-review), after commenting on 3haz in the
document view:

> I also added comments to that document. Can you initiate the command to have it get feedback
> provided now?

What happened (advisor, 2026-10-04): `bridle review add docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md`
succeeded. Then `bridle review now <same path>` failed with `bad_request: the document's agent could not be
started; see the daemon log`. The daemon log (`/Volumes/Data/work/bridle/.bridle/daemon.log`, 14:59:36Z):

```
WARN bridle_daemon::doc_watch: document agent not started; will retry
  path=docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md
  error=bad request: invalid agent name "doc-daemons-deliver-mail-to-each-other-across-machines-store-and-3haz"
```

## Cause

- `doc_watch::agent_name` (`crates/bridle-daemon/src/doc_watch.rs:57`) names a document's agent
  `doc-<whole file stem>`.
- Agent names must match `[a-z0-9][a-z0-9-]{0,39}`, at most 40 characters
  (`crates/bridle-daemon/src/worktree.rs:22`).
- Ticket stems are a slug of up to about 60 characters plus `-<id>`, so for nearly every
  ticket `doc-<stem>` is too long. Document review can't start an agent for real tickets. The
  tests use short stems (`doc-my-ticket-x8jt`), so they miss it.
- The watcher also retries on every pass, logging a WARN each time.

## Fix (advisor's suggestion)

- When the stem ends in a ticket ID (`-<id>` in the ticket alphabet), name the agent
  `doc-<id>`. That is short, stable and readable.
- For any other file, cut the slug so the name fits in 40 characters. Add a short hash of the
  path so two long names don't collide.
- Add a test with a real-length ticket stem.
- The design doc for document review (see x8jt) should say how the name is formed.
