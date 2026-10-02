# The human web UI (design, not built)

Ticket: [[docs/tickets/open/a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy|essy]].
Nothing here is built. First version: **local, read plus check-off and answer, nothing else.**
The TUI stays; this sits beside it.

What exists already (checked at cacab9c): every project daemon serves `/v1/*` on its own port.
`GET /v1/tasks?claimed_by=human` lists the human's to-dos (`--for-human` creates them,
`TaskQuery`, ex9q); `GET /v1/messages?to=human&unread=true` lists what's waiting for the human,
questions included (kind `question`); `POST /v1/tasks/{id}/done`, `/drop`, `/answer` and
`/v1/messages` (with `reply_to`) act. Loopback `GET`s need no token; writes need the human token.
So the UI needs **no new daemon endpoints** for v1.

## 1. What the human sees

One page, "Needs me", grouped by project, then two lists per project: **Decisions** first
(they block agents), then **To-dos**. Each list is in priority order (high first, then oldest);
a decision outranks a to-do of equal priority.

```
bridle                                   nuc ● dalek ●        [refresh ⟳ 5s]
─ bridle ─────────────────────────────────────────────────────────────
 DECISIONS (2)
  ? rs7p  Should the settle period apply to tickets too?      from manager-2 · 3h
          [ answer…                                      ] [Send]
  ? br-91 Land with the failing flaky test or wait?           from orchestrator · 20m
          [ answer…                                      ] [Send]
 TO-DOS (3)
  ☐ high   [at restart] Token migration (t6kq)                      2d   ▸
  ☐ normal Move the SSH key to the Keychain                         4d   ▸
  ☐ low    Try meta-notes mn-fbc0                                   9d   ▸
─ meta-notes (nuc) ───────────────────────────────────────────────────
 DECISIONS (0)   TO-DOS (1)
  ☐ normal Review the data-contracts trial                          1d   ▸
```

One item shows: id, title, priority, age, who asked, and (on `▸`) the body and the thread.
Nothing else: no agents, no events (that's the TUI's job).

