+++
id = "br-7bs2"
title = "macOS firewall still blocks a freshly launched bridle (gateway) after a self-upgrade, despite the stable signing identity (p88z)"
kind = "bug"
state = "pending"
created_at = "2026-10-15T23:15:13.826Z"
updated_at = "2026-10-15T23:18:47.380272Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
priority_at = "2026-10-15T23:17:44.940888Z"
ticket = "7bs2"
+++

docs/tickets/open/macos-firewall-still-blocks-a-freshly-launched-bridle-gatewa-7bs2.md

## Thread

### note · external:aide · 2026-10-15T23:17:29.636Z
The human, 2026-10-15 ~7:20 PM ET, verbatim: "Pretty much every time I open the laptop there is another Allow dialog showing. I've clicked it a dozen times in the past week". So the Allow does not stick under the 'bridle local signing' identity either: the 'click Allow once more' workaround is ruled out. Recommended direction (aide): stop bridle being an inbound listener the firewall sees (127.0.0.1 + tailscale serve), plus the post-restart reachability check.

### note · external:aide · 2026-10-15T23:17:29.678Z
priority: normal -> high

### note · external:aide · 2026-10-15T23:17:34.243Z
priority: high -> normal

### note · external:advisor/product-manager · 2026-10-15T23:17:44.940Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-15T23:17:45.119Z
advisor product-manager: priority high (PdM call): the human hits the Allow dialog on most laptop opens, and a missed one cuts the gateway off from the phone and the other machines.

### note · external:aide · 2026-10-15T23:18:47.380Z
The human, 2026-10-15 ~7:20 PM ET, verbatim, on the aide's recommended fix (bridle listens on 127.0.0.1, fronted by tailscale serve): "I don't like that solution." Don't build that direction.
