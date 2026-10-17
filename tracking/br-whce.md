+++
id = "br-whce"
title = "project start: a failed mail step must not delete moved-in.json (the old machine's [mail] and mail.key copy is lost)"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T21:14:28.552Z"
updated_at = "2026-10-17T03:31:32.016978Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:whce",
]
branch = "bridle/whce"
commit = "5944b7658b98c4a5783b2b503caa2fbaf7534ca2"
summary = "project_start.rs: mail_step now returns success; moved-in.json is removed (finish_moved_in) only if the mail step succeeded, so a failed one is retried on the next project start. Test added; CHANGELOG line; design doc already said 'until all of it succeeds'."
ticket = "whce"
+++

docs/tickets/open/project-start-a-failed-mail-step-must-not-delete-moved-in-js-whce.md

## Thread

### note · external:aide · 2026-10-16T21:42:46.302Z
The human, 2026-10-16 ~5:35 PM ET (via aide), approving tvrp, whce, 8kzq, 9qws, vmrc, r8jf: "Yes that looks good." Readied by aide.

### note · agent:whce · 2026-10-17T01:58:15.285Z
built: 0aa0dbe4aa58973822939dc51e4f01c94150ea78 (branch bridle/whce)

### note · agent:whce · 2026-10-17T01:58:15.353Z
done: failed mail step keeps moved-in.json; just check exit 0, 1777 tests passed; 0aa0dbe4aa58973822939dc51e4f01c94150ea78

### note · agent:manager-2 · 2026-10-17T01:58:20.259Z
Please merge main into your branch again (main moved), confirm just check, and message me.

### note · agent:whce · 2026-10-17T02:12:33.754Z
merged main again; just check exit 0, 1777 passed; c2a39472f0fc728e26eaa495e1e21e4126c18443

### note · agent:manager-2 · 2026-10-17T03:31:07.272Z
integrated: 5944b7658b98c4a5783b2b503caa2fbaf7534ca2 (branch bridle/whce)

### note · agent:manager-2 · 2026-10-17T03:31:32.016Z
cleanup: removed agent whce, branch bridle/whce
