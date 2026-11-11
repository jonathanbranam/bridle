+++
id = "br-k3ux"
title = "Typescript pack: naming rule and check-command backstop for case-colliding paths (slice 2 of 4fpk)"
kind = "feature"
state = "claimed"
created_at = "2026-11-09T14:59:55.882Z"
updated_at = "2026-11-11T20:10:38.247518Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:k3ux",
]
summary = "Added 'bridle case-check' (scans git ls-files incl. directories, exit 1 naming each pair; shares collides()/stem() with case-guard in case_guard.rs via tracked_collisions) and typescript pack rule typescript.file-naming (PascalCase.tsx, kebab-case.ts, backstop = add case-check to commands.check). Tests: case_check_test.rs (timelog.ts + TimeLog.tsx fails, clean repo passes), unit test, plus CLASSES/SCOPES entries. Docs: cli.md, CHANGELOG. Caveat: full just check has one failure, bridle-daemon restart_land_test a_land_past_the_cap... (untouched code; passes alone, fails in full runs at 30s timeout); 1998/1999 pass."
ticket = "k3ux"
+++

docs/tickets/open/typescript-pack-naming-rule-and-check-command-backstop-for-c-k3ux.md

## Thread

### note · external:aide · 2026-11-11T17:16:59.274Z
readied by external:aide

### note · external:aide · 2026-11-11T17:16:59.574Z
aide: readied. Slice 2 of br-4fpk, which the human asked for directly on 2026-11-09 (to aide, verbatim): "i'd like it to be filed and worked on to keep my at-work projects moving."

### note · agent:k3ux · 2026-11-11T20:10:38.247Z
k3ux built on bridle/k3ux, commit 5b4c4b36 (main already merged). just check is NOT green: 1998/1999 pass, 1 fails: bridle-daemon restart_land_test a_land_past_the_cap_gets_one_incident_and_its_check_is_stopped (30s 'check was stopped' timeout; code I did not touch; passes alone, fails in 3 full runs, so likely pre-existing or load related). Not marked built; tell me whether to mark it built anyway or investigate.
