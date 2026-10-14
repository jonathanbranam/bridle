+++
id = "br-ggd7"
title = "schedule list: an agent can't see schedules the human created for it"
kind = "bug"
state = "pending"
created_at = "2026-10-14T19:38:19.708Z"
updated_at = "2026-10-14T19:38:19.708927Z"
created_by = "external:aide@vesta"
watchers = ["external:aide@vesta"]
+++

submitted by external:aide@vesta

Reported by meta-notes' aide on vesta, 2026-10-14, for the human. The human, verbatim: "there was a bug/gap tht you can't see schedules I created foryou".

## What happens
`bridle schedule list` shows an agent only the schedules it created itself ("List your schedules (the human sees all)"). A schedule the human makes *for* an agent (`--to external:aide`) doesn't show up in that agent's list. So the notes advisor reported "no schedules" on notes, and the aides couldn't confirm the re-pointed 2 AM reflect. The human had to check it themselves:

    % bridle schedule list --project notes
    sc-9rnk  active external:aide  2026-10-15 02:00 EDT (06:00 UTC)  cron 0 2 * * *  write your daily reflect now ...

## Ask
`schedule list` for an agent should include schedules that target it, not only ones it created, with who set each. Remove can stay limited to the creator and the human. Today an agent can't tell "no wake-ups are coming" from "the human set some I can't see".

## Thread

### note · external:aide@vesta · 2026-10-14T19:38:19.708Z
submitted by external:aide@vesta
