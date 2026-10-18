+++
id = "br-p2ad"
title = "add-a-machine: WSL2 bullet installs from the release script; state the release-install check (fv86 ends)"
kind = "docs"
state = "integrated"
created_at = "2026-10-17T22:13:33.033Z"
updated_at = "2026-10-18T10:26:10.775456Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wp2ad",
]
branch = "bridle/wp2ad"
commit = "558621faa71abef96688e7f22fd9e3ad586ef4bc"
summary = "Fixed the WSL2 bullet in docs/context/add-a-machine.md: it said to follow the WSL2 guide 'up to and including just install', but that guide has no such step. It now follows the guide for the OS side (Tailscale, linger, SSH), then installs from the release script like the Linux and macOS bullets. Added the release-install check: bridle --version prints the release, and bridle doctor in a project clone exits 0; outside a clone doctor is not expected to pass. windows-wsl2-host.md already installs from the release script with no just install step, so it is unchanged. CHANGELOG has a Docs line under Unreleased. Docs only, no code. just check exit 0 on the merged tip: 1868 run, 1868 passed, 6 skipped (last full count 1868)."
ticket = "p2ad"
+++

docs/tickets/open/add-a-machine-wsl2-bullet-installs-from-the-release-script-s-p2ad.md

## Thread

### note · agent:wp2ad · 2026-10-18T10:13:25.619Z
Handoff: docs edits made (add-a-machine.md WSL2 bullet and release-install check; CHANGELOG Docs line under Unreleased). just check was started in the background, logging to /tmp/p2ad-check.log; not yet confirmed green. Still to do: confirm exit 0 and the nextest Summary, write the ticket summary, commit on bridle/wp2ad, merge local main, run bridle ticket built, and report to the manager. Claim kept because the work is unfinished.

### note · agent:wp2ad · 2026-10-18T10:26:00.565Z
built: dfee4be658304a7c9ecf540e1ab1e7df1ae5b91a (branch bridle/wp2ad)

### note · agent:wp2ad · 2026-10-18T10:26:00.798Z
done: add-a-machine WSL2 bullet now installs from the release script and states the release-install check; just check exit 0 (1868 run, 1868 passed, 6 skipped); commit dfee4be658304a7c9ecf540e1ab1e7df1ae5b91a (branch bridle/wp2ad, merged local main)

### note · agent:manager-2 · 2026-10-18T10:26:10.775Z
integrated: 558621faa71abef96688e7f22fd9e3ad586ef4bc (branch bridle/wp2ad)
