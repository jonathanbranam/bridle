---
id: 9dcz
title: Agents are reminded once to link the ticket and task IDs they show the human (a hook, never a block)
kind: feature
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [jmng, yfjc, 22ab]
tasks: []
---

## The ask

The human, 2026-10-10 ~1:00 PM ET, verbatim (to advisor product-manager):

> I don't know where this fits in terms of themes or epics, but a small quality-of-life thing is
> that agents continuously forget to link tickets and tasks. Someone suggested a hook that would
> ask the agent to add those links somewhere. I want to hear about that. I have some concerns,
> so there might be a ticket.
>
> Please look. I have a few concerns about that because there may be things in a message that
> are not linkable or that are not links. The word "easy" is, I think, a valid ticket name but is
> not a link. I think there are a lot of four-letter English words that actually are potential
> ticket names. That's one concern.
>
> The other concern is: what if a four-letter something or other is just not a ticket? It's
> something else. It just happens to match that pattern. I don't want that to block a message
> from going through.
>
> I don't know what we can do in terms of those hooks, but the idea is that if it scans a message
> and there's a missed link, it says, "Hey, you're supposed to highlight it as a link like this."
> What I want is that anywhere a ticket ID or task ID shows up (we're going to combine those), it
> should be a link. It's fine to use the short name. That's great. That's a link. The text words
> that describe the ticket can also be a link. That's just sort of optional.
>
> You give me IDs a lot, and it's fantastic. I want to be able to click them, but I don't want a
> constant refusal. I'm afraid we're going to design this and find that there's a constant hook
> firing and refusing things, like on the word "then" or "rest" or "this." ... A lot of these
> are valid ticket names, and that's not going to work. We can't do that.
>
> In this latest message, you put the ticket names in parentheses, so that would be an
> indicator, but the agent may not do that. I don't want the hook to take forever. If it's
> possible for it to very quickly check if those are valid tickets, then it should, I think, in
> one round trip, somehow, if this is possible, say, "You need to add links around this," and
> then don't deny it again if the agent refuses. If the agent refuses again, it probably has a
> good reason.
>
> I'm thinking that that hook will just fire a few times, and then the agent will start
> remembering to do it. It won't happen again until the agent's context fills up ..., and then
> it forgets to do it entirely for a long time. That's what I'm hoping. This is not high
> priority, but I'd like that. I'd like to investigate that and see how that works.
>
> The other thing is, I just go recheck the role prompt descriptions. Maybe the advisor role
> wasn't updated with that rule. It should be a bridle rule that applies to every role. It
> shouldn't have to be copied into every role, but I feel like I'm not even sure that stuff
> works properly.

Then, ~1:15 PM ET:

> Yeah, let's go ahead and file it as a new ticket. It shouldn't preempt other work, but it is
> something that's a continual problem, so it'd be nice to get it in today.
>
> Also, with the unification of tickets and tasks, it will potentially be easier because every
> ticket is going to have an internal record in the tasks table, which will be renamed, I think,
> the tickets table. That's not a file scan. That is a bridle command. We should have this be
> really, really easy.
>
> I assume the hooks run a CLI command or something. I don't really know how hooks work. I'd
> actually be really interested in understanding this, so more about how hooks work. Go and
> write that up as a ticket, and I want a design on it. I think we have a design agent. ...
> Have it write up a good design on that so I can read over and understand how it would work.

(Transcribed speech: "mist link" read as "missed link", "one rat" as "one round trip".)

## Facts (PdM, 2026-10-10)

- Rule `link-ids-for-the-human` (`workflow/base/rules/link-ids-for-the-human.md`) exists, with
  `roles: [orchestrator, advisor, aide, manager]`. It does reach the advisor: it is in
  `bridle prime advisor`'s output. The advisor still forgot it in a status update the same day.
  Rules are opted in per role; there is no "every role" scope.
- [[every-task-and-ticket-id-the-human-sees-is-a-clickable-link-jmng|jmng]] (2026-10-05) asked
  the same, listed where links could be added (UI rendering, `bridle send`, a Stop hook), and
  was cancelled by the human before any design ("I don't see anything there that's worth
  implementing so far"). This ticket is narrower: a one-shot reminder hook.
- Ticket IDs use the alphabet `abcdefghjkmnpqrstuvwxyz23456789` (no i, l, o, 0, 1), so "this"
  can never be an ID; "then", "rest", "easy" can. About 430 tickets exist out of ~920k possible
  IDs; none of the human's example words is a ticket today. Task IDs (`br-xxxx`) are
  unambiguous.
- `bridle link <id>` prints a link, or nothing when no `[gateway] public_url` is set.
- Everything-is-a-ticket ([[everything-is-a-ticket-one-record-per-piece-of-work-a-ticket-22ab|22ab]])
  will give every ticket a row in the database, so "does this ID exist" becomes a daemon query.

## The design ask (for the designer)

Write `## Design` into this ticket, for the human to read and understand:

1. **How Claude Code hooks work**, plainly: the events (Stop, PreToolUse, UserPromptSubmit, ...),
   that a hook runs a command and reads JSON on stdin, how it can block or send the agent back
   with a reason, the `stop_hook_active` flag that marks a second pass, timeouts, and how bridle
   installs hooks today (`bridle kill-guard`, `bridle focus gate`; the roles' settings). Cite
   the Claude Code docs.
2. **Where the human reads IDs**: a terminal session's replies (advisor, aide, orchestrator),
   `bridle send` to the human or aide, task comments, the bridle UI. Which of these a hook can
   cover, and which are better linked at render time.
3. **The check**: which tokens count (task IDs always; a bare four-character token only if that
   ticket exists), what counts as already linked, how fast it must be, and how it asks bridle
   (a new batch command, e.g. `bridle link --check <ids...>`, now; the 22ab table later).
4. **One reminder, never a loop or a block**: the agent is sent back once with the list and
   their links; a second pass is always let through; any error or timeout lets the reply
   through.
5. **The rule for every role**: how a base rule applies to all roles without listing them
   (a `roles: all` scope, or another way), and whether the link rule is the first user.
6. Options with trade-offs and a recommendation; open questions for the human.

Build nothing. The build is a later task, after the human reviews the design.
