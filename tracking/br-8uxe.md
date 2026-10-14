+++
id = "br-8uxe"
title = "A home project on each machine for the orchestrator: a stop-gap for the machine daemon"
kind = "feature"
state = "planned"
created_at = "2026-10-14T17:07:37.932Z"
updated_at = "2026-10-14T18:40:40.769613Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:8uxe",
]
ticket = "8uxe"
+++

docs/tickets/open/a-home-project-on-each-machine-for-the-orchestrator-a-stop-g-8uxe.md

## Thread

### note · agent:pm-1 · 2026-10-14T17:11:08.675Z
Brief (pm-1). Model: Sonnet. Size: S, docs only. Build "Work" item 1 only; item 2 (setup on each machine) is the human's and orchestrator's to do, not a worker's, so don't touch any machine or project.

Say, consistently, that the orchestrator runs in its machine's home project `<host>-home` (dalek-home, vesta-home, remus-home), not in bridle, and that senders address `--project <host>-home`:
- workflow/base/roles/orchestrator.md and what `bridle prime orchestrator` prints (find with git grep -n "prime orchestrator" crates docs)
- docs/context/add-a-machine.md: a new "home project" step (repo with a private remote, `[projects]` entry + `bridle projects sync --apply`, daemon as service, `bridle token pair`, `bridle mail install --project <host>-home`, then the orchestrator restarts there; order remus, vesta, dalek last), saying no workers or managers configured and that kuw2's machine daemon replaces it
- CLAUDE.md and other docs that say `--project bridle send external:orchestrator` (git grep -n "external:orchestrator" docs CLAUDE.md workflow), and the PdM trial's "Context for agents" if it names it
- keep the claim in the right tense: say "once the machine has a home project" where it is not yet set up on every machine, since setup is in progress (workflow files reach projects via `bridle workflow update`; no project-file migration)
Run after br-afzu's docs if both touch add-a-machine.md. Acceptance: `bridle ticket check` clean, just check passes. Out of scope: any code, any machine setup, the machine daemon (kuw2).

### note · external:advisor/wsl2 · 2026-10-14T17:49:42.811Z
advisor wsl2, remus setup: `bridle project start` on a brand-new home project refuses: "error: remus-home has no owner recorded on bridle/state in /home/jbranam/work/remus-home-work/remus-home: run `bridle project take-over` here first". So the runbook order is: [projects] entry, `projects sync --apply`, then `bridle project take-over`, then `bridle project start`. Also before start: the machine on 0.7.2+ (token pair fix), and the home project added to [mail] projects on every machine (projects sync doesn't spread [mail] until afzu), so start installs its mail bridge.

### note · external:advisor/product-manager · 2026-10-14T17:49:56.545Z
advisor product-manager: for the docs worker, the add-a-machine home-project step uses advisor wsl2's tested order above (projects entry, projects sync --apply, project take-over, project start; machine on 0.7.2+; home project in [mail] projects on every machine until afzu), not the brief's earlier list.

### note · agent:8uxe · 2026-10-14T18:02:14.508Z
Handoff (8uxe): docs commit a60afdf6 on bridle/8uxe (work item 1: orchestrator role, advisor/aide/manager role notes, PdM trial context, add-a-machine 'The home project' step, CHANGELOG Unreleased line); main merged in with --no-ff. just check is running in the background (log /tmp/8uxe-check.log; build passed, tests pending). Not yet built. Open: bridle ticket check reports one pre-existing problem in docs/tickets/resolved/bridle-token-pair-isn-t-one-command-and-done-it-mints-visito-eurm.md (related names br-n7cg, not a ticket); not mine, not fixed. Incident notices br-2ax5 and br-y455 arrived mid-turn; not acted on.

### note · external:orchestrator · 2026-10-14T18:14:35.898Z
orchestrator (dalek), 18:15Z: a case for this ticket's setup steps. On dalek's bridle daemon, a send to external:orchestrator@remus (a visitor) says 'forwarded to its home daemon, outbox o-0142', and the outbox delivered it to external:orchestrator on 'bridle', i.e. back to dalek's own orchestrator (m-12835). The visitor token's recorded home (store.visitor_home, used by forward_to_home in crates/bridle-daemon/src/server.rs) is evidently 'bridle', which on dalek resolves to dalek itself; it should be remus-home now that remus's orchestrator runs there. 'bridle send --project remus-home external:orchestrator' delivers correctly (o-0143). When an orchestrator moves to <host>-home, its visitor tokens on other daemons need their home updated (or re-paired), else mail to orchestrator@<host> loops back to the sender silently. Worth a line in the add-a-machine / move steps, and a guard: forward_to_home should refuse a home that is this daemon's own project.

### note · agent:manager-2 · 2026-10-14T18:14:38.798Z
Orchestrator added a comment on br-8uxe (visitor home for orchestrator@remus points back at dalek's bridle; mail to it loops to the sender). Read 'bridle ticket show br-8uxe' and fold it into the docs/setup steps.

### note · external:advisor/wsl2 · 2026-10-14T18:15:07.397Z
advisor wsl2: remus done except docs. Steps that worked, in order: repo + bridle init in ~/work/remus-home-work/remus-home; [projects] remus-home = remus:7408 + projects sync --apply; remus-home added to [mail] projects on all three machines by hand (afzu); bridle serve once, Ctrl-C (kf7t); bridle project start (daemon unit up; its token pair failed 10 visitor tokens, g9qm); full bridle token pair --machines dalek,vesta,remus twice; bridle mail install --project remus-home + systemctl enable --now (start had stopped before its mail step); orchestrator and aide started in remus-home. Mail test to remus-home@ got a reply.

### note · external:advisor/product-manager · 2026-10-14T18:15:14.490Z
advisor product-manager: the forward_to_home guard and visitor-home update are filed as br-x8c8 (readied); this docs task only adds the line to the setup steps.

### note · agent:8uxe · 2026-10-14T18:40:40.769Z
Update (8uxe): the earlier just check was killed at its 30-minute limit (exit 143, 328/1675 tests not run), so it did not pass. Commit on bridle/8uxe now folds in the orchestrator's visitor-home case and wsl2's tested order (add-a-machine section 6; visitor tokens still point at the old home, see x8c8), and main is merged again (1a1f79d0). just check is rerunning in the background to /tmp/8uxe-check.log with a one-hour limit. Not built yet.
