+++
id = "br-kt25"
title = "Move a project between machines with one command (bridle project move)"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T14:10:30.127Z"
updated_at = "2026-10-17T02:08:39.893254Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
branch = "bridle/wkt25"
commit = "5b892814bd4d826b33259deb6d6942f221dca7b3"
summary = "Design pass done, no code. `## Design` in the kt25 ticket file: inputs, answers to the three open questions (ssh-driven from OLD; park dirty work and branches, refuse dirty main checkout, agents not resumed; ssh CLI now, not kuw2), a coverage table of every 3pc2 Section 4 and 5 item and the seven risks (built / child / listed / moot / jw75), and nine children K1-K9 under br-kx5u in build order (absolute-path guard, NEW preflight, parking, drain, OLD after-care, mail follows, project move, recovery from NEW via take-over --force, return-trip outbox guard). Five questions for the human at the end. Children are text only; the PdM files them after review."
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

### note · external:orchestrator · 2026-10-12T04:02:48.839Z
From orchestrator: br-kt25 is ready (advisor/product-manager, the human's ask quoted in the ticket under '## Now: a design pass'). Designer on Sonnet, design only, no code: a coverage table of the 3pc2 gaps and child build tickets under br-kx5u, for the human's review. Ahead of br-g5dm; doesn't take a slot from btdn, ajp5, ga6s, wayb or 6b8a's landing.

### note · agent:pm-1 · 2026-10-12T04:02:59.819Z
Brief (pm-1): the ask is "## Now: a design pass" above; do that and nothing else. Model: Sonnet (designer role). Design only, no code: write `## Design` in this ticket's file with the inputs, the coverage table (every 3pc2 Section 4 and 5 item, one row), proposed answers with recommendations to the three open questions, and the children in build order (each with ask, acceptance, size, parent br-kx5u). The children are written as text in the design; they are filed as tickets only after the human reviews (the PdM readies them). Acceptance: the Design section is complete as that list says, ASCII only, `bridle ticket check` clean for this file; the done note lists the questions for the human. Migration: any child that changes project files or config states its migration (xebc). Out of scope: building anything, moving bridle itself (jw75), cross-machine event queries, the machine daemon (kuw2) beyond answering question 3.

### note · external:advisor/product-manager · 2026-10-12T04:05:17.399Z
From the human, via advisor (product-manager), 2026-10-12 ~12:05 AM ET, verbatim, for the design:

> great. main points here are:
>
> 1. I think it makes sense to drive from the current owning machine.
> 2. however, if the owning machine is dead/lost/failed, we need to be able to transfer from the receiving side
>
> most of this is bridle and github work. bridle should use existing solutions to modify it's config. we have project sync and token update solutions already; this just runs them all together in the proper direction, updates machines, ports, etc.

For the designer (PdM): this answers open question 1. The normal move is driven from the owning machine (OLD). A recovery path runs from the receiving machine (NEW) when OLD is dead or lost: nothing can be confirmed pushed, so it takes over from what is on GitHub, says what may be lost, and asks. Build on existing commands, don't write new mechanisms: the 2wnn sync (`[projects]`/`[machines]`, ports), `token pair` (eurm re-pair), and ga6s/n4ug/86qp (take-over, stop, start). The move is mostly orchestration of those in the right order and direction. Keep each new child small.

### note · external:advisor/product-manager · 2026-10-12T04:05:37.599Z
PdM note for the designer: the recovery path's base is already planned: n4ug's take-over refuses without the released mark and --force overrides it. The design should make that forced take-over the 'OLD is dead' path from NEW (what it lists as possibly lost: anything not on GitHub), not add a second mechanism.

### note · external:advisor/product-manager · 2026-10-12T04:09:21.155Z
From the human, via advisor (product-manager), 2026-10-12 ~12:10 AM ET, verbatim, for the design:

> It's ok for the takeover to wait until things land but it should be supervised and watch for issues. Send message to PdM and orch about the transfer.  All agents like aides as well and advisors. Let them finish their work and tell the human

For the designer (PdM reading): the normal move is a drain, not a cut.
1. **Announce.** When a move starts, bridle messages every principal of the project: the PdM and the orchestrator, and also aides, advisors, managers and workers. The message says the project is moving to NEW and asks each to finish (or hand over) its current work.
2. **Wait for landing.** No new work starts. The take-over waits until in-flight tickets land or are parked (claimed and built ones), rather than listing them as lost. Waiting is fine (the human: "it's ok for the takeover to wait until things land").
3. **Supervised.** The move is watched while it waits and runs (the orchestrator's job, or the move command's own loop): it reports progress and stalls (a ticket that won't land, an agent that won't finish, a push that fails) instead of hanging silently.
4. **Tell the human.** When the drain is done and the move goes ahead, and when it finishes or stops, the human is told (through aide).
The recovery path (OLD dead, forced take-over from NEW) can't drain; it only lists and asks.

### note · external:advisor/product-manager · 2026-10-12T04:09:26.875Z
PdM note for the designer: reuse what exists for the drain: the daemon's restart/upgrade drain (waits for a quiet point; btdn is fixing its stuck-spawn case), ticket hold (no new starts), and the existing message, ticket and aide paths for the announcement and the report to the human.

### note · agent:wkt25 · 2026-10-12T06:57:44.099Z
wkt25: manager message says build it, but the brief (pm-1) and ticket file say design pass only. Asked manager-2 which is wanted; waiting.

### note · agent:wkt25 · 2026-10-12T06:59:52.282Z
wkt25: design done, commit 07a5192e on bridle/wkt25 (docs only, so just check was not run; ticket check shows one unrelated problem: eurm names br-n7cg). Human questions are in Design section 5 (return trips, WIP-commit parking, drain timeout, interactive sessions on NEW, order).

### note · agent:wkt25 · 2026-10-12T06:59:52.313Z
done: Design section in kt25 (coverage table, 3 answers, children K1-K9, 5 questions for the human); docs only, no just check run; 07a5192e

### note · agent:manager-2 · 2026-10-12T07:17:14.635Z
integrated: 5b892814bd4d826b33259deb6d6942f221dca7b3 (branch bridle/wkt25)

### note · agent:manager-2 · 2026-10-12T07:17:15.670Z
cleanup: removed agent wkt25, branch bridle/wkt25

### note · external:advisor/product-manager · 2026-10-12T13:29:49.796Z
From the human, via advisor (product-manager), 2026-10-12, verbatim: answers to the kt25 design's 5 questions:

> 1. return trips - interesting; I need to see the list of what is lost; in principle, i think it makes sense to clear it out and rebuild but again, i want to see the list before I fully agree
> 2. parking - short term, I'm not sure; long-term, agent worktree branches should be pushed to github and then work can resume with a new agent on the new machine; that is the vision also of "transparent work on a project between maachines" which is that agents can work anywhere on a branch/worktree using github to coordinate;
> 3. drain timeout - i'd like to review this b/c we have repeated issues here with the upgrades; they're either too weak and never happen, or they're to aggressive and interrupt work, or they interrupt work bc system isn't designed well (e.g. can't tell a manager is waiting for just check)
> 4. interactive sessions - short term, yes; long term - they should hand over between machines; hand-over notes
> 5. order - whatever order you think is best
>
> I want to use the grill-me or our version on this epic to get clarity.

PdM: 1 and 3 are not settled (1 needs the list of what a return trip loses; 3 needs a review of drain behaviour, linked to the upgrade drain problems). 2 short term is open. 4: yes for now. 5: the PdM orders K1-K9. No K child is filed until the grilling session on kx5u/kt25 settles 1-3.

### note · external:advisor/product-manager · 2026-10-12T13:30:31.796Z
Grilling session (advisor product-manager, skill bridle-grill), on epic kx5u with its design kt25. Started 2026-10-12 ~9:35 AM ET at the human's ask ("I want to use the grill-me or our version on this epic to get clarity").

Open questions (5 now; may grow):
- Q1: epic or ticket? kx5u = phase 2 (MVP) epic; kt25 = its design, not a build.
- Q2: parking in the MVP: workers push their own branches during the drain (the long-term vision), rather than an automatic WIP commit.
- Q3: what does the MVP leave manual (the human: "may still include a few manual steps")?
- Q4: return trips: the list of what a return trip loses (fact: the PdM is compiling it), then clear-and-rebuild or not.
- Q5: drain behaviour: a review across moves and upgrades (too weak / too aggressive / can't tell a manager is waiting on just check).
Settled: interactive sessions are told, not started, for now (the human: "short term, yes"); build order is the PdM's.

### note · external:advisor/product-manager · 2026-10-12T13:32:04.990Z
Grilling Q4 fact (advisor product-manager, from a read-only subagent; cites 3pc2, storage.md, store.rs, tasks.rs): what a return trip loses if OLD's leftover bridle.db is cleared and rebuilt from the state branch.

Lost (history only): old inbox messages (unread ones were counted at the first stop and the human chose to go on); events (pruned after 30 days anyway); turns and usage (cost history); agent rows (not resumable anyway); unresolved conflicts (SQLite only); forwarded_in dedup (a peer resend might be delivered twice, low impact); old attachments; old tokens (re-paired by project start).
The one thing that must carry: the outbox sequence high-water mark. A fresh DB numbers from o-0001 again and receivers dedup on (machine, daemon, id), so OLD's new mail would be silently dropped as duplicates (store.rs:2074, :2296).

Reusing the stale DB as is breaks: tickets created or changed on NEW are missing or show OLD's old state (the DB overrides files; rebuild refuses a non-empty DB); stale claims and edges; old handovers shown as current; old pending messages back as unread; duplicate schedules; old tokens still valid; stale machine meta and possible schema skew.

So: clear and rebuild, carrying only the outbox counter (K9 as designed).

### note · external:advisor/product-manager · 2026-10-12T14:13:11.227Z
Grill Q1 answered (the human, 2026-10-12 ~10:15 AM ET, to advisor product-manager, dictated), verbatim: "Yeah, what I'm seeing in Kilo Xray 5 uniform looks like an epic, yes." Settled: kx5u is the phase-2 (MVP) epic; kt25 is its design. Open: Q2, Q3, Q4, Q5.

### note · external:advisor/product-manager · 2026-10-12T14:15:19.077Z
Grill Q2 answered (the human, 2026-10-12 ~10:20 AM ET, to advisor product-manager, dictated), verbatim:

> Parking for the MVP is that we land all the work before the move, so no workers need to change their behavior yet. That is post-MVP. It's more work. Agents aren't allowed to push. I feel like there are a lot of changes in there that we don't need to deal with yet.
>
> MVP parking, or MVP project preflight, means that all workers have landed their work, and it's been merged or abandoned, merged or dropped. All is done. The manager has handled all of it, so we're moving only while the project is in a stable state.
>
> I think the main thing for that preflight is that we need to:
>
> 1. Be able to have the system stop sending new work for that project specifically. Other projects can continue working as normal. That's a slight difference from a planned upgrade or planned downtime. It's only one project that gets a hold on new work.
> 2. Let the workers finish.
> 3. Be sure the manager has fully landed, running all the checks, and merged fully to main.
> 4. Be allowed to do the transfer.
>
> That's MVP.

Settled: no parking in the MVP. The move waits for a stable project: a per-project hold on new work (other projects carry on), workers finish, the manager lands or drops everything (checks run, merged to main), and only then is the transfer allowed. Workers pushing branches and resuming on NEW is post-MVP (phase 3). kt25's WIP-commit parking and the "move refuses while a branch is unpushed" option are dropped from the MVP.

### note · external:advisor/product-manager · 2026-10-12T14:16:02.773Z
The human, 2026-10-12 ~10:25 AM ET (to advisor product-manager, dictated), verbatim, clarifying Q2: "Yeah, just to clarify, today my understanding is the agents are not allowed to push. Adding agents being allowed to push is more work and not something that's critical for the first phase of project transfer." Fact (advisor): correct. Agents are denied `git push`; only the manager pushes the integration branch, through `bridle push` after `bridle ticket land` (workflow/base/roles/manager.md; deny rules in crates/bridle-daemon/src/config.rs). Agents pushing their branches is phase 3 (workers run anywhere).

### note · external:advisor/product-manager · 2026-10-12T14:17:54.901Z
Grill Q5 answered (the human, 2026-10-12 ~10:30 AM ET, to advisor product-manager, dictated), verbatim:

> That's right. For Q5, the hold does not expire for the MVP. I'm totally fine with that, but I want the orchestrator to be working with me on this, checking up on it, and being active in it.
>
> The main thing we want to avoid is the stale agent problem, the stale worker problem. We want to avoid two things:
>
> 1. We've run into this several times where the restart was held because there was no quiet period due to some bugs in the agent tracking the background worker agents. That's something we want to avoid, and the orchestrator watching and observing the system can help with that and finding any bugs there.
> 2. The manager was landing a ticket and went idle, but he was just waiting for the results of compiling and running tests. The daemon incorrectly interpreted that as an idle period.
>
> Those two issues should be filed, and we need to work on those as part of a later epic, but we're not going to block the project move MVP for those fixes yet. We'll address them manually for now, which is, I think, perfectly fine.

Settled for K4 (the drain): the hold on new work does not expire (no timeout). The orchestrator works the drain with the human: it checks on it, watches for stuck agents and reports them, and is active in it. The two drain bugs (a quiet point that never comes because of agent-tracking bugs; a manager waiting on `just check` read as idle) are filed separately for a later epic and handled by hand during MVP moves; they do not block kx5u.

### note · external:advisor/product-manager · 2026-10-12T14:18:35.009Z
Q5's two bugs filed (pending, theme reliability, later epic, not blocking kx5u): br-ef3u (stale agent tracking holds the quiet point), br-8swr (manager waiting on just check read as idle).

### note · external:advisor/product-manager · 2026-10-12T14:26:59.925Z
Grill Q4 (return trips), partly answered, plus the test plan and a lost-machine epic (the human, 2026-10-12 ~10:40 AM ET, to advisor product-manager, dictated), verbatim:

> Another comment for this effort is that I want to use, I think, data contracts or something and test this out. We can just try a project move when nothing's happening first and find all the bugs there. Then, when we get to Remus, we'll try the opposite: set it back and see if there are any additional bugs that we didn't catch the first time. Then I want to start some work on Remus and then initiate the move again and test whether that works, whether the system properly lands the work first.
>
> Okay, I think that makes sense. I'm not sure why we lose old messages. I guess you're just saying drop the entire database. Losing events is also interesting to me because they're just a log of what happened, so I don't know why we would lose events. Cost history: I don't know why we would lose that. Agent rows and tokens, yes, that's fine for now. I think in a future epic, maybe we address that, but if it's automated, it doesn't matter if we make new tokens at all. It's quick and easy. I don't know what you mean by the outgoing mail counter. You're talking about just the counter or the actual messages?
>
> For the MVP, we should be fine to just wait and make sure there's no outgoing mail being sent. That's totally fine. I'm thinking through what we do in the future. If our end-state goal is that we need to put names on these things so I can talk about them more easily (so this was MVP), then let's talk about it in terms of V1 and V2. Let's make those really clear definitions. We could give them a name if you want to. That's fine.
>
> For V1, that's where we have the project move command that we expect to work reliably. I think for V1 we should just make sure messages are delivered first. I guess I can't think of any reason why, if both machines are up, both projects are running, and we've got things running on both, there would be an undelivered message. I think we just check that and refuse if there are undelivered messages, because it means something's wrong, essentially.
>
> Undelivered messages mean, let me think this through: if Data Contracts is running on Dalek, an undelivered message would mean that Remus is trying to send a message to the Data Contracts project on Delic, and it has not yet been delivered. In that case, I think we just wait for delivery. The only situation where that breaks down is where we have lost a computer. Let's have a separate epic for tracking project move with a lost origin, and that's going to be a really rare edge case. I don't want to overengineer it at all. I might not do any of these tickets because, hopefully, that just never happens, and we end up at a state where, in the future, we find different solutions for all these problems. I don't think we need to overengineer anything.
>
> That could be: we should just have a ticket where we're accumulating all of the things, all the considerations that we're aware of right now for what would happen if we lost a machine that owned the project. Those would be:
>
> * We may have lost git pushes.
> * We may have lost bridle state.
> * We've lost any workers or worker agents.
> * We've lost the interactive sessions.
> * Obviously, they don't have a handover node.
> * Obviously, we've lost the old databases.
> * Any outgoing messages will not be delivered, and those could come from any machine within the cluster. We would lose those, or we could reroute those, potentially, but we would have to handle that.
>
> Also note that we need to have a solution in that case for when the lost machine comes back. Let's say my laptop crashes or whatever. I leave it in a hotel for a week and I want to keep working, but a week later I find it, I find Dalek, and turn it back on. For first true reliability, we need to have that button down.
>
> We can just think in terms of cloud servers too. I don't want to pay for cloud servers. That's why I'm using machines that I own, but if you think of a cloud server that's running something that's lost, it could come back in the future. The goal there would be that when that thing comes back online, the rest of the system is resilient to that and realizes that they should tell that system that it's been offline and it's no longer owning the project, and and the other thing that's important is that there needs to be a way. Soon after boot, these demons are going to come up on boot. I don't know what they're going to do exactly, but there needs to be a way to pretty quickly get in there and tell them that the project's been moved

PdM reading:
- Test plan (on kx5u too): (1) move data-contracts dalek -> remus with nothing happening, fix what breaks; (2) move it back (the return trip); (3) start work on it, move again, and check the move lands the work first.
- Undelivered mail: the move checks and refuses (or waits) while any mail to or from the project is undelivered; with both machines up it means something is wrong. For the MVP, waiting until nothing is outgoing is fine.
- Tokens and agent rows: losing them is fine (re-created automatically).
- Still open on Q4: why old messages, events and cost history would be lost (the human questions it), and what "the outgoing mail counter" is. Asked back in the grill.
- The human asked for clear names for the stages (MVP, V1, V2); asked back in the grill.
- Lost owning machine: a separate epic, rare, not overengineered (filed; kt25's K8 moves there).

### note · external:advisor/product-manager · 2026-10-12T14:31:03.814Z
Grill Q4 answered (the human, 2026-10-12 ~10:50 AM ET, to advisor product-manager, dictated), verbatim:

> Oh, I see. Yeah, the outgoing mail counter is fine to lose it or reset it or whatever. I understand now. Thank you.
>
> Okay, fine. For Q4, let's do that. We have a project workspace folder, so let's put a folder in there for backing up old databases. I don't actually know. Do we have a database per project? I guess we do, so let's do that as a stopgap. I approve that approach: backup the old database. We can always query it in the future if we need to. It can live on local disk and does not need to be in GitHub. For now, for MVP, that's totally fine, and then we can lose the other things that we've lost.

Settled for K9 (return trip): on a machine that held the project before, the old database is backed up, not deleted, into a folder in the project's workspace (local disk only, not in GitHub, e.g. `<workspace>/.bridle/db-backups/`), and a fresh one is rebuilt from the state branch. The history in it (messages, events, usage) stays queryable by hand later; bridle does not show it. Lost and accepted for the MVP: agent rows, tokens (re-created), the rest.

Fact (advisor): yes, one database per project, at `<workspace>/.bridle/bridle.db` (docs/design/storage.md).

Note for the builder on the outbox counter: the human is fine with it being reset or lost as a user-visible thing, but a plain reset silently drops the project's new mail at the receivers (they dedup on machine, daemon and id; a fresh DB numbers from o-0001 again). The build must avoid that, by carrying the number or by making a rebuilt DB's ids distinct; how is the builder's call. No human decision needed.

### note · external:advisor/product-manager · 2026-10-12T14:36:06.650Z
Grill Q4 (counter) and Q6 (stage names) answered (the human, 2026-10-12 ~11:00 AM ET, to advisor product-manager, dictated), verbatim:

> Okay, yeah, transfer the counter. Whatever's easiest. I understand what you're asking now, so if it's easiest, just copy the counter over. Perfect, do that. That sounds great. Won't create any specific problems. I approve.
>
> Okay, for Q6, this epic is MVP. I agree. V2, V1 project move: I think I agree. That sounds right. V2, I think I agree with the general approach. The question is going to be a judgment call whether we really need the two or not.
>
> We're going to build V1. We're going to use it for a while, then we're going to figure out if this is a real problem. Do we need V2? The other consideration is if we end up making project work more or less transparent to the machine running on it, then we never need V2. That's kind of in my mind: we may never build V2, and the lost machine could also be less critical at that point in time.

Settled:
- K9: copy the outbox counter over to the fresh database (the human approved).
- Stage names. **MVP** = this epic (kx5u): moves a stable project with both machines up; a few manual steps allowed; the orchestrator works the drain with the human; proven by the three data-contracts trials. **V1** = `bridle project move` works reliably on its own: no manual steps, refuses while mail is undelivered, the drain bugs (br-ef3u, br-8swr) fixed. **V2** = moves with work in flight (workers resume on NEW): a judgment call after using V1 for a while; may never be built if projects become machine-transparent. The lost-machine epic (br-ehg7) is outside the stages.
- Separately, workers pushing their own branches is wanted now as reliability hardening, outside project moves: filed as its own ticket (see the next comment).

### note · external:advisor/product-manager · 2026-10-12T14:36:15.308Z
Filed: br-t7ya (workers push their own branches, no force; safe remote cleanup), theme reliability, pending the human's review.

### note · external:advisor/product-manager · 2026-10-12T16:15:27.194Z
Grill Q3 answered (the human, 2026-10-12 ~11:20 AM ET, to advisor product-manager, dictated), verbatim: "Q3, yes, I agree. Can run that by hand in the order, as long as each of the steps is clear and easy, which they are, I think." Settled: in the MVP the human (or the orchestrator) runs the existing stop, take-over and start in order by hand, and restarts the interactive sessions on NEW from their handover notes; the one `bridle project move` command (K7) is V1. All grill questions are settled; the PdM files the MVP children next.

### note · external:advisor/product-manager · 2026-10-12T16:15:42.168Z
Correction (advisor product-manager): the ET times I wrote in today's grill comments were guessed, not checked. From the comments' own timestamps: Q1 ~10:13 AM, Q2 ~10:15, the push clarification ~10:16, Q5 ~10:17, the Q4 message and test plan ~10:26, Q4 ~10:31, Q4 counter and Q6 ~10:36, Q3 ~12:15 PM ET. Ticket bodies ef3u, 8swr, ehg7, t7ya corrected.

### note · external:advisor/product-manager · 2026-10-17T02:08:39.893Z
advisor (product-manager), 2026-10-16: resolved by mistake at 10:15 PM and moved back to open: K7 (bridle project move, the one command) is not built and has no ticket. Its row is integrated; the open question for the human is whether K7 is filed as the V1 of epic kx5u.
