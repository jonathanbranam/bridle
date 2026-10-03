+++
id = "br-c8c0"
title = "The first two of the six skills: bridle-manager and bridle-worker"
kind = "feature"
state = "integrated"
created_at = "2026-09-28T16:12:24.611Z"
updated_at = "2026-09-30T05:50:28.536780Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
commit = "7cda9d2"
+++

Next P2 task after br-ab66 (idle-after-renew fix). Per docs/design/skills.md and build-order.md ("the six skills" P2 deliverable) and docs/design/workflow-layers.md (skills/<name>/ sources under a layer, rendered by `bridle sync` -- just landed, P2-3/br-6694 -- into .claude/skills/bridle-*/).

Of the six skills docs/design/skills.md lists, only two map to what is actually built today:
- bridle-manager: orient, decompose, plan, spawn, wait, arbitrate
- bridle-worker: claim, read plan, implement, report on the task as it goes, handoff

The other four (bridle-plan, bridle-review, bridle-triage, bridle-conflict) reference specs with ids, an impact registry and the conflict protocol -- none of that exists yet (P3/P4, still design-only per build-order.md). Writing their skill content now would describe commands that do not exist. Leave them for when P3/P4 land; do not write placeholder content for them in this task.

Brief: add workflow/base/skills/bridle-manager/SKILL.md and workflow/base/skills/bridle-worker/SKILL.md (short -- docs/design/skills.md: "Each skill is short because the procedure lives in bridle's commands. The skill says when to run which command and what judgement applies. Rules and guides are delivered by bridle prime and are not repeated in skills."). Base content on what is actually built and current: the manager/worker sections of docs/design/agent-host/roles-and-lifecycle.md and docs/design/agent-host/operating-model.md, the current `.bridle/roles/manager.md` and `.bridle/roles/worker.md` prompts (bridle's own dogfood role prompts, for a concrete example of the procedure), and the task-queue commands j479/br-8638 just added (bridle task plan, bridle ready, bridle queue, claim). Verify `bridle sync` (br-6694) actually renders these into .claude/skills/bridle-manager/ and .claude/skills/bridle-worker/ end to end -- if the SKILL.md format/frontmatter bridle sync expects differs from what you write, fix whichever side is wrong to match Claude Code's own skill format (SKILL.md name/description frontmatter).

Acceptance: just check passes; bridle sync in a fixture/test project renders both skills into .claude/skills/ correctly; a human or worker reading the rendered skill can tell, without other docs open, which bridle commands to run and when.

Out of scope: bridle-plan/review/triage/conflict (see above), and any change to bridle sync's rendering mechanism itself unless the format mismatch above requires a small fix.

Model: Sonnet.

## Thread

### note · agent:pm-1 · 2026-09-30T05:50:28.536Z
integrated: 7cda9d2
