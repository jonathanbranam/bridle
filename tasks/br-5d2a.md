+++
id = "br-5d2a"
title = "Manager role prompt and rules use bridle land"
kind = "chore"
state = "planned"
created_at = "2026-09-29T08:10:45.501Z"
updated_at = "2026-09-29T08:10:47.874488Z"
+++

After the bridle land task merges: update workflow/base/roles/manager.md (and bridle's own .bridle rules if they describe manual merging) so the manager lands accepted work with `bridle land <task>` instead of hand-merging, and reports a refused landing to the worker/PM. Add [integration] check = "just check" to bridle's own .bridle/config.toml. Keep the change small. Acceptance: just check passes; bridle cost check (if it covers role prompt size) still passes. Model: Haiku. Out of scope: code.
