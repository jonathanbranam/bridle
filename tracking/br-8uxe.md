+++
id = "br-8uxe"
title = "A home project on each machine for the orchestrator: a stop-gap for the machine daemon"
kind = "feature"
state = "planned"
created_at = "2026-10-14T17:07:37.932Z"
updated_at = "2026-10-14T17:11:09.045833Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
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
