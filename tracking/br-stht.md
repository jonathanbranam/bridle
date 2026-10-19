+++
id = "br-stht"
title = "CI red on main after br-rdag: three migrate::tests startup_* tests fail on ubuntu (no git identity on the runner)"
kind = "fix"
state = "integrated"
created_at = "2026-10-19T16:37:19.826Z"
updated_at = "2026-10-19T20:02:31.511390Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:wstht",
]
branch = "bridle/wstht"
commit = "dfe7eac35e67109d06d24f36f87af3771261c0ab"
summary = "migrate::tests git_project() now sets a local user.name/user.email after git init, so the startup commit tests pass on a runner with no global git identity. Verified with GIT_CONFIG_GLOBAL=/dev/null and an empty HOME. No production change needed."
ticket = "stht"
+++

docs/tickets/open/ci-red-on-main-after-br-rdag-three-migrate-tests-startup-tes-stht.md

## Thread

### note · external:aide · 2026-10-19T19:38:04.799Z
Readied by aide (a fix; rule ready-without-asking). manager-2 left it uncommitted in the main checkout for ~1.5 h while main stayed red, so aide committed the ticket file.

### note · agent:wstht · 2026-10-19T20:02:23.295Z
built: 3cd9320e545af262a7a733842486e34bff099d73 (branch bridle/wstht)

### note · agent:wstht · 2026-10-19T20:02:23.406Z
done: git_project() sets local git identity; just check exit 0, 1892 tests; 3cd9320e

### note · agent:manager-2 · 2026-10-19T20:02:31.511Z
integrated: dfe7eac35e67109d06d24f36f87af3771261c0ab (branch bridle/wstht)
