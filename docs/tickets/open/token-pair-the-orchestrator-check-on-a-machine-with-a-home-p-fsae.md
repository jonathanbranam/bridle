---
id: br-fsae
title: "token pair: the orchestrator check on a machine with a home passes --home without --machine, so every same-machine orchestrator check is 'refused'"
kind: bug
created: 2026-10-17
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-16 ~8:30 PM ET, running `bridle token pair --machines dalek,vesta,remus` to set up dalek-home (via aide): "More were accepted and different ones (I think) were rejected. Idk. Whatever is happening that sounds like utter nonsense. Why would I need to run it more than once??"

Output seen:
```
check vesta orchestrator -> vesta/dotfiles-local: refused: For more information, try '--help'.
check vesta orchestrator -> vesta/meta-notes: refused: For more information, try '--help'.
```

Cause (aide, from the code, reproduced on vesta): `check_role` (crates/bridle/src/token_pair.rs ~698) adds `--home <home>` whenever the orchestrator has a home (br-8usa), but adds `--machine` only for a visitor token (`!e.same`). `token pair-check` declares `--home` as requiring `--machine`, so for a same-machine entry clap exits with "the following required arguments were not provided: --machine" and the check reports refused. On vesta: `bridle --project meta-notes token pair-check orchestrator --home vesta-home` fails that way; without `--home` it exits 0. The tokens themselves are fine: dalek-home answers the orchestrator and aide locally and from vesta and remus.

So since br-gr3v (homes), every machine with a `<host>-home` shows its own projects' orchestrator checks as refused on every pair, and the guide's "run it twice" (docs/context/add-a-machine.md section 6 step 4) cannot clear them.

Fix: pass `--home` only with `--machine` in `check_role` (a same-machine token has no recorded home to compare). Test: a same-machine orchestrator entry with a home checks accepted. Also: a failed helper should report its stderr's first line, not the last ("For more information..."). And review whether step 4's "twice" is still needed; the human asks why it would be.
