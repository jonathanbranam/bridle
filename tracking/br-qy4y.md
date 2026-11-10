+++
id = "br-qy4y"
title = "Firewall prompts came back after the 11-09 self-upgrade despite the stable signing identity (7bs2 again)"
kind = "bug"
state = "open"
created_at = "2026-11-10T17:12:10.601Z"
updated_at = "2026-11-10T17:12:55.455667Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "qy4y"
+++

docs/tickets/open/firewall-prompts-came-back-after-the-11-09-self-upgrade-desp-qy4y.md

## Thread

### note · external:aide · 2026-11-10T17:12:10.928Z
readied by external:aide

### note · external:aide · 2026-11-10T17:12:19.340Z
aide: readied at filing (kind bug, rule ready-without-asking). The human said it can wait until after 5 PM; nothing urgent. The tailscale-serve fallback changes how bridle listens, so whoever plans it should bring that choice to the human before building it.

### note · agent:pm-1 · 2026-11-10T17:12:55.455Z
pm-1 brief: investigate and report first; do NOT change how bridle listens (the aide is right that the tailscale-serve fallback is the human's choice). Model: Sonnet. The human said it can wait until after 5 PM, so it is normal priority, not preempting other work.

Do: (1) Read the br-7bs2 and br-p88z tickets (docs/tickets/resolved/) and CLAUDE.md's signing notes for what was already tried. (2) Find why a newly launched process with an unchanged designated requirement still prompts. Candidates to test, on this Mac with the installed binary: does the application firewall key the allow-list entry on the cdhash or path for a self-signed leaf (compare `codesign -dvvv` and `socketfilterfw --listapps` before and after an upgrade; `log show` for com.apple.alf around a launch); does launchd relaunch via a path or a temp copy that differs; does the entry vanish or get re-added on each upgrade (list apps before and after). Do not run anything that changes the human's firewall settings beyond what a normal `just install` does; ask on this thread before any step that does. (3) Post the findings as a comment: the cause, whether signing can fix it (and the exact change) or cannot, and what the tailscale-serve fallback would change (listeners on 127.0.0.1, tailscale serve in front, effect on multi-machine access and on `bridle` discovery), with a recommendation. (4) If signing can fix it with a small change inside the existing installers (signing::install and the self-upgrade), you may build and test that fix on a branch; if the answer is the fallback, stop after the report and say so: the human decides.
Also (small, only if (3) shows the post-restart detection gap is separate): note it for br-t9j4 rather than building it here.
Acceptance: a findings comment with evidence; a signing fix only if it is small and `just check` passes. Out of scope: tailscale serve, changing listeners, br-9qws and br-p29s (held).
