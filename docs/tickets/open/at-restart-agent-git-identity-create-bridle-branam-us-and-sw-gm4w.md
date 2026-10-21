---
id: br-gm4w
title: "[at restart] Agent git identity: create bridle@branam.us and switch each clone to its per-machine PAT (4a4v)"
kind: chore
created: 2026-10-21
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

br-4a4v part 1 landed (6e944a54): agents now commit as 'bridle <agent> (<machine>) <bridle@branam.us>'. Two steps are yours:

1. Create bridle@branam.us in Google (mailbox or alias). Add it to your GitHub account's emails if agent commits should link to the account; otherwise they show unlinked.
2. Per machine (dalek, vesta, remus, NUC): in each clone, 'git remote -v'. If it shows git@github.com: (your SSH key), switch to the PAT with 'git remote set-url origin https://github.com/jonathanbranam/<repo>.git'; the first push asks for the PAT once and the keychain keeps it.

Run 'bridle ticket done <id>' when both are done.
