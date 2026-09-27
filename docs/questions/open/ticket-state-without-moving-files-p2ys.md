---
id: p2ys
title: Can ticket state live without moving files, and still be read and edited as markdown?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [c7eb, c5a8, k4wq]
---

## The question

Filed as research. The human's words, verbatim:

> One thing that I'm not sure how it has been resolved: as compared to the
> current solution which heavily uses folders, I think the target state for
> bridle should not require moving files around to indicate state. That is
> going to be a big hassle. I would still prefer to have documents
> human-readable and editable, so I still prefer markdown, but I think the best
> design will be to collapse the ticket system folder layout significantly.
>
> beads does this all with jsonl and so I don't know how a human interacts with
> that system outside the CLI. I find using either vim or Obsidian (which is
> much nicer for reading markdown) to be great solutions. With a CLI vim could
> be launched by bridle for reading and editing but that won't work with
> Obsidian.

And, following up:

> The alternate approach to the Obsidian desire would be a web-based GUI that
> talks to bridle directly.

"The current solution" is `workflow-instructions/ticket-conventions.md`, where
the folder is the state (`intake/<kind>/` → `backlog/` → `active/` →
`archive/landed|dropped/`, every move a `git mv`) and there is deliberately no
`state:` or `kind:` field.

## Why it matters

It decides how the human reads, edits and moves work: whether that happens in
vim and Obsidian on plain files, or only through `bridle` (its CLI or a web GUI). It also
decides whether `bridle`'s store and a human's editor can both write the same
record without one silently losing the other's change.

## Notes

What the docs say today, found while filing. None of it is decided.

- **Nothing states it either way.** The folder-as-state rule was never carried
  into bridle's task design, and never explicitly dropped.
- **The task design already implies flat files with state as a field.**
  [[docs/design/storage#The state branch|The state branch]] has one flat
  `tasks/tw-7fa2.md` per task ("TOML frontmatter + markdown body + thread"),
  plus `events/<month>.jsonl` for transitions, and SQLite as the read path.
  [[docs/design/roles-and-lifecycle#Task lifecycle|The lifecycle]] has ten
  states (`open`, `planned`, `ready`, `claimed`, …), too many to be folders, and
  two of them (`ready`, `blocked`) are computed. Kinds are a list, not intake
  subfolders. The docs never say that the state is stored in the frontmatter,
  though.
- **These docs themselves still use folders for state.**
  `docs/questions/open/` → `resolved/` and `docs/spikes/open/` → `done/`, by
  the rules in [[docs/README|the docs README]], copied from
  `ticket-conventions.md`.
- **The argument for folders.** `workflow/research/13-ticket-system.md` §3.3
  gives two reasons: promotion as a `git mv` is "a deliberate, visible,
  reviewable act" where "a one-line frontmatter edit is too quiet", and "what is
  in flight" is just `ls active/`. In bridle, an accept or promote could be a
  command that writes an event to `events/*.jsonl` and its own commit, and
  `bridle task list` replaces `ls`. Whether that is visible enough when the
  human edits the file directly is part of this question.
- **The argument against a database, which still stands.** Research 13 §3.1:
  *"a database — Beads' JSONL included — makes everything opaque to a human who
  wants to open a file and read it."* Research 02 §5 says the same of beads'
  JSONL. The storage design follows that rule: markdown is the record, and
  SQLite is an index that can be rebuilt.
- **Things that affect vim and Obsidian directly**, to research:
  - **TOML frontmatter.** Storage specifies TOML. Obsidian's Properties are
    believed to read YAML (`---`) frontmatter only, which would make TOML
    (`+++`) show as body text and leave it out of search and Dataview.
    Unverified. The ticket system and these docs use YAML.
  - **Where the files live.** On the state branch, tasks sit in a separate
    worktree, `<workspace>/.bridle/state/` ([[docs/design/storage#The state branch|storage]]). That
    is a dot-folder, which Obsidian is believed to skip when indexing. Unverified. The
    vault would have to point at it, or at a symlink to it. In-tree files would
    sit in the vault already: [[task-records-on-a-state-branch-or-in-tree-c7eb|state branch or in-tree]].
  - **Edits made outside bridle.** SQLite is the read path, and every durable
    write goes to both the database and the file. A hand edit in vim or
    Obsidian reaches only the file. Something has to notice it (a file watch,
    a check on read, `bridle rebuild`) and reject what it can't accept, such
    as an illegal transition or a changed `id`. It also has to deal with bridle
    writing the same file at the same time, since the task thread is appended
    to the file.
  - **Launching an editor.** `bridle task edit` ([[docs/design/cli|CLI]]) can
    open `$EDITOR` and check the file when the editor exits. For Obsidian,
    an `obsidian://open?vault=…&file=…` URI might open the right note, but
    bridle would get no exit to check on. Obsidian would need the file watch
    above.
- **The alternative to Obsidian: a web GUI that talks to bridle.** The human
  added that the markdown viewer and editor is **a separate project**, not part
  of bridle. Bridle's part is only commands, and the API behind them, to:
  - read a document,
  - navigate documents (list, follow links and `needs:`/`see:`),
  - update a document,
  - handle conflicts, probably by reporting them. One way is optimistic
    concurrency: a read returns a version, an update sends back the version
    it was based on, and the daemon rejects a stale update and says why,
    rather than merging silently.

  Edits then go through the daemon, so validation and concurrent writes are
  handled where the state lives. The web GUI has no file watch, vault location
  or frontmatter dialect to worry about. Hand edits in vim still reach only the
  file, so the questions above still apply to them.

  **What exists today:** none of this. The v1 API
  (`crates/bridle-daemon/src/server.rs`) has agents, messages, events, usage
  and tokens, but no task or document endpoints. `bridle task
  new|show|edit|list|drop|reopen` is in [[docs/design/cli|the CLI design]]
  under the phases after v1. `show`, `list` and `edit` cover read, navigate and
  update in outline, but nothing covers conflicts yet. A web GUI is also the
  "local web board" in [[a-human-surface-beyond-the-cli-k4wq|a human surface beyond the CLI]].
- **Related:** where questions live ([[where-questions-live-on-the-state-branch-c5a8|c5a8]]),
  and the human surface beyond the CLI ([[a-human-surface-beyond-the-cli-k4wq|k4wq]]).
