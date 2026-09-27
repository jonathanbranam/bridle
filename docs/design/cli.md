# The CLI surface (first cut)

```
bridle init | sync | prime | doctor              project setup, render, session start, health
bridle project add|list|remove
bridle task new|show|edit|list|drop|reopen
bridle dep add|rm            bridle ready [--all] [--role]
bridle claim|release|handoff bridle plan <id>     bridle accept <id> (human only)
bridle send|ask|answer|inbox [--human] [--inject]
bridle wait <id> [--until <state>] [--or-message] [--timeout]
bridle spawn <role> <task>   bridle agents        bridle review
bridle impact set|show|check bridle conflict list|resolve
bridle spec check|id|export|coverage|import
bridle rules show|explain|diff|propose
bridle goals list|propose       bridle arch propose
bridle trace up|down|suspect|confirm|orphans|coverage
bridle explore new|conclude|adopt|abandon
bridle rebuild
```

Every command takes `--json`. Agents always use it, and humans get tables.
