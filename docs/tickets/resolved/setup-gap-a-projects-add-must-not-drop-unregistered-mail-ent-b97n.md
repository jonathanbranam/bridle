---
id: br-b97n
title: "Setup gap (a): projects add must not drop unregistered [mail] entries when it syncs"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
parent: br-dffz
resolved: 2026-10-16T10:02:58Z
---

## The ask

Part of epic br-dffz (the human, 2026-10-15 ~10:15 PM ET: "Fix as much as you can tonight"; gaps a-e from br-rjd5 first). Seen in the hand-run of vesta-home setup (meta-notes aide on vesta, m-13894): `bridle projects add`'s sync rewrote `[mail] projects` from `[projects]` on all three machines and dropped dalek-home, which was not registered yet.

Wanted: sync never removes a `[mail] projects` entry just because the project is not (yet) in `[projects]`; it adds new projects (afzu) and leaves other entries alone, and says what it left. Also gap (e): docs/context/add-a-machine.md section 6: step 2 says projects sync does not spread [mail] until afzu (it does now) and the step numbering skips 6.
