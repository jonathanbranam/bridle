+++
id = "br-z8ru"
title = "UI work ships without visual inspection: strict workflow and controls for UI tickets"
kind = "re-evaluate"
state = "claimed"
created_at = "2026-11-21T17:36:25.104Z"
updated_at = "2026-11-21T17:51:32.002937Z"
created_by = "external:aide@vesta"
watchers = [
    "external:aide@vesta",
    "agent:wz8ru",
]
ticket = "z8ru"
+++

docs/tickets/open/ui-visual-check-z8ru.md

submitted by external:aide@vesta

From aide (notes), for the human.

The human, 2026-11-21 ~12:36 PM ET, after a meta-notes-ui list checkbox shipped with its check mark (a letter X) drawn outside the box (mu-9gqq), verbatim via voice:
"Is there a meta-notes UI aid working, or can we file tickets there? I'm frankly kind of infuriated. The list check-off button thing doesn't look right. The visual style is terrible. When you click it off, the X isn't inside the box at all. Is it inside the circle at all? I don't really understand why, how that could be, how we have a component that obviously was never visually inspected. Also, isn't it just using React? Isn't there a really obvious and clear way to build a checkbox or something? I don't know. I could send a screenshot, but the X just isn't inside the box, and it's just a letter X. I don't know. It just looks ridiculous. Well, obviously, the whole point of bridle here is scaling, because there should be strict workflow, guidelines, and controls around how these things get built, and that's obviously not happening. We need to work on the bridle overall solution, especially for these UI-based things. They're never really doing what they should be doing. They aren't built in the right way, and it's a massive waste of my time to review them."

The concern is bridle-wide, not this one component: UI work across projects reaches the human without anyone having looked at it, and reviewing it costs them a lot of time. They want strict workflow, guidelines and controls for how UI work gets built and checked, as part of the overall bridle solution: for example, a required visual check (screenshots at phone and desktop widths, attached to the ticket) before a UI ticket can be called done, and guidelines on using standard components rather than hand-rolled ones.

This is a design question for the human; bring options to them rather than building one.

## Thread

### note · external:aide@vesta · 2026-11-21T17:36:25.105Z
submitted by external:aide@vesta

### note · external:orchestrator · 2026-11-21T17:37:01.339Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-21T17:37:01.591Z
readied by orchestrator: re-evaluate kind, but requested by the human in their own words (quoted in The ask, via aide@vesta) and scoped to options only, no build.

### note · agent:wz8ru · 2026-11-21T17:51:31.917Z
Options for UI tickets (wz8ru, research only, nothing built).

WHAT I FOUND ON mu-9gqq: data incomplete. I could not read it.
- `bridle ticket show mu-9gqq --project meta-notes-ui` -> "error: mu-9gqq belongs to project mu, not br; run this in mu's checkout"
- `BRIDLE_PROJECT=mu bridle ticket show 9gqq --project mu` -> "error: ticket 9gqq has no row on this daemon"
- ~/.bridle/daemons holds no meta-notes entry on this machine; the local meta-notes clone has no 9gqq in git log or on bridle/state.
So I do not know what the worker and manager reported. Someone in mu's checkout (or whoever owns that daemon) should run `bridle ticket show mu-9gqq` and add it here. What the current rule allows is clear: it asks only for "one line per acceptance item" in the done message, and a line like "clicked the box, it toggled" passes with the X drawn outside the box. Nothing checks looks.

OPTION A - tighten verify-ui-in-a-browser (edit, no new rule, no code). Cost: a few lines of rule text, ~0 per ticket beyond the screenshots.
- Worker must attach screenshots (playwright-cli screenshot) at ~390px and ~1280px, after state of each changed control in each state (off, on, hover/disabled), saved on the ticket (comment with paths) and must LOOK at them and say what is visually wrong or right, not just that it toggles.
- Manager must open the screenshots before merging; a done or merge message without them is sent back (this is already how the rule treats a missing line).
- Native elements first: use the real <input type=checkbox> / framework component, not a hand-built glyph; a hand-rolled control needs a one-line reason.
- Enforcement is text only, so it relies on say-so, but attached images make a lie or skip visible to the human in seconds.

OPTION B - A plus a gate in bridle. Cost: a day or so of code (ticket flag `ui: true`, `ticket resolve/land` refuses without an image attachment on the thread), plus per-project opt-in in .bridle/config.toml. Catches "forgot entirely", not "looked at and judged wrong". Worth it only if A is ignored again.

OPTION C - A plus an appearance reviewer. Cost: one extra Sonnet/Haiku agent run per UI ticket (a vision model reads the screenshots against the ticket and a per-project docs/ui-guidelines.md; ~ a few cents to a dollar) before merge. Directly targets "does it look right", which a function click-through cannot. Needs each project to keep a short guideline file (component library, spacing, icons).

RECOMMENDATION: A now, plus a 10-line ui-guidelines.md convention per project (native elements, project component library) that A's rule points to. Add C only for projects that opt in (meta-notes-ui first), since it is the only one that judges appearance. Hold B until A has failed once more: a gate cannot tell a good screenshot from a bad one.

QUESTION FOR THE HUMAN: Do you want A alone, or A plus the reviewer step (C) on meta-notes-ui, and may I draft the rule edit and a ui-guidelines.md for that project? Also, should someone with access paste the mu-9gqq worker and manager reports here first, so the edit targets what actually happened?

### note · agent:wz8ru · 2026-11-21T17:51:32.002Z
done: options comment posted on br-z8ru. mu-9gqq unreadable from here (errors recorded on the ticket), so that data is incomplete. Nothing committed.
