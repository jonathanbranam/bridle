# The human web UI: the bridle gateway (design, not built)

Ticket: [[docs/tickets/open/a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy|essy]]
("Option F" and "Gateway v1: the human's answers", decided by the human, 2026-10-02). Nothing
here is built. This replaces the earlier option C (a Rust-rendered page, `bridle ui`).

The shape: a **`bridle gateway`** process in bridle's workspace serves one versioned API; a
separate **TypeScript UI** (`bridle-ui`, a later project) is a client of it. The browser never
holds a bridle token and never talks to a daemon.

## 1. The gateway

- **A subcommand of the `bridle` binary**, its own per-machine process (launchd or systemd, like
  the daemons), not inside any daemon. Daemons are per project and restart often; the gateway
  spans projects and outlives those restarts. It must never be on a daemon's start-up path, and
  a daemon needs no change to be reachable by it.
- **Discovery and fan-out:** it finds every defined daemon the way `bridle --project` does
  (`~/.bridle/daemons/<project>.json`, `[projects]` and `[machines]` in `~/.bridle/config.toml`,
  `bridle_api::machines`), queries them concurrently with a short timeout, and merges the
  results. A daemon or project that doesn't answer is **reported as unreachable** in the
  response (a sleeping laptop's projects, say), not as an error.
- **Tokens:** it holds the human's token per machine and acts with it. Nothing records that an
  action came through the gateway. Gateways don't talk to each other; one gateway reaches
  whatever its machine's config and credentials allow.
- **Contract with the daemons:** it uses `bridle-api` (as the CLI does), so a daemon API change
  breaks the gateway's build in `just check`. The daemons' `/v1` API stays internal.
- **No new daemon endpoints** for v1: to-dos are `GET /v1/tasks?claimed_by=human`, task
  questions are on the tasks, and acting is `POST /v1/tasks/{id}/done`, `/drop`, `/answer`.

## 2. v1 scope

The human's **to-dos and task questions** (decisions), grouped by project, decisions first,
each list high priority then oldest. Actions: check off a to-do, answer a task question, decline
a to-do with a reason. **Retracted items are hidden** from the lists; the withdrawal and its
reason stay on the record as the audit trail (the asker withdraws, "To-do withdrawn by ...").
Out for now: questions sent as messages to `human` (they can't be retracted) and the inbox; no
agents, events or any agent control. The gateway exposes only the v1 actions and refuses the
rest, so a stolen session can answer and check off, not run work.

## 3. API, login, UI

- **One versioned API** (`/api/v1/...`), at most one previous version served beside it.
  TypeScript types are **generated from the Rust types by ts-rs** and committed or emitted by a
  `just` target for `bridle-ui` to consume. (utoipa plus openapi-typescript only if a full
  endpoint spec becomes worth it.)
- **Login:** username and password; the `argon2` hash lives in the machine config. A successful
  login sets an `HttpOnly`, `SameSite=Strict` session cookie. Reached over Tailscale (bind the
  Tailscale or loopback address, never 0.0.0.0); `tailscale serve` can add HTTPS.
- **The UI** is its own TypeScript repo, `bridle-ui`, beside `bridle/`. Its build output is
  installed into a folder the gateway serves, **`~/.bridle/ui/`**, so page and API share an
  origin. The build records the API version it targets; the gateway warns or refuses on a
  mismatch (`ui_version_mismatch`, default warn). The UI files are served without a session, since
  the page must load to show its login form; every API route stays guarded. Not compiled into the `bridle` binary (no Node in bridle's build). In development
  the UI's dev server proxies API calls to the gateway.

## 4. Multi-machine

The human runs one gateway on one machine and manages everything from there. That needs the
human's token for other machines' daemons: **br-8b98** (3ehu part 1, `[human.<machine>]`
fallback), built but unlanded until the human's Saturday review. The multi-machine build task
comes after it. A project on a random port needs a `[projects]` entry with a fixed port to be
reachable from another machine. Until then the gateway works for local projects.

## 5. Build tasks, in order

Each is one branch and one worker. None touches the daemon.

1. **Skeleton, config, serve**: `bridle gateway` subcommand, a `crates/bridle-gateway` library
   (axum), the optional gateway config section, bind address, `GET /api/v1/health`. Tests: the
   server starts, answers, and rejects a bad bind.
2. **Discovery and fan-out**: reuse registry, `[projects]`, `[machines]`; concurrent queries
   with a timeout; unreachable daemons reported. Tests with fake daemons: one up, one down, one
   slow.
3. **Listing across projects**: to-dos and task questions, grouped and ordered per section 2.
   Tests: ordering, grouping, two projects merged.
4. **Actions**: check off, decline, answer, with the human's token (local machine). Tests: each
   action reaches the right daemon; a missing token is a clear error.
5. **Retract handling**: hide withdrawn items, keep them on the record. Tests: a withdrawn
   to-do and a withdrawn question don't list; the audit trail is still on the daemon's record.
6. **Login and session**: argon2 config, login and logout, cookie, every other route behind it.
   Tests: no cookie is refused, a wrong password is refused, a cookie of the right flags.
7. **Versioned API and ts-rs types**: move routes under `/api/v1`, the types derive `TS`, a
   `just` target emits them; a test that fails if the emitted files are stale.
8. **Static UI folder and version check**: serve `~/.bridle/ui/`, compare the build's recorded
   API version, warn or refuse. Tests: serves a file, a missing folder, a mismatched version.
9. **Multi-machine** (after br-8b98 lands): per-machine human tokens for remote daemons, remote
   actions. Tests: a remote fake daemon with its own token.
10. **Service install** (launchd/systemd unit, `bridle gateway` beside the daemons' install).

`bridle-ui` itself (and an install script) is a later project, not part of these tasks.

## 6. Migration

No project file, schema or daemon change. The only addition is new **optional machine config**
(the gateway section with the login and bind address, in `~/.bridle/config.toml`); a machine
that doesn't run a gateway never sees it.
