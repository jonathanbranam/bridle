+++
id = "br-5d2a"
title = "Manager role prompt and rules use bridle land"
kind = "chore"
state = "integrated"
created_at = "2026-09-29T08:10:45.501Z"
updated_at = "2026-09-29T09:11:18.816485Z"
branch = "bridle/land-prompt"
commit = "ecc875055766784ad1890965328db0e699159153"
summary = """Updated workflow/base/roles/manager.md to use 'bridle land <task-id>' for landing completed work instead of manual merge steps. Added [integration] check = "just check" to .bridle/config.toml. Updated config test to verify 'bridle land' appears in manager prompt. All tests pass."""
+++

After the bridle land task merges: update workflow/base/roles/manager.md (and bridle's own .bridle rules if they describe manual merging) so the manager lands accepted work with `bridle land <task>` instead of hand-merging, and reports a refused landing to the worker/PM. Add [integration] check = "just check" to bridle's own .bridle/config.toml. Keep the change small. Acceptance: just check passes; bridle cost check (if it covers role prompt size) still passes. Model: Haiku. Out of scope: code.

## Thread

### note · agent:manager-2 · 2026-09-29T09:11:18.816Z
integrated: ecc875055766784ad1890965328db0e699159153 (branch bridle/land-prompt)
