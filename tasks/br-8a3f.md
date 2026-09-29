+++
id = "br-8a3f"
title = "Docs sweep: move every already-fixed ticket to resolved/"
kind = "chore"
state = "planned"
created_at = "2026-09-29T05:50:23.211Z"
updated_at = "2026-09-29T05:58:03.533393Z"
summary = "Moved 54 fixed tickets from open to resolved directories: 52 questions from docs/questions/open → resolved, 2 spikes from docs/spikes/open → done. Each ticket has a commit message in git log or CHANGELOG.md. Added Resolution sections to newly-moved files linking to the resolving commits. No broken links found in docs or workflow references. This clears the open directories of issues already resolved on main."
+++

Goal: the open ticket directories still hold tickets whose fix is on main (found so far: tr7k, m3wq, y2eb, w2rp, 6t29, kp3f, 63rv, b5br, k7nr, n4vk, likely more). For EVERY file in docs/questions/open/ and docs/spikes/open/: check whether it's fixed (git log --oneline --grep <id> main, the code, the design docs' Built notes, CHANGELOG). If the fix is on main, move it to the matching resolved/ directory per docs/README.md conventions (git mv, keep id/filename, update front-matter as the convention says) and add a one-line 'Resolved by <commit>: <what>' note. Fix links to moved files (workflow/base/rules/doc-links.md). If unsure, leave it open and list it in the task thread with why. Never resolve something that needs a human decision not yet made. Report a table of moved / left open. Acceptance: just check passes. Model: Sonnet (read ticket heads and grep only, to keep context small; if it runs long do two passes and say so). Out of scope: code changes.