| Action | API |
|---|---|
| Check off a to-do | `POST /v1/tasks/{id}/done` |
| Decline a to-do (rescinding stays the asker's) | `POST /v1/tasks/{id}/drop` with a one-line reason (a prompt) |
| Answer a task's open question | `POST /v1/tasks/{id}/answer` |
| Answer a message question | `POST /v1/messages` `{to: <asker>, reply_to: <id>, body}`; the reply marks the question read |

A checked item greys out in place with an Undo for ~10 s (the page just delays the POST), so
running down the list is quick and a slip is recoverable. **Phone-friendly: yes**, by being plain:
one column, large tap targets, no hover. The page is responsive CSS, no separate mobile design.
(Reaching it from a phone is section 4's problem, not the layout's.)
Alternative: a flat cross-project list with a project badge. Rejected: the human asked for
"organized by project".

## 2. Across projects and machines

**Discovery** is what `bridle --project` already does: `~/.bridle/daemons/<project>.json` for
local daemons, `[projects]` (with `machine`, `port`) plus `[machines]` in `~/.bridle/config.toml`
for remote ones (k7mw; `bridle_api::machines`). The UI reuses that code, re-reading on each
refresh, so a new project appears with no UI config.

**Fan-out** is server-side: the UI process queries each daemon concurrently (short timeout,
2 s) for the two lists above and merges them. A daemon that's down shows as a greyed "unreachable"
project header, not an error page. Refresh is polling every ~5 s (v1); SSE is a later swap, since
`/v1/events/stream` exists but means one open stream per daemon.
Alternative: the browser calls each daemon directly. Rejected: CORS, a token in the page, and
remote ports the browser may not reach.

**Needs from 3ehu:** the UI acts with the human's token, which today is read only from the
daemon's own machine's workspace. For a remote project it needs `[principal.<machine>]` from
`credentials.toml`, which the CLI already uses for `--project`; 3ehu fixes the human CLI's lookup
for other machines. Until then the UI is **read-only for remote projects** (off-machine reads need a token too, so
even those wait on a credentials entry). Local projects work without 3ehu.

## 3. Where the server runs

**Recommendation: `bridle ui`, a separate process** (a subcommand of the `bridle` binary, like
`tui`), not part of any daemon. Why: it's the one thing that spans daemons, it must not care
which project daemon is up, and a daemon restart (routine) must not take the UI with it.
Default `127.0.0.1:7390`, `--listen` to change it. Start and stop: `bridle ui` in the foreground,
`bridle ui --detach` for a background run with a pid file, and a launchd/systemd unit later
beside the daemons' (`bridle daemon launchd install` shape). It holds no state, so a restart
loses nothing and a browser tab just reconnects on its next refresh.

Alternatives: **served by one daemon** (couples the UI to that project and its restarts; a project
with no daemon up loses the UI); **per daemon** (N URLs, which is exactly what the human said
they don't want).

## 4. Browser auth

- **v1: loopback only.** The server binds 127.0.0.1. Reads follow the daemons' own rule
  (loopback `GET`s need no token). Writes: the UI process, not the browser, holds the human
  token (read the way the CLI does: workspace token file, then `credentials.toml`) and attaches
  it server-side. The browser never sees a bridle token.
- **Guarding the writes**, since any local process or web page can hit a localhost port: every
  write is a `POST` carrying a random per-process secret (in a `<meta>` tag the server renders
  and a custom header the JS sends), and the server checks `Origin`/`Host` are the loopback ones.
  That blocks cross-site forgery and DNS-rebinding without a login screen.
- **Later, remote and phone (u6wk):** the human reaches it over Tailscale (bind the Tailscale
  address like the daemon does, never 0.0.0.0). That is non-loopback, so it then needs a real
  login: a UI token, supplied once as `?token=` in a URL, swapped for an `HttpOnly`, `SameSite=Strict`
  cookie. Public HTTPS (Funnel or a proxy with a cert) is a further step; skip it until a need.
  The UI token is the UI's own, **not** the human's bridle token.
- **What a browser can never do:** hold or see the human's bridle token; talk to a daemon
  directly; create, spawn, stop or remove agents, change budget, or mint tokens. The UI's server
  exposes only the four actions in section 1 and refuses everything else, so a compromised page
  or token can answer and check off, not run arbitrary work.

## 5. Stack

**Rust in the workspace** (a new `crates/bridle-ui`, axum, which the daemon already uses, and
`bridle_api::Client` for the fan-out), **server-rendered HTML** (plain string templates or
`askama`), **~50 lines of vanilla JS** for the undo timer, the POSTs and the poll refresh, one CSS
file embedded in the binary. No build step, no node, no framework: the page is two lists and
a form, and it ships in the same `cargo build` as everything else. Alternative: a SPA
(React/Svelte). Justified only if the UI grows interactive views far beyond lists; revisit
then, not now. Alternative: htmx. Reasonable and would replace the hand-written JS, but adds a
vendored file for ~50 lines saved; take it if the JS grows.

## 6. Build tasks, in order

1. `bridle ui` skeleton: axum on loopback, fan-out over discovered daemons, one read-only
   "Needs me" page (decisions and to-dos by project). Tests with the fake daemon fixtures.
2. Check-off and answer: the four POSTs, the CSRF guard, the undo delay. Local projects.
3. Remote projects: credentials per machine; depends on 3ehu.
4. `--detach`, pid file, launchd/systemd unit.
5. Later, only if wanted: Tailscale bind plus the UI token and cookie (u6wk); SSE refresh.

**Prototype first, only if the human wants to choose a layout before task 1** (the prototyper
role, `workflow/base/roles/prototyper.md`, builds from a prompt's constraints only): (a) the
section 1 screen as static HTML with fake data, in the grouped layout and the flat one, to
compare how fast a run-through feels on a laptop and a phone; (b) nothing else. Auth, fan-out
and stack are not worth a prototype: they are decided by the constraints above.

## 7. Migration

None. No project file, config or schema changes: discovery reads the existing registry and
`~/.bridle/config.toml`, the UI stores nothing, and no daemon endpoint is added. The one
dependency that touches existing setups is 3ehu, which is its own ticket.

## Open questions for the human

- Is "decline" (drop with a reason) right for a to-do you won't do, or should it stay with the
  asker only?
- Does a to-do's "[at restart]" tag in the title get its own badge, or stay as text?
