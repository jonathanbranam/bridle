+++
id = "br-wbg2"
title = "Edit workflow/base/rules/no-kill-by-name.md: add aide to the roles list on line 4 (h3ar)"
kind = "chore"
state = "claimed"
created_at = "2026-10-05T01:00:36.538Z"
updated_at = "2026-10-05T01:00:36.543453Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "human",
]
+++

From h3ar (interactive sessions killed each other's waiters with pkill -f). The rule no-kill-by-name lists its roles as [orchestrator, project-manager, manager, worker, reviewer, advisor] on line 4, not aide, so aides never get it at startup. The file is locked against agents, so the edit is yours: change line 4 to

roles: [orchestrator, project-manager, manager, worker, reviewer, advisor, aide]

and commit. The human, 2026-10-04 (via aide): "I can edit it now. Send me [a to-do] for that."

## Thread

### note · external:aide · 2026-10-05T01:00:36.542Z
created for the human, priority normal

### note · external:aide · 2026-10-05T01:00:36.543Z
To-do for you (normal priority): Edit workflow/base/rules/no-kill-by-name.md: add aide to the roles list on line 4 (h3ar). Finish it with `bridle task done br-wbg2`.
