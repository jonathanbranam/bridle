+++
id = "br-tvrp"
title = "Release-mode self-upgrade does not re-sign: the gateway is firewall-blocked again after a release upgrade on a Mac"
kind = "fix"
state = "claimed"
created_at = "2026-10-16T21:14:23.982Z"
updated_at = "2026-10-17T01:38:14.428347Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:tvrp",
]
ticket = "tvrp"
+++

docs/tickets/open/release-mode-self-upgrade-does-not-re-sign-the-gateway-is-fi-tvrp.md

## Thread

### note · external:aide · 2026-10-16T21:41:43.128Z
The human, 2026-10-16 ~5:35 PM ET (via aide): "Yes that looks good." / "Bridle work will be switching to Remus soon so yes it needs proper signing or something so that releases installed on OS X don't have the firewall issue. Can GH sign?" Note: remus has no 'bridle local signing' identity yet (just sign-setup not run there).

### note · agent:pm-1 · 2026-10-16T21:41:53.169Z
pm-1: reading the human's note. The brief stands (re-sign on install). Two facts for the human's "can GH sign?":
- Re-signing on the Mac at install time with the local "bridle local signing" identity is what keeps the firewall's Allow across upgrades (the entry is identifier + certificate leaf). It needs `just sign-setup` once on each Mac; remus has not run it, so run it there before switching bridle work to remus, or each release install gets an ad-hoc signature and the firewall re-asks.
- Signing in CI with a self-signed certificate would give a different leaf than each machine's local identity, so it would not help; only an Apple Developer ID certificate (paid) signed in CI gives one identity everywhere, and it adds notarization work. Recommend: stay with local signing at install (this ticket), no CI signing. The human decides if they want Developer ID later (a separate ticket).
Add to the brief: if no signing identity is found, release::install keeps the ad-hoc signature and logs one line saying the firewall may ask again and to run `just sign-setup`.

### note · external:aide · 2026-10-17T01:12:18.401Z
Correction (aide): remus is Linux, not a Mac. Code signing and the macOS app firewall don't apply there, so 'just sign-setup on remus' is not needed. tvrp matters for the Macs (dalek, and any Mac that installs releases).

### note · external:aide · 2026-10-17T01:36:59.308Z
The human, 2026-10-16 (via aide), on re-signing releases at install on a Mac: "Then build that"

### note · agent:manager-2 · 2026-10-17T01:37:14.261Z
Priority: the human wants this first. Also if no signing identity is found, keep ad-hoc and log one line suggesting just sign-setup (see pm-1's note). Merge main, just check, commit, message me as soon as done.

### note · external:advisor/product-manager · 2026-10-17T01:38:14.428Z
The human, 2026-10-16 ~9:45 PM ET, to advisor (product-manager): "Dalek is the only macos". So no 'just sign-setup' on remus (not a Mac). This fix matters for dalek once bridle work moves to remus and dalek becomes a release client; until then dalek runs commit mode (self_upgrade = false + bridle's true, br-mtdg).
