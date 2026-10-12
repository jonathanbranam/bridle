+++
id = "br-3v2m"
title = "K6: the mail bridge and gateway follow a moved project"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T16:16:46.164Z"
updated_at = "2026-10-12T17:09:20.093914Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w3v2m",
]
branch = "bridle/w3v2m"
commit = "fbe95abe83cb7e58817be00cb98111f04f2e0e0f"
summary = "`project start` now moves the mail bridge (crates/bridle/src/project_mail.rs, hooked into project_start.rs carry_in after tokens and schedules, before moved-in.json is removed so a failure retries). take-over records the old owner as `from` in moved-in.json. When `[mail] projects` names P (here or in the old machine's config over ssh): copies the old [mail] section if missing, copies mail.key (0600) if this machine has none, installs the bridge unit (mail_install::install_at) and starts it. Projects with no [mail] entry are untouched. Caveats: (1) the key is copied whenever the old machine has one and this machine does not, not only when open mailed questions exist, because the old workspace's question record is not reachable after stop; if there is no key to copy, or this machine has a different one, it says replies to mail sent from the old machine will fail verification. (2) The gateway already follows [projects] (existing discovery tests cover a remote placement); no gateway change. (3) Tests cover the config/key copy with a fake old machine; the unit install uses the existing mail install code and was not run against fake homes (it reads $HOME)."
ticket = "3v2m"
+++

docs/tickets/open/k6-the-mail-bridge-and-gateway-follow-a-moved-project-3v2m.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:38.613Z
Brief (pm-1). Model: Sonnet. Size: M. The ask above is the spec; read crates/bridle-mail, the `bridle mail install` code, `project start` (53gy C, 86qp) and the gateway's `[projects]` handling (2wnn). Copy mail.key over ssh 0600 only when open mailed questions exist. Use fake homes/ssh in tests; no real mail or AWS. Files also: docs/design/cli.md, CHANGELOG.md. Out of scope: moving a project that has no [mail] entry (must keep working unchanged).

### note · agent:w3v2m · 2026-10-12T17:03:53.026Z
built: 2a10e366 (branch bridle/w3v2m)

### note · agent:w3v2m · 2026-10-12T17:03:53.110Z
done: project start moves mail bridge (config, key, unit); just check exit 0, 1610 tests run, 1610 passed; main merged (up to date); 2a10e366

### note · agent:manager-2 · 2026-10-12T17:08:57.723Z
integrated: fbe95abe83cb7e58817be00cb98111f04f2e0e0f (branch bridle/w3v2m)

### note · agent:manager-2 · 2026-10-12T17:09:20.093Z
cleanup: removed agent w3v2m, branch bridle/w3v2m
