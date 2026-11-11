+++
id = "br-qy4y"
title = "Firewall prompts came back after the 11-09 self-upgrade despite the stable signing identity (7bs2 again)"
kind = "bug"
state = "integrated"
created_at = "2026-11-10T17:12:10.601Z"
updated_at = "2026-11-11T13:43:18.421987Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:qy4y",
]
commit = "b4fd85a4"
summary = "Investigated, no code change. alf.plist requirement matches the signing DR exactly; cause not proven (stale file alias in the ALF entry is the lead, contradicted by ~20 earlier unprompted upgrades; reboot Nov 8 is the other). No small signing fix exists; tailscale-serve fallback is the human's call. Findings and an experiment needing approval are on the ticket thread."
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

### note · agent:qy4y · 2026-11-10T17:59:20.384Z
qy4y findings (investigate-only; no code changed, no firewall setting touched; everything below was read-only on dalek).

CAUSE: not proven. What the evidence rules out and what it leaves.

Ruled out (signing identity / designated requirement):
- /Library/Preferences/com.apple.alf.plist has one bridle entry. Its reqdata decodes to `identifier "bridle" and certificate leaf = H"37e545f6...80e0e6c3"`, byte for byte the installed binary's designated requirement (`codesign -d -r-`). state 0 (allow). So the stored requirement matches; 7bs2's fix did what it said.
- launchd runs the same path (~/.cargo/bin/bridle) every time; no temp-copy path.

