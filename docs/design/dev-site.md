# The throwaway dev site

> **Status (checked 2026-10-06):** Planned: nothing here is built. Ticket
> [[docs/tickets/open/agents-can-run-a-throwaway-bridle-site-on-a-free-port-log-in-vwqt|vwqt]] holds
> the human's ask; the build tickets are listed in section 6. Modelled on track-web's
> `docs/dev-second-instance.md` (read only; track-web keeps its own recipe).

Agents building or reviewing the human web UI ([[docs/design/human-web-ui|gateway and UI]]) must
click through it before calling a task done. The human's gateway and daemons are live and
hold the human's token and real projects, so an agent needs a **second site**: its own
gateway, its own login, its own fake projects, its own ports, deleted when done.

## 1. The command

`bridle dev site up | down | list`, a new `dev` subcommand of the `bridle` binary.

`up` (flags: `--name N`, `--ui DIR`, `--no-ui`, `--json`):

1. Makes `.agent-site/<name>/` in the current directory (default name: the `BRIDLE_AGENT_NAME`,
   else a random word). `.agent-site/` is gitignored. Everything below lives under it.
2. **Own bridle home.** `<dir>/home/` is used as `BRIDLE_HOME` for every process the command
   starts, so nothing reads or writes `~/.bridle` (this env var is already honoured by
   `bridle-api` discovery and the gateway's config load). `home/config.toml` holds a `[gateway]`
   with `bind = "127.0.0.1:<free>"`, `username = "agent"` and the argon2 hash of a random
   password (the same hash `bridle gateway hash-password` makes), and a `[projects]` entry per
   fixture project. No `[machines]`: nothing leaves the laptop.
3. **Fixture projects.** `<dir>/projects/<name>/` are real git repos (a daemon needs one),
   initialised and committed by the command from a tree shipped in the bridle repo
   (`crates/bridle/fixtures/dev-site/`, embedded in the binary so an installed `bridle` works
   outside the bridle repo). Each has `docs/tickets/open` and `resolved` (every kind, with
   `see`/`needs` links, stem links and bare-ID links, a ticket with a question), `docs/design`,
   specs, and plain documents (markdown with wiki links, one broken link). Two projects, so the
   project switcher has something to switch. No `origin`, and `[state] push = false`.
4. **Daemon data.** One real `bridle serve --repo <fixture> --listen 127.0.0.1:0` per fixture
   project, with `BRIDLE_HOME` as above, so `/items`, `/tasks` and `/system` are served by the
   real code. After health, the command seeds through the CLI/API: tasks in several states, one
   claimed by `human` with an open question (the to-do list), one done, a message, and a usage
   figure. It spawns **no agents and never runs `claude`**; `/system` shows an empty roster.
   A fake-claude agent for a roster is deferred until a UI task needs one.
5. **Gateway.** `bridle gateway` as a child with the same `BRIDLE_HOME`; waits for
   `/api/v1/health`.
6. **UI dev server**, unless `--no-ui`: runs the checkout given by `--ui DIR` (default: `[dev_site]
   ui_dir` in the project config, else `../bridle-ui` beside the clone) as
   `node node_modules/vite/bin/vite.js` with `VITE_DEV_PORT=<free>` and
   `VITE_API_TARGET=http://127.0.0.1:<gateway port>`. `--no-ui` is for gateway-only tasks; the
   agent then uses the API with `curl`.
7. Writes `<dir>/site.json` (pids with each process's start time and argv, the three ports, the
   URL, the login, the fixture manifest), then prints the URL and login and exits 0. `--json`
   prints `site.json`. Logs are `<dir>/{daemon-*,gateway,ui}.log`.

**Free ports.** The command binds port 0 and reads the port back (the gateway and daemon take
`:0` already); only Vite needs a probe-then-release, and it runs with `strictPort` so a lost
race fails loudly rather than moving to another port. Never a fixed default, so any number of
agents run at once. The probe never uses the registry's `[ports] range` and never takes a
developer's usual dev port.

**Stopping (rule `no-kill-by-name`).** `down [name]` reads `site.json` and sends SIGTERM to
the recorded pids only, after checking each pid's start time and argv still match what was
recorded (a reused pid is left alone, and said so), then deletes the directory. `down --all`
does the same for every site under this directory's `.agent-site/`. `list` shows names, ports
and whether each pid is alive. There is no pid or name search anywhere. The children are
**not** detached into their own session: they stay in the agent's process tree, so bridle's
cleanup on agent stop reaps a site the agent forgot. If `up` fails midway it runs `down` on
what it started.

**Rejected alternatives**

- *A `just` recipe or script* (track-web's three commands). Fine for one project with one
  server; here it needs five processes, config generation, hashing, seeding and a pid-checked
  teardown, and shell is the wrong place for it (rule `shell-zsh`, no-kill-by-name). It would
  also have to be copied to every project; a subcommand ships with the binary.
- *Pointing a UI at the human's gateway with a test login.* Writes to the live config and shows
  real projects; the whole reason for a second site.
- *`bridle port alloc` for ports.* It asks a daemon, and the site's own daemons are what
  we are starting. Binding `:0` needs nothing.
- *Fixtures in bridle-ui only.* The data must match what the gateway and daemon produce, so it
  changes with them; it lives in bridle and the UI checks against it (section 2).
- *A shared long-lived test site.* One more thing to keep alive and clean up; the site costs
  seconds to start, so each agent gets its own.

## 2. The split between bridle and bridle-ui

**bridle** owns: the `dev site` command, fixtures, config generation, seeding, and
`site.json`'s fixture manifest (project names, ticket and task ids, counts, the login).

**bridle-ui** owns the dev-server side, as its own ticket (text in section 6, ticket
[[docs/tickets/open/bridle-ui-dev-server-takes-a-port-and-api-target-and-checks-dgef|dgef]]):

- `vite.config.ts` reads `VITE_DEV_PORT` (with `strictPort: true`) and `VITE_API_TARGET`,
  defaulting to today's values, and proxies `/api` there. Without the second variable a client
  on another port would still proxy to the human's gateway; that is what track-web found.
- `npm run check:site -- <site.json>` (or a URL and login): logs in against the running site
  and asserts the manifest's facts through the page or `/api/v1` (the tickets list shows the
  fixture counts, a stem link resolves, the to-do list holds the question). It fails when the
  UI and the fixture have drifted, so the fixtures are tested from both sides. It needs no
  real browser; the browser steps are section 3.
- The UI's own README gets the recipe (`bridle dev site up --ui .`).

Version coupling uses what exists: the UI already checks the API version the gateway reports;
the fixture is shipped with the same bridle binary as the gateway, so it can't be older than it.

## 3. How agents verify in a browser

- **Tool: `playwright-cli`** (installed here; track-web's CLAUDE.md uses it): `open <url>`, log
  in with the printed login, click, `snapshot` to read the page, `screenshot --filename=` to
  `/tmp/<task-id>-verify/`, never the repo. A project with another browser tool names it in
  its CLAUDE.md. If the tool is missing, rule `missing-tools` applies: ask, don't skip.
- **Who:** the worker verifies before reporting done, and says in the done message what it
  clicked and what it saw (one line per acceptance item). The manager checks that line at
  review; the aide may re-drive the site. A "done" for a UI task without it is not done.
- **Teardown:** `bridle dev site down` before reporting. The done check includes `list` empty.
- **Rule text** (`workflow/base/rules/verify-ui-in-a-browser.md`, severity `must`, since it only
  applies to tasks whose acceptance is something the user sees):

  > A task that changes what a user sees in a browser is not done until you have used it in a
  > browser. Start a throwaway site (own port, own data, own login: never the human's running
  > instance, never their database), log in, click through each acceptance item, and say in your
  > done message what you did and saw. Stop what you started, by pid. For a bridle project:
  > `bridle dev site up`; for another, the project's recipe (CLAUDE.md, "Verification").
  > Why: the human, 2026-10-05: "While building a UI, the agents need to be able to do this
  > as well to verify that tasks are actually working."

- **Role-doc text.** `worker.md` ("How you work"): "For a UI task, verify it in a browser on a
  throwaway site (rule `verify-ui-in-a-browser`) and put what you clicked and saw in your done
  message." `manager.md` (review): "A UI task's done message states what was clicked and seen; if
  not, send it back." No new role.

## 4. Carrying it to every project with a web UI

Two layers, one contract.

- **The contract (the rule, in `workflow/base`):** a second instance with its own port, own
  data, own login, in a gitignored directory; prints URL and login; stops only its own pids;
  safe for many agents at once. It does not say how; the project's CLAUDE.md "Verification"
  section does.
- **bridle-family projects** (gateway, bridle-ui) use `bridle dev site`. **track-web** keeps
  `docs/dev-second-instance.md`, which already meets the contract: it is the project's recipe,
  the rule points at it, and bridle changes nothing there (rule `existing-projects`). Another
  web project writes its own recipe, or asks for a pack later. No pack now: a pack (a
  distributable bundle of rule, role text and recipe) is only worth building when a third
  project has a recipe worth sharing (yagni).
- **No `[dev_site]` config key for the contract.** The one key, `ui_dir`, is bridle's own
  need (finding the UI checkout) and is added with the command.

**Migration (ticket xebc direction: automatic, opt-out).** What each existing project gets:

| Project | Gets | How |
|---|---|---|
| Every bridle project | the rule and role text | `bridle workflow update` then `bridle sync`: no project file changes, as with any rule |
| Every project | `.agent-site/` in `.gitignore` | a migration (next id in `crates/bridle/src/migrate.rs`), idempotent, appends the line if absent |
| bridle itself | `bridle dev site` and fixtures | ships in the binary |
| bridle-ui | the dev-server config and `check:site` | its ticket (section 6); its CLAUDE.md "Verification" names `bridle dev site up --ui .` |
| track-web | the rule only | its CLAUDE.md already has "Verification" naming its recipe; nothing else changed |

The gitignore migration is the only file change a project receives. [[docs/design/migrations|migrations]]
says "never automatic" today while xebc's later direction is automatic with opt-out; this design
needs only that the migration is safe to run either way, and follows whichever xebc settles.

## 5. Not decided here (not yet, with reasons)

- A fake-claude agent in the fixture roster: wait for a UI task that needs a populated roster.
- Remote actions and machines in the fixture (gateway task 9 is itself unbuilt).
- A pack: see section 4.
- Authenticated screenshots in CI: no CI browser job exists; revisit when one does.

## 6. Build tickets

1. [[docs/tickets/open/a-bridle-dev-site-command-starts-a-throwaway-daemon-gateway-vdu8|vdu8]]
   (repos: bridle): section 1 and the bridle half of 2. Verify: `bridle dev site up` twice at
   once gives two sites on different ports; `down` of one leaves the other; the gateway answers
   `/api/v1/projects` with the fixtures after login; `~/.bridle` unchanged; a pid-reuse test for
   `down`.
2. [[docs/tickets/open/bridle-ui-dev-server-takes-a-port-and-api-target-and-checks-dgef|dgef]]
   (repos: bridle-ui): the bridle-ui half of section 2.
3. [[docs/tickets/open/a-rule-and-role-text-make-agents-verify-ui-work-in-a-browser-tgxy|tgxy]]
   (repos: bridle): section 3's rule and role text, the gitignore migration of section 4, and a
   CLAUDE.md "Verification" note for bridle. Verify: `bridle workflow update` in a scratch
   project renders the rule; `bridle migrate --dry-run` reports the gitignore line.
