+++
id = "br-s4ve"
title = "vitest-bridle: a step can't skip a scenario at runtime"
kind = "feature"
state = "pending"
created_at = "2026-10-05T03:33:06.199Z"
updated_at = "2026-10-09T11:04:38.627967Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
ticket = "s4ve"
+++

docs/tickets/open/vitest-bridle-a-step-can-t-skip-a-scenario-at-runtime-s4ve.md

submitted by external:orchestrator@nuc

From meta-notes-ui mu-hxvt (2026-10-05). A scenario that depends on an environment step (meta-notes init building a .venv) needed to skip with a reason when that step fails. vitest-bridle had no way to do it, so the project patched its vendored tools/vitest-bridle: a step throwing an error with skipScenario:true calls ctx.skip(reason). The vendored copy now differs from bridle's (see br-m4cx, no signal when bridle's copy changes). Ask: a supported runtime skip (e.g. ctx.skip(reason) exposed to steps, or a SkipScenario error) in bridle's vitest-bridle, so projects don't fork it. Also: a scenario running a slow setup (~5s init) needed the spec timeout raised to 60s; a per-scenario timeout would help.

## Thread

### note · external:orchestrator@nuc · 2026-10-05T03:33:06.201Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T03:33:14.695Z
Triage (pm-1): accept; meta-notes-ui is a priority project and its vendored copy has already diverged. Ticket minted (uncommitted; needs committing on main). Stays pending: approve with `bridle task ready br-s4ve`, then I plan it (Sonnet). Related: br-m4cx.

### note · external:advisor/product-manager · 2026-10-09T11:04:38.627Z
watching the task
