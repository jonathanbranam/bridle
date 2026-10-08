+++
id = "br-xg47"
title = "Put the NUC projects' human tokens in dalek's credentials ([human.nuc]) so the web UI can reach NUC documents (ui-9hq8)"
kind = "chore"
state = "integrated"
created_at = "2026-10-08T12:52:01.654Z"
updated_at = "2026-10-08T22:51:27.375103Z"
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

### note · external:aide · 2026-10-08T22:47:34.311Z
bridle-ui aide, 2026-10-08 ~7 PM ET: the human says "meta-notes-ui is a project on the nuc". Add it too: dalek's [human.nuc] needs meta-notes, notes, dotfiles-local AND meta-notes-ui (token in /srv/shared/work/meta-notes-ui-work/.bridle/tokens/human on the NUC). Also dalek's ~/.bridle/config.toml [projects] lacks meta-notes-ui = { machine = "nuc", port = 7405 } (the NUC's config has it; nuc:7405 answers from dalek). The configs have drifted the other way too: the NUC's [projects] lacks bridle-ui and track-web.

### note · external:orchestrator · 2026-10-08T22:48:11.552Z
From orchestrator: I added meta-notes-ui = { machine = "nuc", port = 7405 } to dalek's ~/.bridle/config.toml [projects]. Left for you: put all four NUC tokens in [human.nuc] (meta-notes, notes, dotfiles-local, meta-notes-ui). Then run 'env -u BRIDLE_AS -u BRIDLE_PROJECT bridle gateway restart' so the web UI picks them up. The NUC's [projects] missing bridle-ui and track-web only matters on the NUC.

### note · external:aide · 2026-10-08T22:51:13.958Z
Correction, bridle-ui aide ~7:05 PM ET: dalek's config.toml now has meta-notes-ui under [projects] (line 13; the file changed at 6:48 PM, after my read). Still missing: meta-notes-ui in [gateway] projects (line 33), and the [human.nuc] tokens.

### note · external:aide · 2026-10-08T22:51:18.119Z
Correction to my last note: line 33 is the mail section's projects list (not [gateway]); meta-notes-ui is missing there only if the human wants its questions by mail. For the web UI, only the [human.nuc] tokens are missing.

### note · human · 2026-10-08T22:51:27.375Z
done
