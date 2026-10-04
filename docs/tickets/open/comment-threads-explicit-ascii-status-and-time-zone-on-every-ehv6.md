---
id: ehv6
title: "Comment threads: explicit ASCII status and time zone on every entry, thread IDs, resolve, 'human via <agent>'"
kind: feature
opened: 2026-10-04
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [x8jt, jrm2, wjhp]
tasks: [br-ehv6]
---

## The ask

The human, verbatim (2026-10-04, via advisor doc-review), after the advisor explained how a
thread's state is read from the text (the newest entry is the human's and has no `sent` mark):

> Let's make it explicit about what state each comment is in and include a timestamp for the
> most recent status.
>
> I think, for simplicity, if I make a comment with no stamp (because I'm maybe editing by hand
> and I don't feel like typing it), then that should be picked up for review. If that's picked up
> for review, it'll probably go directly to sent.
>
> If we're doing this through the user interface, then it should be clear that you should add a
> timestamp for pending or something like that, with the date timestamp in the same format. The
> agent's reply should be timestamped as well. In the future, we can add read and resolved, or at
> least read by the human.
>
> I don't know if this exists, but I should be able to resolve a thread from the interface. Now,
> what I'd really like to see is a resolved note in the document before it's deleted, so it's an
> indication that that was resolved by a human in the file.
>
> The flow would probably be for the reviewer and the UI, or probably the resolve command and the
> UI. I'm not sure, but if I resolve it, or my last comment indicates that this is resolved, then
> if I say "thank you" or "resolve this," it would then be tagged as resolved by so-and-so, and
> then include the timestamp.
>
> As a rule, active agents that I talk to can always act on my behalf. Perhaps we need a standard
> format for this, but it should either read:
>
> * Closed by or updated by advisor, role authorized by human
> * Replied from the human and then via orchestrator agents, whatever
>
> It's kind of a policy, I think, that would be nice for traceability. Either way, recommend which
> is preferred. Probably, it seems like "replied by human via the advisor" would be better, but
> that's just my authority, and we're just chasing what came from. Make your recommendation.
>
> Something that just reminds me of is when I do email forwarding, I have the ability to set up a
> Gmail ability to respond from different accounts. It includes some sort of similar response, I
> think, in the message.

Then, on the advisor's proposal (with the caveats below):

> As a general rule, avoid any Unicode symbols in any sort of plain text place where I might be
> typing or editing. I think youre using a Unicode dot there. I can't type that in Vim, at least
> not easily, and I don't want to, so just ASCII.
>
> This is a place where I think we should just add a time zone. I hope everywhere we're writing
> things to the system, we're using UTC, but I don't like UTC. It's extremely distracting, so I'd
> like to add a time zone to the end of this: "What's missing?"
>
> I forget the exact formatting options here, but I want to see something really short, like a
> two- or three-letter abbreviation. I just say Z for CCC, and it is possible. I'm just going to
> be working in the United States, so likely it's pretty much Eastern, Central, or Western.
> Pacific. 95% of the time, it's going to be Eastern, so let's not overdo it on this context or
> the requirements.
>
> Your example says, "resolved by human via." Oh, I see. It was resolved by me via the review
> agent, so that's fine. [...] I guess what I'm debating is whether we should have a role and the
> region ID there. That's my question, because I don't know if those are always going to be the
> same. They may change if the role is given a change of what the role was going to happen.
>
> For the CLI, how do I indicate a thread? I don't see an ID on the thread in the example.
>
> With those caveats, I approve this. I don't see a necessity for read by the agent. I don't know
> if you do, but agents in general are pretty fast at accidenting, so I'm perfectly happy to have
> read by the agent. However, I guess explain what that would hit in the workflow: when the review
> happens, when would the agent mark it as read versus sent?

(Speech-to-text. "region ID" is likely "agent ID", "accidenting" likely "acknowledging".)

## What's there now (advisor, checked 2026-10-04)

- Daemon (`crates/bridle-daemon/src/doc_watch.rs`): a thread is pending when its newest entry
  (the header, or the last `> **who, time:**` reply) is the human's and has no `<U+00B7> sent
  YYYY-MM-DD HH:MM` mark. That mark uses U+00B7 (a Unicode middle dot), in Eastern time with no
  zone. An entry counts as the human's if its author name doesn't contain "agent" (`is_human`),
  so a reply signed `advisor` counts as the human's.
- UI (`bridle-ui` `src/doc/comments.ts`): reads the same sent mark. An agent's `@human` tag is
  marked read by appending `(read)`. Agent replies carry a time only (`**docs agent, 14:06:**`).
- No thread IDs, no resolve.

## Decided (the human approved the advisor's proposal with these caveats, 2026-10-04)

### The format: ASCII only, a status and zone on every entry

Every entry (the first comment or a reply) has `who, YYYY-MM-DD HH:MM ZZZ` and ends its first
line with one status in square brackets: `[<state> YYYY-MM-DD HH:MM ZZZ]`. Only the latest
status is kept, and git keeps the history. Each thread has a short ID.

```markdown
> [!comment] c3 human, 2026-10-04 10:57 EDT, on "nothing records" [sent 2026-10-04 11:00 EDT]
> Why not? Wouldn't an audit want it?
>
> **doc-3haz, 2026-10-04 11:02 EDT:** The task's history already names the human. Rewrote it. [read 2026-10-04 11:15 EDT]
>
> **human, 2026-10-04 11:16 EDT:** thanks [read 2026-10-04 11:16 EDT]
>
> **resolved by human via doc-3haz, 2026-10-04 11:17 EDT**
```

- **ASCII only.** No U+00B7 middle dot, no curly quotes or arrows in anything the human may type or edit.
  The status sits in `[...]`.
- **Zone:** the US abbreviation for the writer's local time, DST included: `EST`/`EDT`,
  `CST`/`CDT`, `MST`/`MDT`, `PST`/`PDT`. In practice it's Eastern (`human-timezone`). This is a
  deliberate exception to that rule, which says to record times in UTC and to leave "ET" off
  Eastern times. These stamps are text the human reads and types, not records. Nothing in the
  logic parses them: state comes from which marks are present, never from comparing times.
- **Thread ID:** `c<n>` right after `[!comment]`, unique within the document. The next ID is the
  highest in the file plus one. The UI writes it with a new thread. For a hand-typed thread, the
  daemon adds it when it marks the thread `sent`. The human never has to type one.

### States

| Entry | Marks, in order |
|---|---|
| The human's | *(none)* or `[pending ...]` -> `[sent ...]` -> `[read ...]` (by the agent) |
| An agent's | *(none)* -> `[read ...]` (by the human: opened in the UI) |
| Closing line | `**resolved by <who>, <when>**` |

- **Pending rule (unchanged in spirit):** a thread needs the agent when its newest entry is the
  human's and has no mark, or `[pending]`. A hand-typed comment with no mark is pending and goes
  straight to `sent`.
- **The UI writes `[pending <now>]`** when it saves a new comment or reply, so the human can see
  it's queued.
- **`sent` vs `read` by the agent:** the daemon writes `[sent]` when it delivers the batch to the
  agent's inbox. It changes the mark to `[read]` when that message is marked read, which happens
  when the agent lists or wakes on it (bridle already records this). Usually that takes seconds.
  A thread stuck at `sent` means the agent is busy, down or out of budget, which is the point of
  showing it.
- **Who is the human:** an entry is the human's only if its author is `human` or
  `human via <agent>`. That fixes the `is_human` "anything without 'agent'" rule.

### Resolve

- **UI button and CLI:** `bridle review resolve <path> c3` (and the UI button) appends
  `**resolved by human, <when>**` to the thread.
- **By saying so:** when the human's newest entry asks to close it ("thanks", "resolve this"),
  the document's agent appends `**resolved by human via <agent>, <when>**`.
- **The agent's own call** (e.g. it rewrote the passage as asked): `**resolved by <agent>, <when>**`.
- **Resolved threads stay in the file**, collapsed in the UI. A cleanup deletes them later, and
  git keeps them. The cleanup is a command or a sweep after a turn or two; its shape is the
  planner's choice and stays small.

### Attribution policy: "human via <agent>" (advisor's recommendation, approved)

- **`human via <agent>`**: the agent is carrying the human's words or request (a relay, a
  "resolve this", a "thanks"). The human's authority makes the action legitimate, so the human
  is the author and the agent is the route. This matches the "From the human, via advisor:"
  relay convention, and Gmail's From/Sender pair ("sent by X on behalf of Y").
- **`<agent>` alone**: the agent acting on its own judgement.
- **`<agent>` is the agent's name, not its role**, e.g. `doc-3haz`, `advisor`,
  `advisor (doc-review)`, `orchestrator`. Names identify one agent you can look up
  (`bridle agents`, its logs), while a role can be changed or reassigned. Add the role only
  if a name doesn't make it clear.
- Record it as a rule in `workflow/base/rules/` beside the relay convention.

### General rule: ASCII in human-editable text

Record as a rule in `workflow/base/rules/`. Anything the human may type or edit (tickets, docs,
comment threads, config) uses ASCII: no U+00B7 middle dot, curly quotes, arrows or em dashes that a Vim
user can't easily type.

## Scope

- Daemon: the parser and the marks (pending, sent, read-by-agent, thread IDs, `is_human`), the
  resolve command, plus migrating the existing U+00B7 `sent` marks in documents under review.
- Gateway: whatever route the UI's resolve and read need.
- bridle-ui: write `[pending]`, IDs and `[read]`; a Resolve button; resolved threads collapsed.
- Docs: the x8jt format section and `docs/design/human-web-ui.md`. The two rules.
