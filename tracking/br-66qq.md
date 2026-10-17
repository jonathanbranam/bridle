+++
id = "br-66qq"
title = "vesta-home, meta-notes and meta-notes-ui daemons never self-upgraded to 0.7.6 (self_upgrade = release), while notes and dotfiles-local did"
kind = "bug"
state = "integrated"
created_at = "2026-10-17T11:54:07.124Z"
updated_at = "2026-10-17T16:59:38.795653Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/w66qq"
commit = "0d48e217d5ad51afc621d51b61e07576fcd237d8"
summary = "Cause (from code, unconfirmed on vesta): the release check ran only as a tail of the CI watcher's tick, after ci.tick(); RealGh::run (gh/git) has no timeout, so a hung gh/git in one daemon stopped its CI loop, and with it the release check, silently (failures were debug-level). Fix: release daemons get their own loop (lib.rs); lookup failure is now warn, newer tag found is info. Test: a_hung_ci_watch_does_not_stop_the_release_check. Not fixed: gh/git still have no timeout (CI watch for that daemon can still hang). Docs daemon.md + CHANGELOG updated."
ticket = "66qq"
+++

docs/tickets/open/vesta-home-meta-notes-and-meta-notes-ui-daemons-never-self-u-66qq.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T15:26:11.229Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:w66qq · 2026-10-17T16:38:09.952Z
done: release check moved to its own loop (a hung gh in the CI tick likely blocked it; unconfirmed, no vesta access) + warn/info logging; just check exit 0, 1821 tests passed; 1fb0d3d3. 'bridle ticket built' was refused for me (forbidden: agent lifecycle endpoints), so not marked built. Out of scope: gh/git calls in ci.rs RealGh::run still have no timeout.

### note · agent:manager-2 · 2026-10-17T16:38:13.996Z
built: 1fb0d3d3 (branch bridle/w66qq)

### note · agent:manager-2 · 2026-10-17T16:59:18.209Z
integrated: 0d48e217d5ad51afc621d51b61e07576fcd237d8 (branch bridle/w66qq)

### note · agent:manager-2 · 2026-10-17T16:59:38.795Z
cleanup: removed agent w66qq, branch bridle/w66qq
