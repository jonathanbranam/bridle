---
id: zta7
title: "Settle period: enforce it at worker spawn and task done, default 10m"
kind: bug
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

From the human, via advisor (notes project, message m-0103, 2026-10-06), verbatim:

"Yes I know it's 5 but I figured it wasn't working here at all which is wasn't. Yes increase the default to 10 for all projects. And verify there is a config setting for this. Yes, ask orch to investigate and identify how this should work and be enforced across machines and projects. Only something identified as a critical fix can bypass the wait."

## What happened (meta-notes, 2026-10-06, UTC)

- 14:48:13 the orchestrator filed mn-cys2 and ran `bridle task ready`; settling until 14:53.
- 14:48 the orchestrator told manager-1 "ready ... it can go next". The manager (which had just seen "settling until 10:53 AM, holding") ran `bridle spawn worker --name add-under-tasks` on it at once.
- 14:50:19 the worker reported done; 14:55:42 the manager merged and ran `bridle task done`: the task went `open -> integrated` without ever being planned or claimed. No skip-settle note.
- The human's two corrections arrived during the window (relayed by the advisor), and the worker's first commit missed the second because it was already done.

## Cause (orchestrator's read of the code at 28d25f8c)

The settle check lives only in `is_ready` / the queue's startable flag / `claim` (`crates/bridle-daemon/src/tasks.rs` `settle_until`, coordination.md "Settling"). Two paths go around it:

1. `bridle spawn worker` (manager) takes a free-text prompt and checks no task state, so a worker can start on an `open`, unclaimed, settling task.
2. `bridle task done` accepts a task straight from `open`, never planned or claimed.

Also: the clock restarts only on the human principal's thread entries. Relayed "From the human, via advisor" comments are authored by `external:advisor`, so they don't restart it.

Config: `[tasks] settle` exists per project (`config.rs` `tasks_settle`, `DEFAULT_SETTLE` = 5m). meta-notes, meta-notes-ui and notes don't set it.

## The ask

- `DEFAULT_SETTLE` = 10 minutes (the human's go); update coordination.md and the config docs.
- Enforce at every start path, not only claim: refuse a worker spawn for (or `task done` / integrate of) a task that hasn't gone through plan and claim, or that is still settling. The design question for the build: tie a worker spawn to a claimed task id (`--task`), so the daemon can check it.
- Skip-settle only for a fix identified as critical (the human). Today `skip-settle` is allowed for the human, orchestrator and PM for "the human asked" or "urgent downtime fix"; narrow its wording and role text to "critical fix", reason required.
- Decide whether a relayed human comment (`human via <agent>`, rule human-via-agent) restarts the clock. Recommendation: yes, when the note starts "From the human, via" (cheap; the relay convention already marks it).
- Across machines and projects: the default lives in the binary, so every daemon gets it on upgrade; projects override with `[tasks] settle`. Nothing per machine.

## Verify

Daemon tests: spawn or done on a settling / unclaimed task is refused with "settling until"; after settle + plan + claim it works; a skip-settle note lets it through; default reads 10m.

## The human's input (via advisor, m-0105, 2026-10-06), verbatim

- "Show me the ticket and write a clear design. I think claims should fail"
- "If this needs to be worked on the bridal site, I can work with over there on it. But this should be mechanical, deterministic, and not a rule that an agent can ignore. And yeah, so if I provide input on a ticket, that should reset the clock. I'm not sure, you know, other comments on a ticket might not reset the clock, so not sure how to handle that exactly, but yeah, if I provide input on the task, I guess it's the task, right? It should reset the clock so that I have 10 minutes from the last time I had made a comment about scope. Or design or whatever."

## Design (advisor's draft, shown to the human; supersedes "The ask" above where they differ)

Principle: enforced by the daemon, not by role text. The claim is the one gate into work, and it fails while a task settles.

1. **Claim fails while settling** ("settling until <time>"). Already true; it stays the single gate.
2. **No work without a claim.** `bridle spawn worker --task <id>` claims the task for the worker and fails while it is settling or unplanned. A spawn with no task (research) is allowed but can't finish a task.
3. **No finish without a claim.** `task done` / integrate is refused unless the task was claimed; no `open -> done`.
4. **Clock:** claimable at `max(created, the human's latest input on the task) + settle`. Human input is the human's own comment, question/answer or edit, and a relayed one. The relay is mechanical, not a text prefix: e.g. `bridle task comment --from-human` (allowed for advisor, aide, orchestrator), recorded as human input; the daemon restarts the clock on it. Other agents' comments don't restart it. Input on a ticket reaches the clock by being relayed onto its task the same way. (This replaces the text-prefix recommendation above.)
5. **Default 10m** in the binary (every daemon gets it on upgrade); `[tasks] settle` per project overrides; nothing per machine.
6. **Skip only for a critical fix:** the human any time; the orchestrator or PM only with a reason naming the critical fix (main red, service down, data at risk); recorded in the thread and sent to the human's inbox. Drop "the human asked".
7. **Visible:** `task show` and `queue` show "settling until <time>".
8. **Tests:** claim, `spawn --task` and done refused while settling or unclaimed; fine after settle + plan + claim; a critical skip passes; relayed human input restarts the clock; default 10m.

## Open questions (not answered yet)

- May `settle = 0` still turn it off for a project? Advisor recommends keep.
- Does human input on an already-claimed task pause the worker, or just reach it? Advisor recommends just reach it.

The human offered to work this through with the bridle side directly if needed.
