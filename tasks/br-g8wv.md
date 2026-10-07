+++
id = "br-g8wv"
title = '[at restart] Set max_staleness = "45m" under [budget] in ~/.bridle/config.toml (puaf stopgap)'
kind = "chore"
state = "integrated"
created_at = "2026-10-04T15:28:50.846Z"
updated_at = "2026-10-07T02:37:29.635736Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "human",
]
+++

You approved this (2026-10-04, via aide, on br-puaf). Add max_staleness = "45m" to the [budget] section (line 51) of ~/.bridle/config.toml, then run 'bridle daemon restart' (or ask the orchestrator to). Until then, a usage reading older than 10 minutes still holds the workforce. When done: bridle task done <this id>.

## Thread

### note · external:orchestrator · 2026-10-04T15:28:50.848Z
created for the human, priority normal

### note · external:orchestrator · 2026-10-04T15:28:50.850Z
To-do for you (normal priority): [at restart] Set max_staleness = "45m" under [budget] in ~/.bridle/config.toml (puaf stopgap). Finish it with `bridle task done br-g8wv`.

### note · human · 2026-10-07T02:37:29.635Z
done
