+++
id = "br-cyzr"
title = "Show the installed UI version and how far it is behind bridle-ui main in bridle status"
kind = "feature"
state = "integrated"
created_at = "2026-11-26T16:30:54.066Z"
updated_at = "2026-11-26T17:32:43.868044Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wcyzr",
]
branch = "bridle/wcyzr"
commit = "d9c935fe51a15fe8016bb36a009ee013c05a2c7f"
summary = "bridle status (and --json, Status.ui, optional) now shows the installed bridle-ui: release tag and bridle-ui commit from bridle-version, or none installed / installed without a bridle-version / version unreadable. The gateway serves the same at GET /api/v1/ui (behind the session, UiVersionView) for the UI footer. Logic in crates/bridle-api/src/ui_version.rs. Behind main: commits on main of this machine's bridle-ui clone (bridle-ui entry in ~/.bridle/daemons/); no network or secret; otherwise unknown with the reason, so release-only machines show unknown. Caveat: bridle-ui's npm run install-ui is unchanged and writes no bridle-version, so dalek's UI shows version unknown until bridle-ui writes that file ('dev bridle-ui <sha>' would parse as is). No browser-visible change."
ticket = "cyzr"
+++

docs/tickets/open/ui-version-status-cyzr.md

## Thread

### note · external:orchestrator · 2026-11-26T16:31:19.964Z
readied by external:orchestrator

### note · agent:wcyzr · 2026-11-26T17:24:38.463Z
Work committed on bridle/wcyzr; 'just check' is running now. Behind-main choice: count commits on main of this machine's bridle-ui clone (the bridle-ui entry in ~/.bridle/daemons/), no network; otherwise unknown with the reason. UI: Status.ui and gateway GET /api/v1/ui. npm run install-ui is unchanged and writes no bridle-version, so such installs show as 'installed another way' with version unknown.

### note · agent:wcyzr · 2026-11-26T17:32:02.739Z
built: 403eed9ae610e3d2161c7cc4e858068c31b9ff7e (branch bridle/wcyzr)

### note · agent:wcyzr · 2026-11-26T17:32:02.849Z
done: bridle status + GET /api/v1/ui show the installed bridle-ui and how far behind main; commit 403eed9ae610e3d2161c7cc4e858068c31b9ff7e; 'env -i ... just check' exit 0, 2040 tests run, 2040 passed, 6 skipped. Behind main = local bridle-ui clone only (no network), else unknown. Follow-up for bridle-ui: npm run install-ui writes no bridle-version, so dalek shows version unknown; have it write 'dev bridle-ui <sha>' to dist/bridle-version. No browser-visible change, so no dev site run.

### note · agent:manager-2 · 2026-11-26T17:32:14.214Z
integrated: d9c935fe51a15fe8016bb36a009ee013c05a2c7f (branch bridle/wcyzr)

### note · agent:manager-2 · 2026-11-26T17:32:14.571Z
ticket file moved to docs/tickets/resolved/ui-version-status-cyzr.md

### note · agent:manager-2 · 2026-11-26T17:32:43.868Z
cleanup: removed agent wcyzr, origin branch bridle/wcyzr, branch bridle/wcyzr
