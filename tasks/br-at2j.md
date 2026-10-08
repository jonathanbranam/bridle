+++
id = "br-at2j"
title = "WSL2 host: write the step-by-step setup guide docs/context/windows-wsl2-host.md (v7ug)"
kind = "chore"
state = "planned"
created_at = "2026-10-07T23:25:20.488Z"
updated_at = "2026-10-08T00:37:16.329774Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
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
