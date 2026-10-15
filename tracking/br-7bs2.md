+++
id = "br-7bs2"
title = "macOS firewall still blocks a freshly launched bridle (gateway) after a self-upgrade, despite the stable signing identity (p88z)"
kind = "bug"
state = "open"
created_at = "2026-10-15T23:15:13.826Z"
updated_at = "2026-10-15T23:20:59.346814Z"
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

### note · agent:pm-1 · 2026-10-15T23:20:59.346Z
Brief (pm-1). Model: Sonnet. Size: M, investigation first. High priority (the human hits Allow on most laptop opens; a missed one cuts the gateway off from the phone and other machines). Human's decision: option 1 now (find why the firewall ignores the "bridle local signing" identity, fix the signing/upgrade path); option 2 optional; NEVER disable the firewall; 127.0.0.1 + tailscale serve is the fallback only if 1 cannot be fixed. Do not build the fallback without telling the manager first.
Step 1 (report findings on the thread before building): on dalek (this machine, macOS), reproduce with a controlled experiment, not guesses: which path does the firewall list (`/usr/libexec/ApplicationFirewall/socketfilterfw --listapps`) hold for bridle, and does the entry survive a replaced file? Test hypotheses: (a) the self-upgrade/`bridle sign binary` replaces the file at ~/.cargo/bin/bridle (new inode) and the firewall entry is tied to the old file/cdhash, so rewriting the signed binary IN PLACE (same inode) or signing before the final rename may keep it; (b) the entry is keyed on cdhash not the designated requirement for a self-signed leaf; (c) a plain launchd restart (same file, no re-sign) does not re-prompt, which isolates the signing step. Use a throwaway copy of a small signed binary listening on a spare port for the experiments, not the live gateway; avoid prompting the human (note any dialog that appears). Read crates/bridle sign code (git grep -n "bridle local signing\|codesign" crates/ justfile), docs/tickets/resolved for p88z, and the self-upgrade install path (crates/bridle-daemon upgrade/install).
Step 2: fix the cause in the install/sign path so an upgrade leaves the firewall decision intact; document in CLAUDE.md "Build configuration"/docs/design/agent-host as the finding dictates, plus CHANGELOG.
Step 3 (only if cheap, optional per the human): after a restart the gateway/daemon self-checks reachability on its tailnet address and raises a notice/incident naming the macOS firewall as the likely cause, so the human is not the detector.
Stop points: land step 1's findings (a docs note on the ticket thread) and step 2 if found; if no cause is found, say what was ruled out and stop, do not guess a fix. Option 2 (sudoers + socketfilterfw --add) is out of scope unless the manager asks: it needs the human's sudoers edit. Acceptance: just check; for the fix, verified across a real self-upgrade or a simulated binary replace on dalek, stated in the done note.