What the log shows (/usr/bin/log, socketfilterfw): after the 12:11:00 relaunch, ALF gave three "DoAnswer: received filtering decision" at 12:12:06, 12:12:08, 12:12:08 (the human's clicks). Three prompts = the three newly launched listeners (gateway, bridle-ui, track-web). The long-running 7401/7407/7410 did not prompt, as 7bs2 saw.

Odd facts, which point at the entry's alias rather than the requirement:
1. The plist's mtime is still Oct 15 21:41, so neither the Nov 9 upgrade nor the Nov 9 Allow clicks rewrote the entry.
2. The entry also holds an old-style file alias for the bridle file. It records file creation date 2026-10-15 18:41:36. The installed file now was born Nov 9 12:10:57 (inode 191922223). Every install makes a new file (copy, sign, rename in `signing::install`), so that alias has been stale after every upgrade since Oct 15.
3. But daemon.log shows ~20 upgrades through the same installer from Oct 19 to Nov 1, identical flow ("installed binary signature", then "restarted launchd jobs" for the same 6 jobs), and the human reported no prompts until Nov 9. So a stale alias alone does not explain it, or the earlier ones were not noticed.
4. Dalek rebooted Nov 8 08:46, and ALF logged a DoAnswer at 08:48 (a prompt answered right after boot). The unified log here is retained only from Nov 7, so I cannot see whether older upgrades prompted.

So I cannot say whether ALF keys the new-pid decision on the alias (file id / creation date), on kernel state lost at the Nov 8 reboot, or on something else. Candidate list, most to least likely: (a) the stale file alias (breaks every upgrade, but earlier upgrades apparently did not prompt: contradiction); (b) kernel per-binary state lost at the reboot, then re-asked once per newly launched process and not persisted; (c) the human's Allow clicks not persisting for this entry.

CAN SIGNING FIX IT? Nothing I can name. The requirement already matches. If the cause is (a), the only fix inside our installers is to overwrite the file in place (keeping inode and creation date). That is unsafe: macOS kills running processes whose pages change under them, and is exactly why 7bs2 moved to copy-sign-rename. The other way to refresh the entry is `sudo socketfilterfw --add/--unblockapp <path>`, which needs root; the daemon is not root, and I will not run it (it changes the firewall's settings). So there is no small signing change to build here.

THE FALLBACK (not built, needs the human): bridle listens on 127.0.0.1 only and `tailscale serve` (tailscaled, itself already in the ALF list as io.tailscale.ipn.macsys.network-extension) fronts the tailnet ports. Effects: (1) the firewall never sees bridle as an inbound listener, so no prompts ever, upgrade or reboot; (2) every non-loopback consumer changes: remote machines reach daemon 7401, UI 7405, track-web 7406, gateway 7878, 7407, 7410 through `tailscale serve` mappings (one per port, https or tcp), so the daemon's advertised/discovery address (bridle discovery and the multi-machine config) must become the serve address, and each project/machine needs a serve rule created and kept (a new install step, per port); (3) sees a tailscale-authenticated source instead of the raw peer address, which affects anything that logs or trusts the peer IP; (4) the post-restart self-check (t9j4) changes to check the serve address. This is a design change to how bridle listens, so per the brief I stop here.

RECOMMENDATION: do not build either yet. First one cheap experiment to pick the cause, which needs the human's OK because it can raise a dialog: on the next ordinary `just install` / self-upgrade, take `plutil -p` of the alf.plist bridle entry and `stat` of the binary before and after, and watch `log show --predicate 'process == "socketfilterfw"'`. If the prompts return with an unchanged requirement and a changed file, the alias is the key (cause a), and the fallback is the only guarantee short of in-place overwrite; if they do not return, cause (b) (one prompt after a reboot) is likely and the answer is to live with it or detect it. In the meantime I can run the experiment now (start a second `~/.cargo/bin/bridle` listener on a spare port, which may raise one dialog on dalek) if manager-2 or the human says yes; I have not.

SEPARATE (for br-t9j4, not built here): on Nov 9 no incident fired. The detection gap is the one already filed in t9j4 and 9qws (a self-connect from the same host may not be filtered by ALF); it is independent of this cause.

### note · agent:qy4y · 2026-11-10T17:59:20.450Z
qy4y: findings posted on the ticket thread. Cause not proven; no signing fix (requirement matches; only in-place overwrite or root socketfilterfw would refresh the entry). Fallback is the human's call; I stopped. One experiment needs a yes from you/the human (it may raise a dialog). No code changed, nothing to merge.

### question · external:aide · 2026-11-11T13:07:01.978Z
aide, for the human: what next on the firewall prompts? (from qy4y's findings above)

Cause not proven. Signing isn't it: the firewall's stored requirement matches the binary byte for byte. Lead: the entry also keeps a pointer to the bridle file, which goes stale at every upgrade because each install is a new file. But ~20 upgrades from Oct 19 to Nov 1 didn't prompt, so the other candidate is state lost at the 11-08 reboot. No signing fix exists: refreshing the entry needs an in-place overwrite (crashes running processes) or root.

A. Watch the next upgrade (aide recommends). Record the firewall entry and the binary before and after the next normal self-upgrade or `just install`, to tell which cause it is. Cost: maybe one more round of Allow clicks.
B. Switch now: bridle listens on 127.0.0.1 only and `tailscale serve` fronts the tailnet ports. Ends the prompts for good, but every port needs a serve rule on every machine, and discovery must use the serve addresses.
C. Live with it, and only add the missing incident (br-t9j4) so a block is reported, not found from a dialog.

Aide's recommendation: A, and build the t9j4 warning either way.

### answer · external:aide · 2026-11-11T13:09:34.241Z
The human, 2026-11-11 (to aide), verbatim: "Yeah, that's good. Let's go with C to do the warning." (Earlier: "I think I'm fine with probably just seeing live with it.") So: no investigation (A) and no tailscale-serve switch (B). The human clicks Allow when prompts appear; the work is a warning that fires when bridle is blocked. That warning exists (br-t9j4 landed) but stayed silent on 11-09; making it fire is br-9qws. Nothing to build on this ticket: close it.

### note · agent:qy4y · 2026-11-11T13:09:40.297Z
qy4y: human chose C (live with prompts, warning tracked in br-9qws). Nothing to merge; ready for you to resolve br-qy4y.

### note · agent:qy4y · 2026-11-11T13:09:42.075Z
qy4y: human chose C (live with it, warning via br-9qws). No code changed. Releasing my claim; manager can resolve.

### note · external:orchestrator · 2026-11-11T13:43:18.421Z
integrated: b4fd85a4
