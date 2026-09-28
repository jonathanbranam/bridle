+++
id = "br-1e0c"
title = "Move roles and role prompts into workflow/base"
kind = "feature"
state = "dropped"
created_at = "2026-09-28T16:41:34.238Z"
updated_at = "2026-09-28T18:07:01.309477Z"
+++

Part of onboarding data-contracts (docs/context/onboarding-data-contracts.md, go-ahead
from the orchestrator/human 2026-09-28). Today role prompts live per-repo at
.bridle/roles/{advisor,manager,orchestrator,product-manager,worker}.md and are referenced
directly by path from .bridle/config.toml's system_prompt keys. A second project
(data-contracts) needs the same roles without copying these files.

Brief: move the role prompt sources into workflow/base/ (L1, docs/design/workflow-layers.md)
so a project gets them from the layers instead of a local copy, the same way
workflow/base/skills/ and workflow/base/rules/ already work. Concretely:
- Pick the right home under workflow/base/ for role prompts -- workflow-layers.md's layer
  shape lists `agents/<role>.md` as a layer content type; check whether that's meant for
  these driver-facing role prompts or for something else (e.g. Claude Code subagent
  defs under .claude/agents/) before assuming the name. If `agents/<role>.md` is the
  wrong slot, document why and pick a clear one (e.g. `roles/<role>.md`).
- Move all 5 files from .bridle/roles/ to their new home in workflow/base/, preserving
  content.
- Update .bridle/config.toml's system_prompt = ".bridle/roles/<role>.md" references to
  point at the new workflow/base/ location (all 4 roles that set system_prompt: worker,
  product-manager, manager; check advisor.md's role too -- it may be referenced elsewhere,
  e.g. a daemon config or CLAUDE.md).
- Search the repo for other references to .bridle/roles/ (docs, other config, CLAUDE.md)
  and update them.
- If bridle sync (crates/bridle-daemon/src/sync.rs, crates/bridle/src/commands.rs sync())
  already renders role/agent files from resolved layers, make sure this move doesn't
  break it; if it doesn't yet cover this content type, note that as a follow-up rather
  than expanding scope here.

Acceptance: just check passes; bridle's own daemon still starts and resolves system
prompts correctly from the new location (existing tests should catch a broken path,
but do a manual `bridle status`/role prompt check too if easy).

Out of scope: writing a data-contracts-side config or rules -- that lands directly in
data-contracts once this is done. Building workflow/packs/python/ (separate task, br pending).

Model: Sonnet (touches config wiring and needs to check sync's assumptions).

## Thread

### note · agent:pm-1 · 2026-09-28T18:07:01.309Z
dropped: Merged to main already (d9a770a), per orchestrator report; clearing done-but-stale planned record, same gap as br-4221/br-789a.
