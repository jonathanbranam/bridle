+++
id = "br-9s8u"
title = "Background wakes carry the session's prompt context: quiet hours, and every hook that should apply to them"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T02:51:03.960Z"
updated_at = "2026-10-18T06:14:57.147912Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "external:advisor/product-manager",
    "agent:w9s8u",
]
branch = "bridle/w9s8u"
commit = "392b22a76c23683ae68cb21a00d169522e0d6bb9"
summary = "P1 only: focus::wake_text (over focus::gate, plain text) printed by bridle agent wake and orchestrator wait-for-wake before the wake reasons, for message/task change/timeout; with --json it goes to stderr. Nothing extra outside quiet hours. Unit test for in/out of quiet hours; cli.md and CHANGELOG updated. Caveat: shares the gate's per-session nudge throttle (5 min), so a wake right after a prompt may print nothing."
ticket = "9s8u"
+++

docs/tickets/open/background-wakes-carry-the-session-s-prompt-context-quiet-ho-9s8u.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:38.266Z
watching the task

### note · external:aide · 2026-10-18T02:14:05.910Z
From the human, via aide (2026-10-17 ~10:15 PM ET), replying to my list that recommended "yes" on ui-nvea and "P1" on br-9s8u: "Agree. Fix later". Read as: ui-nvea YES (add Bash(npm run install-ui) to the bridle-ui manager's allowed_tools, that permission only), br-9s8u yes to P1 only (the wake command prints the focus gate text before the wake reasons). "Fix later" = the human's hands-on items (ui-sxs3 dropdown check, xccp, 4a4v) wait.

### note · external:orchestrator · 2026-10-18T02:14:23.881Z
Approved by the human, via aide (m-0180, 2026-10-17 ~10:15 PM ET): "Agree. Fix later", replying to aide's list recommending P1 here. Scope: P1 only (the wake command prints the focus gate text before the wake reasons). Readied by orchestrator.

### note · external:advisor/product-manager · 2026-10-18T02:14:29.929Z
advisor product-manager: readied, scope P1 ONLY (the human via aide, 2026-10-17 ~10:15 PM ET: 'Agree.'). Build: bridle agent wake and bridle orchestrator wait-for-wake print focus::gate's text (same function as bridle focus gate) before the wake reasons; nothing extra outside quiet hours. Out of scope: P2-P5. Planner: write the brief as a proposal (problem, approach, scope). Normal priority, after the Stages MVP children.

### note · external:aide · 2026-10-18T02:20:14.760Z
Confirmed by the human, via aide (2026-10-17 ~10:20 PM ET), asked whether "Agree. Fix later" approved ui-nvea and br-9s8u P1: "Those are good".

### note · agent:w9s8u · 2026-10-18T05:21:46.002Z
built: 36e67c8d0067ab39b0c1af6476608386cf5c48c6 (branch bridle/w9s8u)

### note · agent:w9s8u · 2026-10-18T05:21:47.735Z
done: wakes print the focus gate text (P1 only) before the reasons; just check exit 0, 1858 tests, on 36e67c8d (main merged in)

### note · agent:manager-2 · 2026-10-18T06:14:57.147Z
integrated: 392b22a76c23683ae68cb21a00d169522e0d6bb9 (branch bridle/w9s8u)
