+++
id = "br-xg47"
title = "Put the NUC projects' human tokens in dalek's credentials ([human.nuc]) so the web UI can reach NUC documents (ui-9hq8)"
kind = "chore"
state = "claimed"
created_at = "2026-10-08T12:52:01.654Z"
updated_at = "2026-10-08T12:52:01.658326Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "human",
]
priority_at = "2026-10-08T12:52:01.656917Z"
+++

Needed for remote documents in bridle-ui (ui-9hq8) and the gateway work (br-7172, br-ty37). br-8b98 (the [human.<machine>] fallback) is in; the NUC daemons answer from dalek (http://nuc:7402/v1/health ok).

Steps (you, not an agent: these are your own tokens):
1. On the NUC, print each project's human token from its workspace, e.g.
   cat /srv/shared/work/meta-notes-work/.bridle/tokens/human
   and the same file in the notes and dotfiles-local workspaces.
2. On dalek, add to ~/.bridle/credentials.toml:
   [human.nuc]
   meta-notes = "<token>"
   notes = "<token>"
   dotfiles-local = "<token>"
3. Check: bridle --project meta-notes task list   (works without BRIDLE_TOKEN)
Then `bridle task done <this id>`.

## Thread

### note · external:orchestrator · 2026-10-08T12:52:01.656Z
created for the human, priority normal

### note · external:orchestrator · 2026-10-08T12:52:01.658Z
To-do for you (normal priority): Put the NUC projects' human tokens in dalek's credentials ([human.nuc]) so the web UI can reach NUC documents (ui-9hq8). Finish it with `bridle task done br-xg47`.
