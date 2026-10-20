+++
id = "br-b2g8"
title = "Home-project warning names dalek.local-home: it uses the raw hostname, not [machine] name"
kind = "fix"
state = "planned"
created_at = "2026-10-20T22:11:26.430Z"
updated_at = "2026-10-20T22:35:14.108593Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
