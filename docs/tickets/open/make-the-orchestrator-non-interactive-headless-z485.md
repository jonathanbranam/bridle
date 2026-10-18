---
id: br-z485
title: Make the orchestrator non-interactive (headless)
kind: feature
created: 2026-10-03
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-r8kv, br-jttf, br-ae9f, br-gcvj, br-gb97, br-pdmd]
theme: agents-and-cli
parent: br-p7jf
---

## The ask

Now wanted, as the first step of epic br-p7jf (agents as their number grows), before the machine
admin role (gb97). The human, 2026-10-18 ~8:45 AM ET, to advisor (product-manager), verbatim:

> That is the right idea for the admin, but I don't want more interactive sessions on RC. Let's
> move the orch to background before pursuing the admin. Sequence that work.
>
> I think we would be ok to move the orch to background as a first step.

(RC: Claude Code's Remote Control, how the human reaches interactive sessions from the phone.)

Earlier, 2026-10-03 (via the advisor, a list headed "Ideas to record not build"):

> Make orch non interactive

## Proposal

- **Problem:** the orchestrator is an interactive `claude` session in a tmux pane, reached by RC.
  It hears messages only through its own `bridle agent wake` waiter, and every interactive session
  adds another RC session for the human to keep. The admin role (gb97) would add one more.
- **Approach:** design pass first (designer), the human reviews before any build. Run the
  orchestrator headless under the daemon, like managers and workers, so the daemon delivers its
  messages as turns. The design must answer: how the human talks to it without RC (gateway / web
  UI, mail, an advisor relay; research gcvj found RC cannot reach a headless agent); what happens to
  its supervisor, handover and restart path (orchestrator-supervision.md); its permissions and
  auto mode as a spawned agent; one per machine in `<host>-home` still; the cut-over and fallback.
- **Scope:** the orchestrator only. Other interactive roles (aide, PdM, advisors) are the epic's
  later design. Not the admin role (gb97 waits on this).

## Notes

- Today the orchestrator is an interactive `claude` session the human talks to
  (`bridle session orchestrator`). Goes with the split in
  [[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]]: if a triage role
  takes over the human's conversations, the orchestrator could run headless under the daemon.

## Design options

Focus: both. Interface (how the human reaches a headless orchestrator, what changes at the
keyboard) and internal architecture (principal, message delivery, restart and handover). Design only;
the human reviews before any build. Written for gb97, which waits on this.

Problem in a sentence: the orchestrator is an interactive `claude` in a tmux pane kept alive by a
supervisor and reached by its own waiter, and the human does not want it on Remote Control (RC)
any more; run it under the daemon like a manager.

### What exists today

- **The role already exists as a daemon role.** `orchestrator` is a built-in role "like the manager
  without `Bash(git *)`" (roles-and-config.md), with `permission_mode = "dontAsk"`, an explicit
  `allowed_tools`, `resume_on_restart`, a role prompt and the context governor (`[context]
  wind_down_at`, then a `Context handoff:` note and a renew; agents.md). The name `orchestrator`
  is reserved and only the human, the daemon and the orchestrator may spawn it (2vja). The
  lifecycle gate already treats an agent of role orchestrator as an orchestrator. Nobody runs it
  that way: the real one is the external session.
- **The external orchestrator** (`bridle session orchestrator`, orchestrator-supervision.md): pid
  file, tmux pane tag, a 10 s supervisor that relaunches it with backoff, wake conditions served to
  one long-poll waiter (`bridle agent wake`), context and uptime notes, a forced restart at a
  deadline, a handover note record (`bridle handover write`), `bridle up` at boot. Principal
  `external:orchestrator`, tokens in `~/.bridle/credentials.toml` `[orchestrator]`, and visiting
  `orchestrator@<machine>` on other machines' daemons (j86p). Its mail follows its waiter (dg5g).
- **Where the principal name matters:** `external:orchestrator` is matched in about 25 places in
  `bridle-daemon/src` (10 in `server.rs` for the endpoint rights: restart, hold, queue writes, ready,
  skip-settle, handover; the rest in orchestrator.rs, wake.rs, store.rs, outbox.rs, mentions.rs),
  plus the CLI and gateway message code.
- **The human already talks to the aide, not the orchestrator.** The orchestrator role says it "no
  longer talks with the human": aide (interactive, one per project) relays both ways, and the
  orchestrator reaches the human only through aide. So the loss from going headless is smaller than
  the ticket assumes: the pane is the rare direct line (supervision doc rule 1: "the pane is the
  human's").
- **RC cannot reach a headless agent** (research gcvj). Chat in the gateway/UI is medium work and not
  built; the gateway has `POST .../messages` (send to an agent) and reads, but no agent routes.
- The daemon delivers a headless agent's messages as turns, folded mid-turn, and resumes agents
  after a restart; a headless agent runs no waiter.

### Questions answered up front

1. **How the human talks to it without RC.** Through aide (and the advisor), by message, as now. In
   addition, anything that can send a message works: `bridle send orchestrator ...`, the UI's
   send box (gateway `POST messages`), mail. To watch it: `bridle agent logs orchestrator` today; a
   transcript view in the UI is gcvj's later slice. Direct chat in the UI is not needed for the cut-over
   if the human agrees they go through aide (question 1).
2. **Supervisor, handover, restart.** The daemon's agent host replaces them: autostart in the home
   project, resume after a daemon restart, and the context governor's wind-down note followed by a
   renew. The handover note stays: the wind-down note tells it to run `bridle handover write`, and
   the fresh session starts from `bridle orchestrator prime orchestrator` as its `start_prompt`.
3. **Permissions and auto mode.** As an agent it runs `dontAsk` with an explicit `allowed_tools`,
   not the classifier, so what it may run is now written down. Keep today's reality (j2vq): deny
   `bridle agent stop|rm` in its `disallowed_tools`. This also makes gb97 easier: narrowing the
   orchestrator is a role-config edit, not a trust-the-prompt rule.
4. **One per machine, in `<host>-home`.** The agent is spawned in the home project only; the name
   `orchestrator` allows one live agent (spawn refuses a duplicate name).
5. **Cut-over and fallback:** below.

### Options

**A. Run it as the existing `orchestrator` agent in the home project; make the daemon treat it as
the orchestrator.** What the user sees: `bridle status` lists `orchestrator` as an agent; `bridle
send external:orchestrator ...` and `bridle send orchestrator ...` both reach it; no pane, no waiter,
no RC. Inside: (1) one helper, `is_orchestrator(principal)`, true for `external:orchestrator`
and for an agent of role orchestrator, replacing the ~25 string matches; (2) messages and
forwarded wakes addressed to `external:orchestrator` on a home daemon with a running orchestrator
agent are delivered to that agent as turns (an alias, not a second inbox); with no agent running,
they queue for the waiter as today (this is the fallback path); (3) toward other projects' daemons
the agent uses the same visiting token as now (it runs with `BRIDLE_AS=orchestrator` for those; to
verify in the build that the CLI picks `[orchestrator]` credentials for a non-home project while it
holds an agent token for home). Cost: one helper and one routing rule; a mode switch; the old
supervisor code stays for the fallback until it is deleted. Falls short: every wake becomes a
turn (the same per-wake context cost pdmd measured; batching wakes is a follow-up, not here); the
human loses typing into the pane.

**B. A new "external agent host": keep the principal `external:orchestrator`, but run its claude
process under the daemon without an agent row.** Keeps every `external:orchestrator` check as it is,
but builds a second way to run `claude` beside the agent host (spawn, stdin delivery, resume,
renew, logs), which already does all of it. Fails KISS and modularity. Rejected.

**C. Stay interactive, but off RC.** Launch the pane without RC (local tmux only) and leave
everything else. A one-line launcher change and nothing else to build. It meets "no more RC
sessions" for the orchestrator only if RC is the only complaint, and leaves what the epic wants: the
waiter fragility (9aj2), the second supervisor, a tmux dependency at reboot, and the admin role
(gb97) still has to choose between another interactive session or the same move later. I could not
verify from the repo whether the orchestrator is registered on RC today. This is the honest
cheapest baseline; recommend it only as a stopgap if A slips.

**D. Do nothing.** Costs the human an RC session per interactive role, as the epic says.

### Measured against the principles

- **KISS/YAGNI:** A reuses the agent host, the governor and the role that exist. B builds a second
  host. C builds nothing but leaves the problem. A deliberately builds no chat UI, no wake batching
  and no new command.
- **One name per action:** A adds no commands: `bridle send`, `agent logs`, `handover write` and
  `orchestrator prime` stay. Reaching it by `external:orchestrator` or `orchestrator` is one
  alias, not two inboxes.
- **Modularity:** the principal check moves behind one helper (good); the routing rule couples the
  message router to "is an orchestrator agent running" (accepted, one place).
- **The user's side first:** the human keeps talking to aide, unchanged; the pane disappears; the
  UI shows an agent.

### Recommendation

**A.** Cut-over, per machine and reversible:

1. Add `is_orchestrator()` and the routing alias; ship it inert (nothing spawns the agent yet).
2. A home-daemon config switch `[orchestrator] mode = "session"` (default, today) or `"agent"`.
   In `agent` mode the home daemon autostarts the `orchestrator` agent with the prime as its start
   prompt and does not run the session supervisor. The launcher `bridle session orchestrator`
   refuses while an orchestrator agent runs, and the agent spawn refuses while a live session pid
   exists (the supervisor's existing guard against two orchestrators).
3. Trial on `dalek-home` for a mobile-only day (gcvj's bar); then `vesta`, `remus`.
4. Fallback at any time: set `mode = "session"`, restart, run `bridle session orchestrator`. The
   session path and its supervisor are kept for one release after all machines moved, then deleted
   (their own ticket).

Verification: unit tests for `is_orchestrator` and the routing rule (a message to
`external:orchestrator` with and without a running agent), a fake-claude test of renew with a
handover, the existing endpoint-rights tests passing for both principal forms, then the live trial
and the reboot test (mru4): the orchestrator is back after a reboot with no tmux and no human.

Trade-offs accepted: no pane to type into; every wake is a turn; two orchestrator code paths for one
release.

### Rejected (so the next reader doesn't re-propose them)

- A second external-process host beside the agent host (B).
- Building chat in the gateway/UI first: useful (gcvj) but not needed for this cut-over if the human
  goes through aide. Do it in the epic's later design if the human wants it.
- Renaming the principal to `agent:orchestrator` everywhere now: breaks visiting tokens, forwarding
  and every older daemon. The helper makes the rename optional later.

### Questions for the human

1. Do you ever type to the orchestrator directly (pane or RC), or only through aide? If only aide, no
   chat UI is needed first.
2. Is "every wake is a turn" acceptable for now, with batching a later ticket?
3. Keep `bridle agent stop|rm` denied to it (recommended, as today), or allow them now that it is an
   allow-list?
4. Is a mobile-only day on dalek the right bar before the other machines move?
5. How long do we keep the session path as a fallback: one release (recommended)?
