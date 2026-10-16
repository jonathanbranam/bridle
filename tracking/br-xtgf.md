+++
id = "br-xtgf"
title = "bridle project setup: the driver (steps with check and apply, --check, idempotent) over clone, init, register, state, service, tokens, mail"
kind = "feature"
state = "claimed"
created_at = "2026-10-16T02:24:25.726Z"
updated_at = "2026-10-16T04:55:13.817759Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:xtgf",
]
ticket = "xtgf"
+++

docs/tickets/open/bridle-project-setup-the-driver-steps-with-check-and-apply-c-xtgf.md

## Thread

### note · agent:pm-1 · 2026-10-16T02:24:35.314Z
Brief (pm-1). Model: Sonnet. Size: L: split into stop points; stop at the first that gets long and tell the manager (follow-ups need the PdM to ready them). Read br-dffz's design sections, docs/context/adding-a-project.md, and the existing commands: `bridle init`, `projects add` (29se), `project start`, `token pair`, `mail install`. New module crates/bridle/src/project_setup.rs plus the clap wiring in main/cli (follow how project_start.rs is wired); steps are a table of (name, check, apply) calling the existing functions.
Stop point 1: the framework + `--check` + the register/state/service/tokens/mail steps for a project that already has a clone (run inside it, or `setup <name>`). Tests per step check (fake state), the checklist output, exit codes, idempotence (second run does nothing).
Stop point 2: clone from a URL / `--remote`, the root prompt with the `<p>-work/<p>` default and `--root`, the main/adopt question with `--main/--adopt/-y`, the aide token [Y/n], no-terminal exit 2.
Stop point 3 (only if small): the "next: <command>" line the step commands print, computed from the same inspector (br-dffz option C).
Docs: new section in docs/design/cli.md, docs/context/adding-a-project.md becomes "run bridle project setup", CHANGELOG. Verify with `--check` against this repo's own state and a throwaway workspace (never touch an existing project's main or dev; rule existing-projects). Acceptance: just check. blocked-by br-btpy (same project_start.rs area, mail install in start). Also run after br-b97n and br-prf4 if they are in flight (same sync/init code). Out of scope: gateway/auto-upgrade checks beyond reporting what a new project needs (note findings on the ticket), running setup for a project owned by another machine.

### note · external:advisor/product-manager · 2026-10-16T02:24:58.391Z
Readied by advisor product-manager under the human's go on dffz (2026-10-15 ~10:15 PM ET): "Yes dffz. Fix as much as you can tonight".
