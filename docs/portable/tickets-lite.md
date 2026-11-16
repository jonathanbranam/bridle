# Tickets lite: a small ticket system to adopt elsewhere

Status: a proposal for other tools. **This is not documentation of how bridle works.** It is a
simplified version of bridle's ticket conventions, written for another project (or a small
daemon + CLI) to adopt, adapt or ignore. Where it keeps bridle's behaviour it says so; where it
departs or adds something bridle doesn't have (the roadmap), it says that too. Bridle's real rules
are in [docs/README.md](../README.md) and [workflow/base/rules/tickets.md](../../workflow/base/rules/tickets.md).

## The idea in one paragraph

A ticket is a Markdown file in the repo, one per ticket, with a short YAML front matter block and
a free body. Its name ends in a 4-character ID that never changes. Its folder is its state
(`open/` or `resolved/`). An epic is just a ticket of kind `epic`; a child names it with
`parent:`. A roadmap is one file that lists epics in order. Everything is plain files you can
browse in Obsidian or on GitHub, and a small local daemon does the mechanical writes so agents in
isolated worktrees can still create and update tickets.

## Folders

```
docs/tickets/
  open/        one file per open ticket
  resolved/    one file per closed ticket (moved here on resolve)
docs/roadmap.md   the ordered list of epics (new; bridle has no such file)
```

- **The folder is the state.** Resolving a ticket moves it from `open/` to `resolved/` and stamps
  `resolved: <UTC timestamp>` in its front matter (kept from bridle).
- Add more states only if you need them; every extra folder is another move that breaks
  path-based links (see Links).

## File names

`<slug>-<id>.md`, for example `login-page-drops-the-redirect-k7tm.md`.

- **slug**: from the title at creation: lowercase ASCII letters and digits; every run of other
  characters becomes one `-`; no leading or trailing `-`; cut to 60 characters. The slug is for
  humans browsing the folder; nothing depends on it.
- **id**: the last 4 characters before `.md`. This is the identity. Retitling a ticket doesn't
  rename the file (kept from bridle: names are set once).
- The file name stays bare even if IDs get a project prefix (below).

## Stable IDs: the algorithm

This is the part bridle gets right and is worth copying exactly.

- **Alphabet**: 31 characters, `abcdefghjkmnpqrstuvwxyz23456789`. Lowercase letters and digits
  without `i`, `l`, `o`, `0` and `1`, so an ID can be read aloud, typed from a screenshot or
  written by hand without confusing `l`/`1` or `o`/`0`.
- **Length**: 4 characters. 31^4 = 923,521 possible IDs: plenty for one repo, short enough to say
  in a sentence or a commit message ("fixed k7tm").
- **Minting**:
  1. Collect every ID already used: the 4-character tail of every file name in `open/` *and*
     `resolved/`. Resolved IDs are never reused.
  2. Draw 4 characters at random from the alphabet (any decent random source; bridle uses a
     CSPRNG-seeded draw and ignores the tiny modulo bias).
  3. If the ID is already taken, or is a reserved placeholder, draw again.
  4. Write the file with that ID in its name and in `id:`.
- **Reserved**: `xxxx`, because people write it as a placeholder ("br-xxxx: todo") and a checker
  must never mistake it for a real ticket.
- **Random, not sequential**, on purpose: two people (or two agents, or two branches) can mint
  tickets at the same time without a shared counter, and an ID says nothing about order or age, so
  nobody reads meaning into it. Each mint checks every existing ID, so a clash can only happen
  between two tickets minted at the same time on different branches, and the odds for any such
  pair are 1 in 923,521; the checker catches it and one ticket gets a new ID. If one process (the
  daemon) does all minting from the main checkout, even that goes away.
- **Optional project prefix**: when tickets from several projects get mentioned together, qualify
  the ID: `id: br-k7tm`, and use the qualified form in references. File names stay bare
  (`...-k7tm.md`). Prefixes are short, lowercase and unique per project. Bridle does this; a
  single-project setup can skip it and add it later, because readers can accept both forms.

## Front matter

Keep it small. A minimal set:

```yaml
---
id: k7tm                 # or br-k7tm with a prefix
title: "Login page drops the redirect after SSO"
kind: bug                # see kinds
created: 2026-11-16
parent: q3rp             # optional: the epic this belongs to
blocked_by: []           # optional: ticket IDs that must be resolved first
related: []              # optional: ticket IDs worth reading alongside
resolved: 2026-11-20T14:02:11Z   # set only by resolve
---
```

- IDs in front matter are bare IDs (or prefixed IDs), never links: a tool can resolve them, and
  they don't break when files move.
- Bridle has more fields (`repos`, `changes`, `specs`, `tasks`, `theme`, `created_by`); add them
  only when something reads them.
- Body: free Markdown. Bridle's habit worth keeping: open with `## The ask` (what is wanted, in a
  few lines), and quote the requester's words verbatim rather than paraphrasing them.

## Kinds

A short list, so filters stay useful. From bridle, trimmed:

| kind | for |
|------|-----|
| `feature` | something new to build |
| `bug` | something broken (a report) |
| `chore` | maintenance, ops, cleanup |
| `docs` | documentation only |
| `research` | a question to investigate; its answer goes in a doc, then the ticket resolves |
| `epic` | a large piece of work split into child tickets |

There's no `idea` kind: whether a ticket is scheduled is its state or its place on the roadmap,
not its kind.

## Epics

- **An epic is a ticket** with `kind: epic`. Its body says the goal and what "done" means. It
  lives in the same folders and gets an ID the same way.
