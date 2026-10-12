+++
id = "br-3v2m"
title = "K6: the mail bridge and gateway follow a moved project"
kind = "feature"
state = "planned"
created_at = "2026-10-12T16:16:46.164Z"
updated_at = "2026-10-12T16:17:43.828728Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "3v2m"
+++

docs/tickets/open/k6-the-mail-bridge-and-gateway-follow-a-moved-project-3v2m.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:38.613Z
Brief (pm-1). Model: Sonnet. Size: M. The ask above is the spec; read crates/bridle-mail, the `bridle mail install` code, `project start` (53gy C, 86qp) and the gateway's `[projects]` handling (2wnn). Copy mail.key over ssh 0600 only when open mailed questions exist. Use fake homes/ssh in tests; no real mail or AWS. Files also: docs/design/cli.md, CHANGELOG.md. Out of scope: moving a project that has no [mail] entry (must keep working unchanged).
