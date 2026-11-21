+++
id = "br-r8td"
title = "Per-project startup context: notes aide gets meta-notes prime output injected, no agent choice"
kind = "feature"
state = "pending"
created_at = "2026-11-21T19:00:23.447Z"
updated_at = "2026-11-21T19:00:23.447976Z"
created_by = "external:aide@vesta"
watchers = ["external:aide@vesta"]
+++

submitted by external:aide@vesta

Filed by aide (notes) for the human.

## What happened

The notes aide's startup tells it to run `meta-notes prime` and follow it (notes `CLAUDE.md`, `.bridle/roles/aide.md`). On 2026-11-21 the aide ran `meta-notes prime 2>&1 | head -80`, chained with other startup commands, before ever seeing the output: a reflex to cap unseen output. It missed the conventions (task editing, ✅ dates), then misreported correct meta-notes behavior as a bug (orchestrator m-0398, withdrawn m-0402) and hand-edited a task line. Another agent has done the same truncation before.

## The ask

For the notes project's aide only, the `meta-notes prime` output (which includes the conventions) is put into the session's context at startup automatically, in full, with no step the agent runs or can shorten. For example: a per-project setting naming a command whose output `bridle prime aide` (or the session start) appends, or a SessionStart injection. Not a PreToolUse hook.

## The human's words (2026-11-21 ~2:00 PM ET, verbatim)

"The pre-tool use hook is radical overkill for this problem. This just needs to be enforced without the agent having any control over this. I wasn't realizing that this is not happening in the startup for the aide. This is specifically for the aide at notes, not for any other agent at all. In your instructions, or somehow when you start, that should just be inserted automatically, no matter what, at startup, or just executed automatically, or I'm not sure. The agent should not have any agency over this, but a pre-tool use hook is complete overkill. I don't think that's the right approach at all. Bridle should have some way of just pasting that in for you or forcing you to run it, or something."

## Thread

### note · external:aide@vesta · 2026-11-21T19:00:23.447Z
submitted by external:aide@vesta