- **A child names its epic** with `parent: <epic id>`. One parent per ticket. The epic doesn't
  have to list its children: a tool derives them by scanning for `parent:`, so there's one source
  of truth and no list to keep in sync. (Bridle does this.)
- **Order inside an epic** is `blocked_by:` between children, only where order really matters.
- **An epic resolves** when its children are resolved and its goal is met; resolving it is a
  person's decision, not automatic.
- No nesting beyond one level (epic → tickets). If an epic gets too big, split it into two epics
  and order them on the roadmap.

## The roadmap (new; not in bridle)

One file, `docs/roadmap.md`, that sequences epics. The order of the list is the plan; the file is
edited by hand (or by the CLI) and reviewed like any other change.

```markdown
# Roadmap

## Now
1. [[sso-login-cleanup-q3rp|SSO login cleanup]]
2. [[ticket-daemon-mvp-h7ds|Ticket daemon MVP]]

## Next
3. [[obsidian-dashboards-x2mf|Obsidian dashboards]]

## Later
- [[mobile-triage-view-9pkc|Mobile triage view]]

## Done
- [[repo-layout-for-tickets-m4zt|Repo layout for tickets]] (2026-11-02)
```

- Only epics go on the roadmap; a lone ticket that matters enough becomes an epic of one.
- `Now` / `Next` / `Later` / `Done` are enough. Numbers in Now and Next are the sequence; Later is
  unordered on purpose.
- A checker can verify: every roadmap entry is an epic that exists; every open epic appears on the
  roadmap exactly once; Done entries are resolved.
- If you later want the order machine-readable without the file, add `blocked_by:` between epics;
  keep the roadmap file as the human view.

## Links

Two styles, chosen by whether the target moves.

**To a ticket: a wiki link by full file stem**, never a path and never a bare ID in prose:

```markdown
See [[login-page-drops-the-redirect-k7tm|the redirect bug]].
```

- Tickets move between `open/` and `resolved/`, which breaks any path link. A wiki link names
  the file, not the folder, so Obsidian resolves it wherever the file is (kept from bridle).
- Use the full stem (slug + ID), not just the ID: Obsidian matches whole file names, and the slug
  makes the raw link readable.
- Known downside: **GitHub does not render `[[...]]` links** in repository Markdown; they show as
  plain text. The ID at the end still makes the ticket easy to find (search `k7tm`). If GitHub
  clicks matter later, options are: never move files (keep state only in front matter; bridle
  has this as an open question), or have the CLI render a GitHub view with real links.
- Roadmap entries follow the same rule (they are links to tickets).

**To anything else in the repo** (docs, code, configs): a normal Markdown link with a path
relative to the file it appears in, so it works on GitHub and in Obsidian:

```markdown
The flow is in [the auth design](../design/auth.md); the handler is
[`src/auth/callback.ts`](../../src/auth/callback.ts).
```

- Relative paths work on GitHub in both the file view and rendered Markdown. Root-relative
  paths (`/docs/...`) work on GitHub but not in all Obsidian setups, so prefer relative.
- In Obsidian, set "New link format" to "Relative path to file" and turn off "Use [[Wikilinks]]"
  for normal links, so new links follow this rule; ticket links you type as `[[...]]` by hand
  (or let the CLI write).

## A small daemon and CLI

The problem it solves: tickets should be edited on the main branch, so every agent and person sees
the current state, but agents work in isolated worktrees and (with Claude Code's own worktrees)
can't write outside them. So the agent doesn't write ticket files; it asks a local daemon that
owns the main checkout.

- **Daemon**: listens on `127.0.0.1:<port>`, one per project (or one per machine with a project
  name in each request). It has the path to the project's main checkout, and is the only writer
  of `docs/tickets/` and `docs/roadmap.md` there. After each change it commits just that file to
  the main branch, with a message like `Ticket k7tm: filed` (bridle commits ticket files the same
  way; it never pushes for you).
- **CLI**: a thin client. It finds the daemon (a port file in the project, or an env var), sends
  a request, prints the answer. It works the same from the main checkout, a git worktree or a
  Claude Code worktree, because it touches no files itself; only a localhost connection is needed.
- **Commands**, enough to start:

  | command | does |
  |---------|------|
  | `new "<title>" --kind <k> [--parent <id>] [--body-file -]` | mint an ID, write the file in `open/`, commit; print the ID and path |
  | `show <id>` | print the ticket |
  | `list [--kind] [--parent <id>] [--state open\|resolved]` | one line per ticket |
  | `set <id> <field> <value>` | edit a front matter field (title, kind, parent, blocked_by, related) |
  | `append <id> --body-file -` | add text to the body (a note, a finding) |
  | `resolve <id>` | stamp `resolved:`, move to `resolved/`, commit |
  | `check` | IDs unique and well formed, file names match `id:`, references exist, wiki links resolve, roadmap rules |
  | `roadmap [show\|check]` | print or verify the roadmap |

- **Accept a bare ID anywhere** an ID is expected, and look the file up by its tail (`*-k7tm.md`
  in either folder). The daemon never needs an index: the files are the database.
- Only the daemon renames, moves or writes front matter; people and agents write bodies freely.
  That keeps IDs, names and links consistent without a lock on every edit.

## What this leaves out on purpose

Bridle has a database of task states, threads, watchers, readiness rules, version 2 tickets with
stages, and cross-project submission. None of that is needed to get stable IDs, epics and a
roadmap; add pieces only when the files alone stop being enough.
