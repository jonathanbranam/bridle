---
id: rrqe
title: "A phone app that prompts the human: noise, reminders, voice-to-text replies"
kind: explore
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask


An idea from the human (2026-10-03), relayed by the NUC orchestrator from the notes advisor. A raw
idea: it needs refinement and a specification with the human before any task (k7tm).

The human, quoted:

> somehow we need to build a mobile app I can run on my phone because I need it to make noise, I
> need it to yell at me, I need to be prompted. How that's going to work with our backend system
> is a pretty open question, but I'd like to use what we're building with Bridle and these daemons
> and the agents that wake up and I can interact with to have a system that can prompt and remind
> me and ask what's going on.

Voice is a must:

> I want solid support for voice to text. I use a lot of voice for these messages. And I do pay
> for Whisperflow [Wispr Flow]... whatever our solution is, should integrate as cleanly as
> possible with that... I only want one subscription for voice to text

They'd switch from Wispr Flow if something else works better around iOS's limits on it.

Related, in meta-notes (filed, not scheduled): personal-and-work-planning-differences-zyab and
daily-exercise-block-in-planning-u34c. The workout reminder is the first prompt they named.

Related here: the gateway and bridle-ui (`docs/design/human-web-ui.md`), the human's surface
(k4wq), and email (rs7p).

Context-aware nudges (the human, 2026-10-03, relayed by the NUC orchestrator), GTD-style: "I
could be prompted or nudged to check on, depending on where I am and what's going on that day,
so it would require my schedule." Location is tracked in their notes now, by the phone app later.
Examples: laundry started 1-2 hours ago, check the dryer; home on a Saturday, take the recycling
to the center; a periodic, unscheduled kitchen compost reminder. meta-notes holds the data side
(its ticket context-aware-nudges-5zm3); the waking and prompting is bridle's and the app's
(scheduled messages: hrcn).
