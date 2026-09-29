# Incident notices

Design for ticket [[incident-notices-that-retract-themselves-nc7r|nc7r]]. **Not built.** An
incident is a record, not a message: opened with a short body, shown to every agent in the
audience while it's open, and closed when it's over. An agent that never saw it never hears of
it. Almost all of the mechanism is the existing message queue
([[messages]]): a notice is a `system` note that is withdrawn if it's still `pending` or `held`.

An agent that already read a notice can't be made to unread it (stdin can't be unsent), so
"retract" means two things: undelivered notices are **dropped**, delivered ones get a short
"resolved".

## 1. The record

One table, in the same SQLite store (`crates/bridle-daemon/src/store.rs`, a new schema
version after V14; storage.md gets the table):

```
incidents(seq INTEGER PK AUTOINCREMENT, id UNIQUE,   -- i-0007 from seq, like m-0042
          title,                                     -- one line, shown in status and the resolved note
          body,                                      -- what agents are told (a few lines)
          audience,                                  -- 'all' | 'role:<name>'
          state,                                     -- 'open' | 'closed'
          opened_by, opened_at, updated_at, closed_by, closed_at,
          resolution)                                -- optional closing line, sent with "resolved"
```

and one nullable column, `messages.incident_id`, linking a notice to its incident.

- **States: `open` and `closed`.** "Updated" is not a state: an update rewrites `body` and
  bumps `updated_at` (event `incident.updated`). Closed is final; a recurrence is a new
  incident.
- **Not on the state branch.** Like messages, incidents are runtime and needn't survive a lost
  database ([[docs/proposal/decisions|decision 2]]). A daemon restart keeps them (they're in
  SQLite) and its recovery already returns written/held messages to `pending`.
- Retention: closed incidents are pruned with the events, at 30 days.

## 2. Audience

`all` (every agent, any role) or `role:<name>` (the same fan-out target `send` already has,
[[messages]]). Nothing else in v1:

- **External principals** (advisor, orchestrator) are the one gap. They have an inbox, not a
  process, so there's nothing to deliver to or retract from, and they read `bridle status`
  when they choose. Rather than a second delivery path, `bridle incident list` and
  `bridle status` show open incidents to every principal, and that is what they get. The
  orchestrator, who opens most incidents, doesn't need to be told.
- **Cross-project: not in v1.** Each daemon has its own store and agents. The orchestrator
  opens the same incident on each affected daemon with `bridle --project <p> incident open`
  (it already holds one credential per project, principals.md). A cross-daemon incident would
  need a shared id, shared close and a story for an unreachable daemon; the cost of not doing
  it is three commands instead of one.

## 3. Who may open, update and close

- **The human and the orchestrator** (`external:orchestrator`), and **`manager`** agents for
  their own project. Workers and other roles are refused (403), same as lifecycle endpoints
  ([[principals]]). Any of these may update or close any incident: there's one project,
  and the person who noticed it's over shouldn't need the opener.
- **The daemon opens none in v1.** The candidates are a budget hold, a failed push and a red
  `main` (c8qw's CI watcher). The API supports `opened_by = system` so a later task can add
  them, each as a small hook: open on the state change, close on the reverse. **Budget holds
  stay as they are**: their wind-down and resume notices carry per-agent instructions
  (finish your turn, you'll be resumed), which a broadcast body can't. Revisit only if a
  hold ever needs to be shown to the agents that aren't paused.

## 4. Delivery

A notice is a message: `from_principal = system`, `kind = note`, `when = idle`, `incident_id`
set, body `Incident <id>: <title>\n<body>`. It goes through the existing send path
(`Supervisor::send` in `supervisor.rs`), so held/pending, budget holds and acks all apply
unchanged. `when idle` because a notice mustn't interrupt a tool call; a short delay is fine.

- **Open** fans one notice out to each live-or-resumable agent in the audience (running or
  not: a stopped agent's notice sits `pending`, like any message to it).
- **An agent that starts while it's open** gets a notice row created at spawn (after its first
  message; it becomes the first message if there is none). `resume` and `renew` need
  nothing extra beyond one check: they write every `pending` message, and an agent that has
  no notice row for an open incident in its audience gets one first. A notice is **not** put
  in the system prompt: `render_system_prompt` is shared and cache-stable
  ([[agents]]), and a prompt can't be retracted.
- **Update** rewrites the body of any notice still `pending` or `held` (the agent then sees only
  the latest), and sends a fresh `Incident <id> updated: …` note to agents whose notice is
  already written or delivered.
- **Close**, per notice:
  - `pending` or `held` → `dropped` (event `message.dropped`, as `--drop-held` does). The agent
    never hears of it. This is the advisor case.
  - `written`, `delivered` or `read` → one `Incident <id> resolved: <resolution or title>`
    note, `when idle`. An agent with several notices for the incident (open, then updates)
    gets one resolved note.
  - A resolved note itself is not tracked: it has no `incident_id`, so it can't be dropped.
    If its agent is stopped, it stays `pending` and is delivered on resume; accepted
    (an agent that saw the incident should hear it ended, however late).
- **Reused:** `send`, `pending`/`held`, `dropped`, acks, resume's flush of pending messages,
  the budget hold on idle agents. **New:** the `incident_id` link, the drop-on-close and the
  fill-on-start check. There is no coalescing (`main moved` needs it; incidents are rare).

## 5. Surfaces

CLI ([[docs/design/cli]]), all `--json`-capable:

```
bridle incident open "<title>" [--body <text>|--file <path>] [--role <name>]   # prints the id
bridle incident update <id> [--title ..] [--body ..]
bridle incident close <id> [--resolution "<line>"]
bridle incident list [--all]          # open ones; --all adds closed
bridle incident show <id>             # the record, plus who has seen it (delivered / pending / dropped)
```

- **`bridle status`** gets an `incidents` list of the open ones (id, title, audience, age).
  Every principal reads it, external ones included.
- **API** (api.md): `POST /v1/incidents`, `GET /v1/incidents[?state=]`,
  `GET /v1/incidents/{id}`, `PATCH /v1/incidents/{id}` (title/body), `POST
  /v1/incidents/{id}/close`. Wire types in `bridle-api/src/types.rs`.
- **Events:** `incident.opened` · `incident.updated` · `incident.closed`
  (`{incident, title}`), actor the caller. The orchestrator's watcher can wake on
  `incident.opened`; that's a filter change in the watcher, not daemon work.

## Build split

Not decided yet: the orchestrator will frame incidents and the human's to-do list
([[docs/questions/open/incident-notices-that-retract-themselves-nc7r|nc7r]], ex9q) as one
pattern first, then the split follows.
