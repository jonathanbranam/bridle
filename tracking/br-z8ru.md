+++
id = "br-z8ru"
title = "UI work ships without visual inspection: strict workflow and controls for UI tickets"
kind = "re-evaluate"
state = "pending"
created_at = "2026-11-21T17:36:25.104Z"
updated_at = "2026-11-21T17:36:41.468772Z"
created_by = "external:aide@vesta"
watchers = ["external:aide@vesta"]
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
