---
id: y496
title: "TUI: optional panels, and seeing the work in progress and coming up"
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [yurx, 8ups, fgu6, j479]
---

## The ask

Backburner ideas, for after things are working better. The human, verbatim (2026-09-28):

> I'm interested in discussing some options about the Bridal Tui.
> - The top section: I really like seeing each of the agents and workers, and what they're
>   doing and what's going on. I think it's interesting. There could maybe be a little more
>   interactivity or updates happening there. These are all backburner ticket ideas for
>   things to work on another time.
> - The Events tab or the Events section is also kind of interesting, but not very
>   important to me. It kind of gives me a sense that things are happening, and I think
>   mostly I'm interested in the agent. I might just hide the whole thing. I'm kind of
>   thinking about some enhancements to this that would show maybe optional panels in
>   different places that we could turn on and off. I'm thinking I might not watch the
>   events most of the time.
> - The logs, I guess, are per agent. I'm not quite sure what's different about the logs
>   versus everything else.
>
> I think, commonly, what I'm interested in seeing, in addition to the agents, is seeing
> work get done and work coming up. I know a lot of that is in the queue, but I'd like to
> see, at a high level, what the agent is doing.
> - If a worker agent is working on a specific ticket, I would want to see the name of that
>   ticket and ideally be able to hit Enter or something and view the ticket details, or
>   possibly open the ticket. In Vim, I can read-only maybe somehow just see the ticket
>   details.
> - See what's coming up: I see a little chart of the tier 1 and tier 2 tasks, the ones
>   that would be coming up to work on next.
> - If the manager is working on a merge, what they're doing with the merge, kind of things
>   like that would be pretty interesting to see.
>
> These are all just ideas to write down somewhere and come back to sometime in the future
> after we get things working a little bit better.

## Notes

- Today's four views (`docs/design/cli.md`, `tui`): agents list, event tail (every
  daemon event, all agents), the selected agent's transcript tail ("Logs", the same as
  `bridle logs --follow`), and the inbox.
- The agents-panel scroll bug from the same conversation is filed separately:
  [[tui-agents-panel-doesnt-scroll-yurx|yurx]].
- "Coming up" would read the queue from [[the-task-queue-in-bridle-task-not-messages-j479|j479]]
  (`bridle queue`).
