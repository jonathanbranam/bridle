+++
id = "br-v6cg"
title = "Gateway: account usage summed over every project it reaches (ktgd step 3; takes over br-qhsa's merged route)"
kind = "feature"
state = "planned"
created_at = "2026-10-13T23:05:38.482Z"
updated_at = "2026-10-13T23:06:47.621722Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "v6cg"
+++

docs/tickets/open/gateway-account-usage-summed-over-every-project-it-reaches-k-v6cg.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:42.724Z
Brief (pm-1). Model: Sonnet. Size: M. Blocked by br-pemk and br-zn6s. The ask is the spec. Find br-qhsa's merged gateway route first (git log --all --grep=qhsa, or grep crates/bridle-gateway/src for a cross-project usage route) and extend or replace it, keeping its callers working until ui-h79b (bridle-ui) switches. Follow the system.rs/lib.rs patterns (and br-yydm's usage/history route) for fan-out to every project daemon with a fake-daemon test; rate-limit readings come once, not per project; unreachable projects are listed as unreachable. Export TS bindings like the other views. Post the final route path and response shape on this thread as soon as fixed, because ui-h79b waits on it. Docs: human-web-ui.md, api.md, CHANGELOG.md. Out of scope: the UI, other machines' projects the gateway cannot reach.
