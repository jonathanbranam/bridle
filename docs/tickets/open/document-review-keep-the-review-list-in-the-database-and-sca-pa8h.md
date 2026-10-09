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
see: [x8jt, ehv6, ppa6, yj38, wjhp]
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

## Found in the first end-to-end test (2026-10-08, docs/context/name-ideas.md)

The test worked: the gateway put the document under review on comment save (19:52 EDT), the daemon sent c1-c4 at 20:07, and agent `doc-name-ideas-e11e82` answered all four in commit 82990f87 at 20:08. The human: "Sure add them." Two bugs:

4. **The send came 15 minutes after the last comment, not 7.** The last comment (c4) was saved at 19:52 and the send was at 20:07, with `quiet_minutes` 7 (the default). The cause isn't known. Candidates: the 30 s tick, the `max_agents` cap, the machine-load hold, or the quiet timer counting from something other than the last change. Note that `[review]` is read only at daemon start, so the human's `quiet_minutes = 1` in config.toml didn't apply. That's surprising next to `[[focus]]`, which applies at once.
5. **The agent dated its replies 20:15 EDT, after it had committed them at 20:08.** It's guessing the time instead of reading the clock. The role should get the time from `date`, or bridle should stamp it.

## Web tools for the document reviewer (the human, 2026-10-08)

> Also - does the agent have web search? I think it needs it. Will be helpful often and the agent isn't writing code or anything either.

It doesn't. `document-reviewer` uses `Role::worker_default()` (config.rs), which has no WebSearch or WebFetch; only the researcher role adds them. On c1 the agent reported docs.fsfe.org answering 403 on every path, presumably by fetching without the tools (rule `report-task-failures`).

6. **Give `document-reviewer` WebSearch and WebFetch,** as the researcher role has. This is small and independent of the rest, so it can ship first.

## Assigning an agent to a document (the human, 2026-10-08)

> Also - where is the ticket for assigning a specific agent to review the document? Liked tagging an agent to the document for review or assigning them? That should be part of any change to the txt file.

No ticket has this as its ask. The nearest are x8jt's "Which agent answers" section, the human's 2026-10-02 thoughts on a per-document advisor-like agent, and yj38 (responder agents for each kind of incoming item). Today the agent is always `doc-<file stem>`, made by `agent_name(path)` and started on first send. The wjhp bug: that name can exceed 40 characters.

7. **When the review list moves into the database (ask 1), give each entry an assigned agent.** The human chooses it, by tagging or assigning an existing agent (an advisor, say, or a named agent) to a document, from the UI or the CLI. The default stays the per-document reviewer. Design this together with ask 1, not after it.
