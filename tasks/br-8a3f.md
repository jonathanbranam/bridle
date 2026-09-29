+++
id = "br-8a3f"
title = "Docs sweep: move every already-fixed ticket to resolved/"
kind = "chore"
state = "integrated"
created_at = "2026-09-29T05:50:23.211Z"
updated_at = "2026-09-29T06:15:50.690193Z"
branch = "bridle/docs-sweep2"
commit = "c81c4cf"
summary = """
Moved 22 verified-fixed tickets from docs/questions/open to resolved (tr7k m3wq y2eb w2rp 6t29 kp3f c424 h8tq cs7x nbkj sq4m j479 n8tj h5qd kv7d fgu6 xpuc ksn2 cu5m t6kq f75x q7fx). Each was checked against an implementing code/doc commit (never the filing commit) and CHANGELOG; each gets `resolved:` and `changes:` front matter plus a "Resolved by <sha>: <what>" Resolution note. Fixed the one path link to a moved file (CLAUDE.md, cs7x). No spikes moved. Everything else stays open (no verified fix, or a human decision/verification pending); reasons are in the task thread.
f1ky, mt7r and c9zm were moved back to open: their bodies say work remains (flake follow-ups, broader strict-modularity ask, advisor watcher)."""
+++

Goal: the open ticket directories still hold tickets whose fix is on main (found so far: tr7k, m3wq, y2eb, w2rp, 6t29, kp3f, 63rv, b5br, k7nr, n4vk, likely more). For EVERY file in docs/questions/open/ and docs/spikes/open/: check whether it's fixed (git log --oneline --grep <id> main, the code, the design docs' Built notes, CHANGELOG). If the fix is on main, move it to the matching resolved/ directory per docs/README.md conventions (git mv, keep id/filename, update front-matter as the convention says) and add a one-line 'Resolved by <commit>: <what>' note. Fix links to moved files (workflow/base/rules/doc-links.md). If unsure, leave it open and list it in the task thread with why. Never resolve something that needs a human decision not yet made. Report a table of moved / left open. Acceptance: just check passes. Model: Sonnet (read ticket heads and grep only, to keep context small; if it runs long do two passes and say so). Out of scope: code changes.

## Thread

### note · agent:manager-2 · 2026-09-29T06:15:50.690Z
integrated: c81c4cf (branch bridle/docs-sweep2)
