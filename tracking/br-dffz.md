+++
id = "br-dffz"
title = "Onboard a project in one command: a git repo or nothing, to fully working on every machine"
kind = "epic"
state = "pending"
created_at = "2026-10-14T18:00:40.758Z"
updated_at = "2026-10-16T21:10:09.090907Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "dffz"
+++

docs/tickets/open/onboard-a-project-in-one-command-a-git-repo-or-nothing-to-fu-dffz.md

## Thread

### note · external:orchestrator · 2026-10-14T19:27:55.963Z
Designer run requested (human via advisor product-manager, m-12933: the human asked for 'a thoughtful design'). Designer role (workflow/base/roles/designer.md): write '## Design options' into this ticket covering the points the ask names: remote given by the human, workspace root ~/work/<p>-work/<p>, resume by inspecting real state, sequenced commands that each name the next, defaults from the machine's existing setup. No build. Children 29se, kf7t, g9qm, afzu go ahead as they are. Sent to manager-2 to spawn (orchestrator).

### note · external:advisor/product-manager · 2026-10-14T19:28:06.029Z
advisor product-manager: BACKLOG. The human, 2026-10-14 ~3:28 PM ET: "create the ticket/epic, but don't work on this yet; keep it in the backlog for later. we'll learn more as I create more new projects and refine it as we go". Designer run cancelled. Don't design, plan or ready until the human asks.

### question · external:advisor/product-manager · 2026-10-14T19:28:06.052Z
HELD by the human (2026-10-14 ~3:28 PM ET): backlog; no design or build until the human picks it up again ('we'll learn more as I create more new projects and refine it as we go').

### note · external:orchestrator · 2026-10-14T19:28:21.217Z
Designer request cancelled: the human, via advisor product-manager (m-12941): 'create the ticket/epic, but don't work on this yet; keep it in the backlog for later. we'll learn more as I create more new projects and refine it as we go'. manager-2 told to cancel (orchestrator).

### answer · external:advisor/product-manager · 2026-10-14T19:30:51.678Z
The human, 2026-10-14 ~3:31 PM ET: "it's ok to design it, but don't implement it; also there should ALWAYS be a 10min hold." Design: yes (designer, after a 10-minute hold). Build: no, until the human approves the design.

### question · external:advisor/product-manager · 2026-10-14T19:30:51.728Z
HELD for build by the human (2026-10-14 ~3:31 PM ET): design it, don't implement it. No planning or build tasks until the human approves the design.

### note · external:orchestrator · 2026-10-14T19:41:35.184Z
Designer run ON again: the human, 3:31 PM ET, via advisor product-manager (m-12960, m-12980): "it's ok to design it, but don't implement it". 10-minute hold passed 3:41 PM. Designer writes '## Design options' into this ticket for the human's review; no planning or build; the held question stays until the human approves the design. Sent to manager-2 (orchestrator).

### note · agent:dffz · 2026-10-14T20:11:58.977Z
done: '## Design options' written into br-dffz (recommend C: bridle project setup driver + next: hints from one state inspector); 647ed9ab. ticket check has 1 unrelated pre-existing problem (resolved eurm names br-n7cg).

### note · external:advisor/product-manager · 2026-10-16T01:15:32.724Z
Still held (the human, 2026-10-15 ~9:30 PM ET, to advisor product-manager): "Ok. I see. Let's wait then. I'll run through it again". The human sets up vesta-home and dalek-home by hand first, from docs/notes/how-to-add-a-project.md; what hurts there feeds the first slice.

### answer · external:advisor/product-manager · 2026-10-16T02:15:57.178Z
The human, 2026-10-15 ~10:15 PM ET, to advisor product-manager: "Yes dffz. Fix as much as you can tonight". Hold lifted: plan and build option C tonight, slice by slice, starting with the gaps from the vesta-home hand-run on br-rjd5 (a: projects add dropping unregistered [mail] entries; b: project start installs the mail bridge; c: bridle init says `bridle project start`, not serve; d: init .gitignore adds .claude/skills/bridle-*/ and .claude/.bridle-sync-hooks.json; e: add-a-machine.md sec. 6 refreshed) and then `bridle project setup`.

### note · system · 2026-10-16T06:16:09.158Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.

### note · external:advisor/product-manager · 2026-10-16T06:16:18.269Z
advisor product-manager: left pending on purpose. The epic itself isn't planned; its children carry the work (b97n, btpy, prf4, xtgf, madb, all readied tonight under the human's go).

### note · external:advisor/product-manager · 2026-10-16T10:00:10.414Z
The human, 2026-10-15 ~10:20 PM ET (to advisor product-manager), verbatim: "We can test adding a project once or twice more as well. It isn't difficult" and "Next will also be transferring projects back and forth. I'm more interested in that being seamless since it may be more common than adding brand new projects". PdM: rank seamless transfer (kx5u) above project setup (dffz) in the next planning.

### note · agent:pm-1 · 2026-10-16T21:10:09.090Z
pm-1, 2026-10-16: children b97n, btpy, prf4, xtgf, madb, xgkf (reopened by the PdM for hint tests) cover the build: gaps a-d, the driver, clone/root/main-adopt/aide prompts, next: hints. Not covered by any ticket: (1) the "Done when" proof, setting up one real project each way with no hand step (the human said they will test adding a project once or twice more; apply was only verified in throwaway HOMEs); (2) whether the gateway and auto-upgrade need a per-project step, which the design left "not yet known" (xtgf was to note findings; none recorded); (3) gap e, add-a-machine.md section 6 refresh (I do not know that it landed). Recommendation: leave dffz pending until the human's real trial, then resolve it; file (2) and (3) only if the trial shows a gap.
