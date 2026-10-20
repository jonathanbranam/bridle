+++
id = "br-b2g8"
title = "Home-project warning names dalek.local-home: it uses the raw hostname, not [machine] name"
kind = "fix"
state = "integrated"
created_at = "2026-10-20T22:11:26.430Z"
updated_at = "2026-10-20T23:01:29.110310Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wb2g8",
]
branch = "bridle/wb2g8"
commit = "9e2b4858c685490ec2924ac2b33bf7afcff52770"
summary = "The home-project warning already takes [machine] name before the hostname (machine_home in crates/bridle-api/src/machines.rs, via owner_names), so with name = dalek it names dalek-home. The dalek.local-home seen in the thread came from the stale installed binary (bridle 0.7.6 on PATH); the current build does not show it. Added a test (the_home_is_named_from_the_machine_name_not_the_raw_hostname) pinning dalek-home and no dalek.local. Other <host>-home builders checked: up.rs orchestrator_command reads [machine] name only (no hostname fallback), token_pair.rs machine_home takes the holder's [machines] name; none use the raw hostname. Open question, not changed: with no [machine] name, machine_home falls back to the raw hostname, so an unnamed dalek.local still gets dalek.local-home; the fix there would strip the domain, which changes owner_names semantics, so it is left for a decision."
ticket = "b2g8"
+++

docs/tickets/open/home-project-warning-names-dalek-local-home-it-uses-the-raw-b2g8.md

submitted by external:orchestrator

## The ask

The warning that tells a sender to address the orchestrator with `--project <host>-home` builds the host from the raw `hostname`. On dalek that is `dalek.local`, so it says `--project dalek.local-home`, which is not a known daemon. The home project is `dalek-home`. Take the host from `[machine] name`, then `hostname` (the same function the daemon uses for owner.toml, as br-pcde did for the mail bridge), and grep for any other `<host>-home` builders using the raw hostname.

## Seen

manager-2 (bridle), 2026-10-20 22:10 UTC, m-0370 to external:orchestrator: "--project dalek.local-home, the name the warning gave, is not a known daemon here, so I sent without it."

## Check

A test: with `[machine] name = "dalek"` and hostname `dalek.local`, the warning names `dalek-home`.

Model: haiku or sonnet; small change. Related: br-pcde.

## Thread

### note · external:orchestrator · 2026-10-20T22:11:26.431Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-20T22:11:58.648Z
orchestrator: readied (fix, self-evident ask; rule ready-without-asking).

### note · agent:wb2g8 · 2026-10-20T23:00:53.359Z
built: 58ec91135baf50e587bca1bdfb00a8c846e95130 (branch bridle/wb2g8)

### note · agent:wb2g8 · 2026-10-20T23:00:53.757Z
done: home warning test pins dalek-home; no code change needed (machine_home already uses [machine] name first); dalek.local-home came from the stale installed bridle 0.7.6; just check exit 0, 1942 passed; 58ec9113

### note · agent:manager-2 · 2026-10-20T23:01:06.645Z
integrated: 9e2b4858c685490ec2924ac2b33bf7afcff52770 (branch bridle/wb2g8)

### note · agent:manager-2 · 2026-10-20T23:01:29.110Z
cleanup: removed agent wb2g8, origin branch bridle/wb2g8, branch bridle/wb2g8
