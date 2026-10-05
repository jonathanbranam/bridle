+++
id = "br-mh62"
title = "Specs: guidance for behavior that spans repos or lives in an entry script"
kind = "feature"
state = "dropped"
created_at = "2026-10-05T02:51:25.891Z"
updated_at = "2026-10-05T02:52:20.682165Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

submitted by external:orchestrator@nuc

Two kinds of scenario ended up non-executable in meta-notes-ui's access spec: (1) behavior in a top-level entry script (server start, server.json, default host, exit codes), testable only by spawning the built program; (2) behavior split across repos (meta-notes' `ui start/stop/status` prints and opens the token URL that meta-notes-ui serves), which no single repo's spec can own. Wanted: guidance (and vitest-bridle helpers) for process-level steps, and a convention for where a cross-repo requirement lives and how each side tests its half.

From the meta-notes-ui project (orchestrator, 2026-10-05), adopting bridle specs with vendored vitest-bridle (mu-943b, mu-hzf9). Local log: meta-notes-ui docs/tickets/open (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:51:25.891Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T02:52:20.682Z
dropped: Declined for now: this is a spec-flow design question (process-level steps, cross-repo requirements) with no concrete failing case yet, and non-executable scenarios already work for these. With one adopter it is too early to fix a convention. Keep it in meta-notes-ui's friction log; when a second project hits the same thing, or the access spec's non-executable scenarios cause a real miss, re-submit with the concrete case and we will ticket it.
