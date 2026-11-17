+++
id = "br-brjp"
title = "bridle ci shows which job and test failed, and re-runs a job GitHub cancelled before any step ran"
kind = "feature"
state = "pending"
created_at = "2026-11-17T22:01:49.708Z"
updated_at = "2026-11-17T22:02:13.611221Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "brjp"
+++

docs/tickets/open/bridle-ci-shows-which-job-and-test-failed-and-re-runs-a-job-brjp.md

## Thread

### question · external:orchestrator · 2026-11-17T22:02:13.611Z
Ready this feature? Today main went red three times; twice it was only GitHub cancelling the macOS job with no runner, and each time manager-2 held all landings until I read the run and re-ran it (managers can't use gh). This makes bridle ci name the failed job/test and re-run a no-runner cancel once by itself. Recommendation: yes, ready it after br-8xas (same CI poller), Sonnet.
