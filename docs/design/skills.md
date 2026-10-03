# Skills: rewrite as a small set

> **Status (checked 2026-10-03):** Built, not wired in: `bridle-manager` and `bridle-worker` (sources in `workflow/base/skills/manager|worker/`, rendered to `.claude/skills/bridle-*/` by `bridle sync` run by hand). The rendered copies are gitignored, so a worker's worktree has none, and the manager's `allowed_tools` (bridle's `.bridle/config.toml`, `dontAsk`) don't include `Skill`; no role prompt mentions them · Planned: `bridle-plan`, `bridle-review`, `bridle-triage`, `bridle-conflict`

The ~10 OpenSpec skills per repo are replaced by a handful in the base layer,
with project addenda:

| Skill | For | Replaces |
|---|---|---|
| `bridle-manager` | orient, decompose, plan, spawn, wait, arbitrate | driver-guide, propose/new/continue/ff-change |
| `bridle-worker` | claim, read plan, implement, report on the task as it goes, handoff | apply-agent-guide, apply-change |
| `bridle-plan` | writing the task plan, spec edits with ids, impact declaration | propose-change, design.md conventions |
| `bridle-review` | reviewing a diff against plan + specs + impact | verify-change, acceptance-verifier |
| `bridle-triage` | intake → planned, dropping, dedup | ticket-conventions, maintenance |
| `bridle-conflict` | the [conflict protocol](docs/design/impact-and-conflicts.md) protocol from a worker's side | the same-spec pile-up rule |

Each skill is short because the procedure lives in bridle's commands. The
skill says when to run which command and what judgement applies. Rules reach
spawned agents in their system prompt (and guides through `bridle prime`), so they
aren't repeated in skills.
