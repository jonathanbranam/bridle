+++
id = "br-k3ux"
title = "Typescript pack: naming rule and check-command backstop for case-colliding paths (slice 2 of 4fpk)"
kind = "feature"
state = "integrated"
created_at = "2026-11-09T14:59:55.882Z"
updated_at = "2026-11-11T20:35:37.603588Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:k3ux",
]
branch = "bridle/k3ux"
commit = "f6141820d7fc182350f99869b117bd8681d77e34"
summary = "Added 'bridle case-check' (scans git ls-files incl. directories, exit 1 naming each pair; shares collides()/stem() with case-guard via tracked_collisions in case_guard.rs) and typescript pack rule typescript.file-naming (PascalCase.tsx, kebab-case.ts; backstop: add case-check to commands.check). Tests: case_check_test.rs (timelog.ts + TimeLog.tsx fails, clean passes), unit test, CLASSES/SCOPES entries. Docs: cli.md, CHANGELOG. just check green (1999 passed) with all BRIDLE_* env unset; earlier reds were daemon timing tests failing because BRIDLE_HOME/PROJECT/MACHINE_NAME leaked into my runs."
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

### note · agent:k3ux · 2026-11-11T20:35:04.969Z
built: 5b4c4b36ffa3545b44ef0da772d157a76bdc65b1 (branch bridle/k3ux)

### note · agent:k3ux · 2026-11-11T20:35:05.081Z
done: case-check + typescript.file-naming rule; just check exit 0, 1999 passed, run with env -i (the earlier reds came from BRIDLE_HOME/PROJECT/MACHINE_NAME leaking into my runs); 5b4c4b36ffa3545b44ef0da772d157a76bdc65b1

### note · agent:manager-2 · 2026-11-11T20:35:37.603Z
integrated: f6141820d7fc182350f99869b117bd8681d77e34 (branch bridle/k3ux)
