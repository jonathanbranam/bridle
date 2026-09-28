# Role notes

The orchestrator's running notes on how work is split between the human, the
orchestrator and bridle's roles: what the human and the orchestrator did by
hand, which admin tasks could move into bridle, and where a role didn't fit.
The goal is to learn over time how to assign responsibilities and which new
roles are needed. The orchestrator keeps this up to date (every session, and
at each handover); tickets come out of it once a change is clear.

## The human's direction (2026-09-28)

The human, verbatim:

> I am concerned we shouldn't have too many responsibilites on the orchestrator. An
> important thing I need is as an interface from the bridle workers to me; like the
> "voice of bridle" - but also we need a more mechanical orchestrator within bridle, that
> lives on the same local machine, that can take important admninistrative actions;
> let's consider re-aligning some of the responsibilities.

On whether the orchestrator may stop or remove agents: "I am personally fine
with that".

So the working split to design towards:

- **The voice of bridle** (the orchestrator today): the interface between the
  workforce and the human. It presents questions with a recommendation,
  answers within the human's standing decisions, relays direction, and
  summarises. It can live wherever the human can reach it (hj4g).
- **An in-bridle admin role** (name to be decided): mechanical, on the same
  machine as the daemon. It does the administrative actions listed below,
  without needing the human's session to be awake.

## Admin tasks done outside bridle's roles

Who did each one today, and where it could go. "Voice" and "admin" refer to the
split above.

| Task | Done by today | Could move to |
|---|---|---|
| Verify each merge: `just check` twice on `main`, off load | orchestrator | admin (mechanical; CI covers part) |
| Push `main` after a merge | development manager or orchestrator | development manager only |
| Cut SemVer releases on verified `main` | orchestrator | admin, with the human told |
| Rebuild the binary (`cargo install --path crates/bridle`) | orchestrator | admin (`bridle rebuild`-like step) |
| Restart the daemon | human only | admin, if a supervisor outside the daemon exists |
| Resume `lost` workers after a restart, and tell each one | orchestrator | the daemon itself, or admin |
| Renew contexts by hand (before htp6b) | orchestrator | done: automatic since htp6b |
| Watch usage and budget windows | orchestrator's watcher | done in part: the budget governor |
| Stop or remove agents (`bridle stop`, `bridle rm --delete-branch`) | human (the orchestrator's auto mode refuses) | development manager (allowed since v4nk) or admin |
| Delete merged branches | orchestrator, in bulk | development manager, at merge (already the rule) |
| Notice everything is idle and find out why | orchestrator | admin, or the product manager |
| Nudge a manager that asked in a `note` and went idle | orchestrator | fix the manager's prompt, or admin |
| File tickets | orchestrator | stays with the voice (it holds the human's words) |
| Create the machine-wide `~/.bridle/config.toml` | human, from the orchestrator's text | admin, with the human's sign-off |
| Hand over between orchestrator sessions | orchestrator + human (one command) | stays with the voice; state lives in bridle (d4mz) |
| Plan around the laptop sleeping | open | prvy, hj4g |

## Role issues seen

- **Agents can't message the orchestrator** (a7h3). They message `human`,
  and the orchestrator reads that. It works, but the voice and the human share one
  inbox.
- **The orchestrator can't mark the human's questions answered.** It answers
  by messaging the asking agent; the question stays unread for the human.
- **Managers ask in a `note`, not a `question`, then idle.** The watcher
  only wakes on questions, so this stalls until someone reads the notes.
- **The product manager runs out of right-sized work** and idles waiting on
  a direction only the human can give (m-0716, after P1). The voice should
  bring "what next" to the human before the queue drains, not after.
- **Permissions for the orchestrator** (j2vq): the fix put `Bash(bridle *)`
  in the repo-wide settings, which grants it to every session in the repo.
  Parked. If the orchestrator needs stop/rm, grant it in
  `scripts/claude-orchestrator` only; but with the development manager
  allowed to do it (v4nk), the voice may not need it at all.
- **The orchestrator sleeps with the laptop** (hj4g, prvy), so the voice goes
  silent exactly when the human switches to the phone.
- **The watcher is a shell script outside bridle** (`scripts/orchestrator-watch.sh`).
  What it watches for (questions, `main` moving, crashes, all idle, usage) is
  admin work that bridle could do itself and push to the voice.

## Log

Newest first. One line per item: what happened, who did it, what it says about roles.

- 2026-09-28: the product manager proposed a separate `bridle-workflow` repo,
  and the orchestrator seconded it; the human rejected it as a hassle (r2uq).
  The voice should check a design's heavier choices against KISS before
  recommending them, not pass them through.
- 2026-09-28: the human asked for the voice/admin split and for this file.
  The orchestrator took the human's answers to three questions (P2 next,
  j2vq parked, no alias) and relayed them to the product manager.
- 2026-09-28: the human rebuilt, restarted the daemon and created
  `~/.bridle/config.toml` (the budget schedule) from the orchestrator's text:
  three admin steps only the human could take.
