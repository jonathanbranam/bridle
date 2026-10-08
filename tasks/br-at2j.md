+++
id = "br-at2j"
title = "WSL2 host: write the step-by-step setup guide docs/context/windows-wsl2-host.md (v7ug)"
kind = "chore"
state = "planned"
created_at = "2026-10-07T23:25:20.488Z"
updated_at = "2026-10-08T01:05:06.518821Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
summary = '''Added docs/context/windows-wsl2-host.md: the human's step-by-step guide to turn the Windows PC into a bridle host under WSL2 (Ubuntu), modelled on docs/context/nuc-host.md. Six numbered steps, each with commands and a check: (1) wsl --install -d Ubuntu; (2) %UserProfile%\.wslconfig (vmIdleTimeout=-1, networkingMode=mirrored, memory/processors left as fill-in placeholders because CPU and RAM are unknown) and /etc/wsl.conf (systemd=true); (3) Tailscale inside WSL, not on Windows, as the NUC does; (4) Task Scheduler start-up task plus green (schtasks /run) and red (wsl.exe --shutdown) desktop shortcuts; (5) power and Windows Update settings, and unattended-upgrades Automatic-Reboot off; (6) toolchain (rustup, just, cargo-nextest, tmux, claude) and bridle (just install), then bridle systemd install, marked to be verified under WSL's systemd (audit task br-4yc8). Steps not confirmed on the PC are marked unverified. Index row added in docs/README.md. Decisions: Tailscale inside WSL (the Linux host gets its own name, like the NUC); the red and green buttons are desktop shortcuts, not built. Caveat: the human's PC name, CPU, RAM and active hours are still <fill in> placeholders. Docs only; no code changed.'''
+++

Ticket: docs/tickets/open/run-bridle-s-heavy-work-on-the-windows-pc-under-wsl2-v7ug.md (read it, and docs/tickets/open/ ur9v: the-windows-gaming-pc-as-a-bridle-host-without-wiping-window-ur9v, for the research and the human's answers: Windows Home, BitLocker off, plenty of disk, gaming means bridle stops, a big red / big green button).
Goal: ONE new doc, docs/context/windows-wsl2-host.md, a guide the human follows step by step to turn the Windows PC into a bridle host under WSL2 (Ubuntu). Model its shape on docs/context/nuc-host.md (the machine, how it's reached, how the human uses it, how bridle runs, moving a project there), but the core is a numbered checklist of the human's hands-on steps, each with the exact command or setting and how to check it worked:
1. Install WSL2 + Ubuntu (`wsl --install -d Ubuntu`).
2. `%UserProfile%\.wslconfig` and `/etc/wsl.conf`: `systemd=true`, idle timeout / `vmIdleTimeout`, `networkingMode=mirrored`, memory and processors limits (leave room for gaming; note CPU and RAM are still unknown: put a "fill in" line).
3. Tailscale (in Windows or in WSL: state which and why, using what nuc-host.md does).
4. A Windows Task Scheduler task that starts WSL at boot/login (so systemd units come up), and the two desktop shortcuts: green = the start-up task, red = `wsl --shutdown` (the gaming button).
5. Power and Windows Update settings (no sleep while bridle runs; active hours so updates do not reboot mid-run).
6. Install the toolchain (rustup, just, cargo-nextest, git, tmux, claude) and bridle (`just install`), then `bridle systemd install` (mark this step "to be verified under WSL's systemd" until br-<the audit task> reports).
State assumptions plainly and mark anything unverified as "unverified". ASCII only (rule ascii-in-editable-text). Link other docs root-relative in wiki form.
Out of scope: changing any code; naming the PC (a human to-do); the bridle-side audit (separate task). Add the doc to the docs/README.md index.
Acceptance: just check passes (docs only); the doc has all six steps with commands and checks. Model: Haiku.

## Thread

### note · agent:pm-1 · 2026-10-07T23:25:33.292Z
Step 6 refers to the audit task: it is br-4yc8 (WSL2 audit). Link it by that id in the guide.

### note · agent:pm-1 · 2026-10-08T00:37:16.329Z
pm-1: the audit landed (ticket v7ug, section Audit). Fold its facts into the guide: Tailscale must run INSIDE the Ubuntu distro (needs systemd), not only on Windows; the Task Scheduler start-up task must run a held-open command (e.g. 'wsl -d Ubuntu -- sleep infinity') and .wslconfig needs vmIdleTimeout raised, or linger alone does not keep WSL up; work in the Linux home, never /mnt/c; apt packages: build-essential, pkg-config, cmake, git, tmux (verify the list); 'claude auth login' headless. Follow-ups are tasks, not the guide's job.

### note · agent:wsl2guide · 2026-10-08T01:01:51.833Z
just check on bridle/wsl2guide (docs-only change) exits 100: 1 failed test, upgrade_test::a_drain_holds_new_turns_and_delivers_them_after_the_restart (panic at crates/bridle-daemon/tests/upgrade_test.rs:462). It failed on two full runs and passes alone (1 passed). The code is identical to main, so this is a pre-existing flake under parallel load, not from this change. Not fixed (out of scope; the test must not be loosened). Changes left uncommitted until the check is green.

### note · agent:wsl2guide · 2026-10-08T01:05:06.518Z
done: guide committed and main merged; NOT green: just check exits 100 on the tip, 1 failed test = upgrade_test a_drain_holds_new_turns_and_delivers_them_after_the_restart (same flake as before: passes alone, code identical to main; 1179 of 1180 passed). Tip sha 2139de7749e1df09964a9c0272b7be73c0881eb6 (merge of main ac05fb20; guide commit below it). Land only with the flake waived, or after it's fixed.
