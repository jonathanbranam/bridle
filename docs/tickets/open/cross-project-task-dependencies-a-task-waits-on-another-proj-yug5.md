---
id: yug5
title: "Cross-project task dependencies: a task waits on another project's task, and its watchers hear when that one lands"
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [ztss, xxxq, ckvz, cy2v, yghs, 2mtr]
tasks: []
---

## The ask

The human, verbatim (2026-10-04 ~8:50 PM ET, via the aide), on re-running track-web's tw-sxfh
after bridle's researcher role (2mtr) lands:

> This also points out, I think, something that's missing, or maybe it's not missing: how would
> that get rerun? How would that work with the schedule? We need the ability to notify an agent, or
> we're sequencing work between projects here, right? That's actually functionality that we
> definitely need, so write up a ticket about this. This is a good use case. I've seen this 3 or 4
> times, particularly between bridle and bridle UI, where we have a ticket or a task (probably
> should be a task) between two different projects that are dependent.
>
> I think we have dependencies modeled already, so this is just a between-project dependency that
> needs to be modeled. One task on one project can be dependent on a task on another project, and
> then this needs to be messaged, right? When the parent task is shifted, when it lands, then a
> message needs to be sent to some agent in that project to say, "Hey, the task you depend on is
> now done. We can continue with this."
>
> I guess the other thing is that the way that should work is there's some sort of event or status
> change on the task, and that way all watchers get a notification. That sounds like the right thing.

## What's wanted

- A task in one project can depend on a task in another (`bridle task dep` today makes edges only
  within one daemon).
- When the task depended on changes state (lands), that's an event on the dependent task (its
  state changes, for example it becomes startable), and every watcher of the dependent task is
  notified through the normal watcher path (xxxq). No one has to remember to send a message.

## Cases

- tw-sxfh (track-web) re-run waits on bridle's researcher role (2mtr).
- bridle-ui tasks waiting on bridle API changes (the human has seen this "3 or 4 times").
- harness waiting on track-web's engine (yghs).

## Related

This is option 2 of [[scheduling-work-across-projects-that-depend-on-each-other-ztss|ztss]]
(cross-project `blocks` edges; the waiting daemon polls the other's task, and the edge may need a
condition: integrated or released), now asked for by the human. Watchers that span daemons:
[[one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v|cy2v]]; split tasks inheriting
watchers across projects: [[tasks-split-from-another-inherit-its-watchers-across-project-ckvz|ckvz]].
