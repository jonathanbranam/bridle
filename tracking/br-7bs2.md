+++
id = "br-7bs2"
title = "macOS firewall still blocks a freshly launched bridle (gateway) after a self-upgrade, despite the stable signing identity (p88z)"
kind = "bug"
state = "open"
created_at = "2026-10-15T23:15:13.826Z"
updated_at = "2026-10-15T23:20:14.659051Z"
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

### note · external:aide · 2026-10-15T23:20:04.754Z
The human's decision, 2026-10-15 ~7:25 PM ET, verbatim: "(1) for now. 2 optional. I agree not 3. If we can't fix it I'd rather use Tailscale than disable the firewall". Options as put to them by the aide: (1) find why the firewall ignores the 'bridle local signing' signature and fix that (e.g. a self-upgrade replaces the file at the path, so the firewall entry may be tied to the old file; rewriting in place might fix it), investigate on dalek first; (2) optional: re-add bridle to the firewall after each upgrade with socketfilterfw --add via a one-time NOPASSWD sudoers entry; (3) rejected: disable the macOS firewall. Fallback if (1) can't be fixed: 127.0.0.1 + tailscale serve (preferred over disabling the firewall). This supersedes the earlier 'don't build that direction' note: Tailscale is the fallback, not ruled out.

### note · external:advisor/product-manager · 2026-10-15T23:20:14.659Z
Readied by advisor product-manager on the human's direction via aide (m-13700): "(1) for now. 2 optional. I agree not 3. If we can't fix it I'd rather use Tailscale than disable the firewall". Build option 1 (find why the firewall ignores the signing identity on dalek; fix the signing/upgrade path); option 2 optional; never disable the firewall; tailscale serve is the fallback.
