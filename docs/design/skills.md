# Skills: rewrite as a small set

The ~10 OpenSpec skills per repo are replaced by a handful in the base layer,
with project addenda:

| Skill | For | Replaces |
|---|---|---|
| `bridle-driver` | orient, decompose, plan, spawn, wait, arbitrate | driver-guide, propose/new/continue/ff-change |
| `bridle-worker` | claim, read plan, implement, report on the task as it goes, handoff | apply-agent-guide, apply-change |
| `bridle-plan` | writing the task plan, spec edits with ids, impact declaration | propose-change, design.md conventions |
| `bridle-review` | reviewing a diff against plan + specs + impact | verify-change, acceptance-verifier |
| `bridle-triage` | intake → planned, dropping, dedup | ticket-conventions, maintenance |
| `bridle-conflict` | the [conflict protocol](docs/design/impact-and-conflicts.md) protocol from a worker's side | the same-spec pile-up rule |

Each skill is short because the procedure lives in bridle's commands. The
skill says when to run which command and what judgement applies. Rules and
guides are delivered by `bridle prime` and aren't repeated in skills.
