+++
id = "br-4yc8"
title = "WSL2 host: what bridle needs changed to run on WSL2, and which work moves (audit + recommendation on v7ug)"
kind = "research"
state = "planned"
created_at = "2026-10-07T23:25:26.046Z"
updated_at = "2026-10-07T23:25:53.791096Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
+++

Ticket: docs/tickets/open/run-bridle-s-heavy-work-on-the-windows-pc-under-wsl2-v7ug.md (the "Bridle work, if any" section). Research from the code and docs; no build, and nothing needs the PC to exist (it is not set up yet), so mark what only the real machine can confirm as "verify on the PC".
Answer, written as findings on the ticket (a section "Audit"), each with the file or doc that backs it:
1. Does anything in bridle assume macOS or a non-WSL Linux? Check `bridle systemd install` (crates/bridle, docs/design) under WSL's systemd, the Tailscale/discovery path across machines (docs/design/agent-host/daemon.md discovery order, docs/context/nuc-host.md), the self-upgrade build, `bridle sign`, paths under /Volumes, tmux/session code. List each with: works as is / needs a change (say what) / verify on the PC.
2. Which work moves: only builds (remote builds), whole projects (the move steps in docs/context/nuc-host.md "Moving a project to another machine"), or bridle's own repo too. Recommend one, with why. Take into account that the human's dalek sessions must reach the PC's daemon (br-8b98 [human.<machine>] token fallback is related).
3. A short list of follow-up bridle tasks needed (each one line: goal, files, size); the PM files them.
Out of scope: writing the setup guide (separate task), code changes. Findings from npj2 (build-time research) may change what moves: read its ticket if findings are there.
Acceptance: the "Audit" section exists on the ticket with the three parts; no source changes. Model: Sonnet.
