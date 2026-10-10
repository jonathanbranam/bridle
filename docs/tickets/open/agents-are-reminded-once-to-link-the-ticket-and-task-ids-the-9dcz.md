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
tasks: [br-9dcz, br-g8pe]
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

## Design

Designer dg8pe, 2026-10-10. Focus: interface (what the agent is told, which command runs) with a
little architecture (where the check lives). Nothing is built.

In one sentence: a small check that runs when an agent finishes a reply, spots task IDs and
real ticket IDs that are not links, and sends the agent back exactly once to add them.

### 1. How Claude Code hooks work

A **hook** is a command that Claude Code runs for you at a fixed moment in an agent's work. You
list hooks in the agent's settings (a JSON file or the `--settings` argument). Each entry says
"at this event, run this command". The program does not need to be clever or long-lived: it is
started, given the facts, answers, and exits. Official reference: the Claude Code hooks guide
(https://docs.claude.com/en/docs/claude-code/hooks; I did not re-fetch it for this design). What
I state about the Stop hook below was checked against the real `claude` in
`docs/spikes/05-stop-hook-findings.md`.

**The events** (the moments), the ones that matter here:

| Event | When it runs | Can it stop something? |
|---|---|---|
| `UserPromptSubmit` | the human's message arrived, before the agent sees it | yes, it can refuse the message |
| `PreToolUse` | the agent is about to run a tool (Bash, Edit, ...) | yes, it can refuse that one call |
| `PostToolUse` | a tool just ran | it can add a note for the agent |
| `Stop` | the agent finished its reply and is about to go idle | yes, it can send the agent back to work |
| `SessionStart` | a session begins or is cleared | no, it observes |

**What a hook receives and answers.** Claude Code starts the command and writes one JSON object
to its standard input: who and where (`session_id`, `cwd`), the event name, and event facts. For
`Stop`, the facts include `last_assistant_message` (the text the agent was about to stop on) and
`stop_hook_active` (explained below). The command answers in one of three ways:

- print nothing, exit 0: "carry on";
- print `{"decision":"block","reason":"..."}` and exit 0 (or write the reason to stderr and exit
  2): "do not stop; here is why". For `Stop`, the reason reaches the agent as an ordinary user
  message ("Stop hook feedback: ...") and the agent takes another turn. It cannot tell it from
  something a person typed, so the reason should read as a plain instruction;
- crash, or exit with another code: Claude Code treats it as a failed hook and carries on. A
  hook that breaks therefore does not stop the agent.

**`stop_hook_active`.** When a Stop hook has already sent the agent back and the agent finishes
again, the next Stop event arrives with `stop_hook_active: true`. A well-behaved hook answers
"carry on" whenever it sees it. That is the whole mechanism for "remind once": the first pass
blocks, the second pass is let through. Claude Code adds its own safety net: after 9 blocks in
one turn it overrides the hook (spike 05), but that is nine paid model turns, so we do not rely
on it.

**Timeouts.** Each hook entry may carry a `timeout` in seconds; past it, Claude Code gives up on
the hook and carries on. Bridle sets 1800 s for `stop-check` (it may run the project's checks);
this one should be a few seconds.

**How bridle installs hooks today.** Three ways, all of which end as the `hooks` object of the
agent's `--settings`:

- `bridle kill-guard` and `bridle arch-guard` are `PreToolUse` entries listed in
  `workflow/base/hooks/PreToolUse.json`. Bridle merges each workflow layer's `hooks/<event>.json`
  into every spawned agent (`layer_hooks` in `crates/bridle-claude/src/command.rs`) and into
  `bridle session` (`with_layer_hooks` in `crates/bridle/src/session.rs`). `kill-guard`
  (`crates/bridle/src/kill_guard.rs`) is a pure function: read the Bash command, say refuse or
  not, print the reason.
- `bridle focus gate` (`UserPromptSubmit`) and `bridle focus reply` (`Stop`) are written into
  the settings text of the orchestrator, advisor and aide sessions (`FOCUS_GATE`, `REPLY_HOOK`
  in `session.rs`). Those three roles share the advisor's settings string.
- `bridle stop-check` (`Stop`) is added for the worker role only (`stop_check` in `command.rs`).
  It is the closest cousin of what we want: it reads `stop_hook_active`, allows at once when
  set, allows on any error reaching the daemon, and otherwise blocks with a reason
  (`crates/bridle/src/commands/hook.rs`).

So a link-reminder hook is one more small `bridle <name>` command, in the same shape, plus one
settings entry. Nothing new in how hooks are delivered.

### 2. Where the human reads IDs, and what a hook can cover

| Where the human reads | Can a Stop hook cover it? | Better handled |
|---|---|---|
| Replies in a terminal session (advisor, aide, orchestrator) | **Yes.** `last_assistant_message` is the reply. | Only a hook or the agent's memory can do this; the text is the agent's own. |
| `bridle send` to the human or the aide, task comments, notes | **No, not well.** The text is inside a Bash tool call, not in the final reply. A `PreToolUse` hook on Bash could read it, but it has no "second pass" flag, so "only once" would need state kept between calls. | **At render time**: the bridle UI and the gateway know every real ID and could link them whatever the agent wrote. That needs no agent cooperation. |
| The bridle UI (messages, threads, tickets) | Not applicable. | Same: render-time linking. |
| Managers and workers | Their replies go to other agents, not to the human. Reminding them costs a model turn for nothing. | Leave alone; a manager's `bridle send human` falls under the row above. |

A hook only sees the **last** message of a turn. If the agent wrote IDs in earlier text of the
same turn and ended on "done", those are not seen. That is acceptable for "reminder": the
agent learns the habit from the misses we do catch.

A link is useless when `[gateway] public_url` is not set (`bridle link` prints nothing). The
hook does nothing then.

### 3. The check

Input: the reply text. Output: a list of IDs that are not yet links, each with its link.

**Which tokens count.**

- **Task IDs** (`br-` followed by 4 characters from the ID alphabet): always. They cannot be a
  word by accident.
- **A bare 4-character token**: only if it is made of the ID alphabet **and** a ticket with that
  ID exists. The alphabet is `abcdefghjkmnpqrstuvwxyz23456789`: no `i`, `l`, `o`, `0`, `1`. That
  already rules out "this", "will", "also", "look". Words that survive ("then", "rest", "easy")
  are flagged only if a ticket really has that ID. Today about 430 tickets exist out of roughly
  920,000 possible IDs; none of the human's examples is one. So "easy" in a sentence is never
  flagged unless a ticket named `easy` is filed, and then it is a real ticket.
- **Skipped even if they match**: a token glued to other characters (part of a longer word, or
  of a path or file name such as `docs/tickets/open/foo-9dcz.md`), anything inside a code block,
  and capitalised words (IDs are lower case). Cheap guards against the "it just looks like an
  ID" worry.

**What counts as already linked.** The ID sits inside a Markdown link, either as the text
(`[9dcz](http://...)`, `[the link ticket](...)` next to it) or in the address, or inside a
`[[wiki]]` link, or the reply already contains a URL ending in that ID. The human said the short
name as the link text is fine, and long text as the link is optional, so the test is "does this
occurrence sit inside link syntax", nothing about the wording.

**How it knows a ticket exists.** Two ways, now and later:

- **Now:** list the file names in `docs/tickets/open/` and `docs/tickets/resolved/` (IDs are the
  end of each file name). That is a directory listing of a few hundred names, a few
  milliseconds, no daemon call, and it fails safe (cannot read: no flags). The project is the
  one the session runs in.
- **Later:** once [[everything-is-a-ticket-one-record-per-piece-of-work-a-ticket-22ab|22ab]]
  lands, every ticket has a row in the database, and the same function asks the daemon instead.
  Only that one lookup changes.

**Speed.** Bridle starts, reads stdin, scans the text with a small pattern, lists two folders,
answers. Expected well under 100 ms; the hook gets a 5 s timeout so a stuck disk cannot hold a
reply. There is no model call in the hook itself. The only real cost is the extra model turn
when it does remind (see next section).

**A batch command?** The ticket suggested `bridle link --check <ids...>`. If the hook is itself
a `bridle` command, it can call the same code directly and needs no second process. I would not
add the batch flag for the hook's sake (`yagni`, and one more spelling next to `bridle link`).
`bridle link` taking several IDs, one link per line, is a reasonable small extension if agents
want it, but it is not needed by this design.

### 4. One reminder, never a loop or a block

The flow, using the existing pattern of `stop-check`:

1. The agent finishes a reply. Claude Code runs `bridle link-check` with the Stop JSON.
2. If `stop_hook_active` is true: print nothing, exit 0. (Second pass: always let through. If
   the agent chose not to link, it probably had a reason; that is the human's own wish.)
3. If there is no `public_url`, or no unlinked ID is found, or **anything** goes wrong (bad
   JSON, unreadable folder, a timeout): print nothing, exit 0. The reply goes through.
4. Otherwise print one block, once:

```
{"decision":"block","reason":"Your reply names these without a link: br-ab3k, 9dcz (these IDs are real tickets). Add each as a Markdown link, e.g. [9dcz](http://...). Links: br-ab3k http://..., 9dcz http://.... Ignore any that are not tickets."}
```

The reminder never refuses a message and never keeps the agent from stopping a second time. A
wrongly flagged word costs one extra model turn and a one-line reply, not a stuck agent. The
worst case of the whole feature is therefore one cheap extra turn per reply that missed a link.

### 5. The rule for every role

Finding: this already works without new syntax. `rules_section` in
`crates/bridle-daemon/src/rules.rs` gives a rule to a role when the rule's `roles:` is empty
("tagged for role, or untagged"). So "a rule for every role" is a rule file with **no `roles:`
line**. No `roles: all` is needed, and adding one would be a second way to say the same thing
(`one name per action`). The link rule would be the first base rule to use it: every other base
rule lists roles today.

And the human's suspicion about the advisor: the advisor **was** listed
(`roles: [orchestrator, advisor, aide, manager]`) and `bridle prime advisor` does print the rule.
It was forgotten anyway. Rules are read once at session start, so a long session drifts away
from them. That is the argument for a hook over more rule wording: the hook repeats the rule at
the moment it matters.

Should the link rule go to every role? Only the roles whose words reach the human need it; for
workers it is noise in every prompt. My recommendation is to keep the rule, drop the `roles:`
list entirely so it cannot go stale again when a role is added (the human's "shouldn't have to
be copied into every role"), and accept a few lines of prompt for roles that never use it. The
rule already says "managers: only in messages that reach the human", so the wording copes.

### 6. Options

**Option A: do nothing.** The rule stays, agents keep forgetting. Cost: the human keeps
asking, by hand, in every session. It is a recurring annoyance he named, so "not much" is not
honest here.

**Option B: a Stop-hook reminder in the three interactive sessions (recommended).** New hook
command `bridle link-check` (named like `stop-check`, `kill-guard`), installed next to
`bridle focus reply` in the settings the orchestrator, advisor and aide sessions share
(`session.rs`). Checks as in sections 3 and 4. No daemon call now; the 22ab lookup later.
The rule loses its `roles:` list. Covers the terminal replies, which is where the human sees
the problem. Costs: one small command and a few tests; one extra model turn when it fires; does
not cover `bridle send` / comments / UI; sees only the last message of a turn.

**Option C: B plus a PreToolUse check on `bridle send` and `bridle task comment`.** Covers the
second row of section 2. Falls short: no second-pass flag, so to be "once" it needs a small
state file per session; and the same effect is available for free by linking at render time
(D). Adds complexity for a case with a better answer (`kiss`, `yagni`).

**Option D: link at render time in the bridle UI and gateway.** The UI turns every real ID in a
message or comment into a link, whatever the agent wrote. Deterministic, no model turn, no
rule. It cannot reach a terminal session, so it complements B rather than replaces it. This was
the useful part of the cancelled jmng; it is a separate ticket in another component (the human
UI), not part of this one.

**Option E: a hook that blocks or rewrites.** A reply cannot be rewritten by any Claude Code
hook; blocking every time is the "constant refusal" the human ruled out. Rejected: it breaks
the one-reminder rule and the fail-open rule.

**Option F: a `roles: all` scope word, or a rule copied into each role prompt.** Rejected, see
section 5: the empty `roles:` already means everyone; copies drift.

### Recommendation

**B**, with the rule's `roles:` list removed, and **D** as a separate ticket (render-time links)
if the human wants `bridle send` and comments covered. B is the smallest thing that answers
"agents forget in the terminal", matches an existing hook shape exactly, and cannot get an agent
stuck. It accepts: one extra model turn per miss, no coverage of tool-call text, only the last
message of a turn, and a directory listing until 22ab gives a database lookup.

### Questions for the human

1. Install the hook only for the three sessions you talk to (orchestrator, advisor, aide), as
   recommended, or also for managers (their `bridle send human` is not seen by a Stop hook, so
   probably not)?
2. Is "an ID counts as linked if it is inside any Markdown link or `[[...]]`" enough, or must
   the link point at the bridle UI (the `public_url`) to count?
3. Do you want the reminder to name the ID's title as well, so the agent can see what was
   matched (small extra text), or IDs and links only?
4. Do you want Option D (render-time links in the UI and for `bridle send`) filed as its own
   ticket now?
5. The rule's `roles:` list removed so it reaches every role, including workers: acceptable, or
   keep a list that adds the missing human-facing roles (project-manager, designer)?
