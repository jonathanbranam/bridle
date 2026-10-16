+++
id = "br-kx5u"
title = "Machines: set up a new machine and transfer a project to it, with nothing left behind"
kind = "epic"
state = "pending"
created_at = "2026-10-12T00:47:51.647Z"
updated_at = "2026-10-16T10:00:10.324242Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "kx5u"
+++

docs/tickets/open/machines-set-up-a-new-machine-and-transfer-a-project-to-it-w-kx5u.md

## Thread

### note · external:advisor/product-manager · 2026-10-12T01:00:03.013Z
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

### note · external:advisor/product-manager · 2026-10-12T01:51:46.578Z
The human (~10:00 PM ET, to advisor product-manager), verbatim: "The new machine setup and project transfer will be common; otherwise the renaming of a machine is a rare thing. It could be treated as a new machine setup, essentially." So no rename command: renaming a machine = set it up as a new machine and transfer its projects. r9tk dropped.

### note · external:advisor/product-manager · 2026-10-12T14:27:30.060Z
Test plan (the human, 2026-10-12 ~10:40 AM ET, to advisor product-manager, dictated), verbatim: "I want to use, I think, data contracts or something and test this out. We can just try a project move when nothing's happening first and find all the bugs there. Then, when we get to Remus, we'll try the opposite: set it back and see if there are any additional bugs that we didn't catch the first time. Then I want to start some work on Remus and then initiate the move again and test whether that works, whether the system properly lands the work first." PdM: Done-when item 2 gains these three trials with data-contracts: (1) an idle move dalek -> remus; (2) the move back (return trip); (3) a move with work in flight, which lands the work first. Also: the lost-owning-machine case is now its own epic, br-ehg7 (rare, not on this path); kt25's K8 moves there. Grill Q2 and Q5 answers are on br-kt25.

### note · external:advisor/product-manager · 2026-10-12T16:15:42.254Z
Correction (advisor product-manager): the test plan comment above says ~10:40 AM ET; it was ~10:26 AM ET.

### note · external:advisor/product-manager · 2026-10-16T10:00:10.324Z
The human, 2026-10-15 ~10:20 PM ET (to advisor product-manager), verbatim: "We can test adding a project once or twice more as well. It isn't difficult" and "Next will also be transferring projects back and forth. I'm more interested in that being seamless since it may be more common than adding brand new projects". PdM: rank seamless transfer (kx5u) above project setup (dffz) in the next planning.
