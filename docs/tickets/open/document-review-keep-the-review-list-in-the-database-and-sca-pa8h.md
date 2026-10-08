---
id: pa8h
title: "Document review: keep the review list in the database, and scan for unresolved comments so none are missed"
kind: feature
opened: 2026-10-08
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [x8jt, ehv6, ppa6]
tasks: [br-pa8h]
---

## The ask

The human, verbatim (2026-10-08 ~8 PM ET), while testing document review on docs/context/name-ideas.md:

> this using a plain text file on the filesystem is janky; why did we choose that? review-documents.txt? Why not store this in a table or something; also, I htink we need a periodic scan of documents that contain unresolved comments; that might be a longer / later thing to do , but would be helpful IMO to be sure things aren't missed; i'm not sure if we'd have to parse every document though; or be able to grep? I guess we can grep for [!comment] at least; but then would we have to parse the file? or could we write a grep that would always work?

## Context
- **Where the list lives:** `.bridle/review-documents.txt` in the main checkout (daemon.md, "Document review"). The file is untracked: it shows as `??` in `git status`.
- **Why a text file:** no reason is recorded. It came in with x8jt slice 2 (br-aj9d), under x8jt's "experiment first, then codify". As far as the aide can tell, it was never revisited.
- **Comments were missed today:** the gateway's automatic add on comment save failed (br-ppa6, the gateway started inside Claude Code), so the document was never on the list and its comments sat pending, unnoticed. A periodic scan would have caught them.

## The ask
1. **Keep the review list in the daemon's database,** not a loose text file.
2. **Scan periodically** for documents with unresolved comments, whether or not they're on the list, so nothing is missed. Either put them under review or tell the human.

## Notes for the design (the aide's, not decided)
- `grep -rl '\[!comment\]'` finds the candidate files cheaply.
- Whether a thread is pending depends on its newest entry, and on its marks and resolved line. That needs the existing parser (`pending_threads` in doc_watch.rs).
- So the cheap approach: grep for candidate files, then parse only those.
- A pure grep that's always exact would need the thread's state on a single line. Ticket ehv6 (explicit status on comment threads) is related.

## Leaving review (the human, 2026-10-08)

> also - how does a file every become un-watched? We need some kind of TTL for that otherwise they'll stay watched forever

Today the only way out is `bridle review remove` by hand. `[review] idle_hours` (default 4) stops the document's *agent*, but the document stays on the list and is re-read every 30 s indefinitely; a renamed or deleted file is never cleaned up either. 3. Give a watched document a TTL: e.g. it leaves review after N days with no pending thread and no new comment (the periodic scan in 2 puts it back if a new comment appears).
