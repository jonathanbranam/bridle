+++
id = "br-vmrc"
title = "Mail digest stamps with the host's local zone but says US Eastern; fix is_bare_sent_notice's doc comment"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T21:14:31.286Z"
updated_at = "2026-10-17T05:13:46.165883Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:vmrc",
]
branch = "bridle/vmrc"
commit = "4d22dba2d9ddf2a08b41ba31b97000f7ad103632"
summary = "Mail digest now stamps real US Eastern: bridge.rs passes chrono_tz America/New_York (added chrono-tz to bridle-mail) for the digest's now (so digest_at is Eastern too) and item stamps; header says 'Times are US Eastern.'. is_bare_sent_notice doc comment now matches the code (whole body is a bare sent line, not a reply, no incident). Test: DST-aware Oct/Jan stamps. Docs: mail.md, config.rs doc, CHANGELOG."
ticket = "vmrc"
+++

docs/tickets/open/mail-digest-stamps-with-the-host-s-local-zone-but-says-us-ea-vmrc.md

## Thread

### note · external:aide · 2026-10-16T21:42:46.620Z
The human, 2026-10-16 ~5:35 PM ET (via aide), approving tvrp, whce, 8kzq, 9qws, vmrc, r8jf: "Yes that looks good." Readied by aide.

### note · agent:vmrc · 2026-10-17T04:48:54.161Z
built: c8296011c83e3cb3b67b0196861d86dfbdd602ea (branch bridle/vmrc)

### note · agent:vmrc · 2026-10-17T04:48:54.252Z
done: digest stamps real US Eastern, doc comment fixed; just check exit 0, 1787 tests passed, after merging main; c8296011c83e3cb3b67b0196861d86dfbdd602ea

### note · agent:manager-2 · 2026-10-17T05:13:46.165Z
integrated: 4d22dba2d9ddf2a08b41ba31b97000f7ad103632 (branch bridle/vmrc)
