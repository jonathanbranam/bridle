+++
id = "br-kt25"
title = "Move a project between machines with one command (bridle project move)"
kind = "feature"
state = "open"
created_at = "2026-10-09T14:10:30.127Z"
updated_at = "2026-10-12T04:02:27.819566Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "kt25"
+++

docs/tickets/open/move-a-project-between-machines-with-one-command-bridle-proj-kt25.md

## Thread

### note · external:advisor/product-manager · 2026-10-10T14:36:29.695Z
advisor/product-manager (PdM): sequencing, the human 2026-10-10 ~10:50 AM ET: the epic 'One orchestrator per machine' (ticket 3mz4) comes before project transfer; takeover can start from either machine and is handled by both orchestrators (tickets 3mz4, evy6, rjd5).

### note · external:advisor/product-manager · 2026-10-12T01:00:05.124Z
PdM: the human's decisions on what a move does with the project's database (2026-10-11, to advisor product-manager), verbatim:

> 1. it's ok to lose tokens; we have a command that re-creates them; so, we can lose them, move the project; create them; bridle should do this as part of the take-over somewhere
> 2. undelievered messages - the command to stop/transfer or whatever should check for them and count them and offer to stop or continue
> 3. scheduled messages need to be enumerated; a simple command should transfer them to the new machine; i assume this is only scheduled messages to roles in that project
> 4. agent turns and handovers are lost; this is ok
> 5. event history is interesting; can we query it between machines? Or communicate? Seems like an ok loss
> 6. usage history wouldn't be lost, but just not transfered I guess
>
> I htink the main thing here is, when moving a project
>
> 1. check and list what will be lost and explain it before committing
> 2. move anything that can be moved (scheduled messages would be good here)
> 3. re-create things that can't be moved - tokens

So the database is NOT copied (supersedes the PdM's copy recommendation). The move:
1. Before committing: lists what stays behind and why (counts: undelivered outbox mail, unread inbox messages, agents and sessions, events, usage), and asks to continue or stop. Undelivered mail is counted and the human chooses stop (let it drain) or continue.
2. Moves what can move: scheduled messages for that project's principals (enumerated, recreated on the new daemon). Tickets, task rows, edges, claims, open questions, queue and handovers already move on the state branch (rebuild); handovers are NOT lost (correcting point 4).
3. Recreates what can't: tokens, as part of take-over (re-run pair: eurm), and config everywhere (2wnn).
Events and usage stay in the old machine's bridle.db, which the move leaves in place (not deleted), so they can still be read there; querying across machines is out of scope.
