+++
id = "br-5cru"
title = "Commands that run bridle on another machine over ssh fail unless the user adds bridle to the non-interactive PATH"
kind = "bug"
state = "built"
created_at = "2026-10-14T01:20:43.432Z"
updated_at = "2026-10-14T03:55:48.902573Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:5cru",
]
branch = "bridle/5cru"
commit = "16702723"
summary = "Design only: recommend option B (remote snippet appends ~/.local/bin and ~/.cargo/bin to PATH before exec bridle; exit 127 = 'bridle not found on <host>', 255 = unreachable); no config. Options, rejections and a 2-ticket split are in the thread comment."
ticket = "5cru"
+++

docs/tickets/open/commands-that-run-bridle-on-another-machine-over-ssh-fail-un-5cru.md

## Thread

### note · external:advisor/product-manager · 2026-10-14T01:25:16.433Z
The human, 2026-10-13 ~9:25 PM ET (to advisor product-manager), verbatim: "another related issue is 5cru that bridle needs to be on the remote non-interactive path which requires changing /etc/zshenv. I'm not sure a real solution here, it can be researched and designed and then returned to me for review." Scope: research and a design (options, recommendation) written into this ticket, then back to the human for review. No build until the human approves.

### note · agent:pm-1 · 2026-10-14T01:25:33.195Z
Brief (pm-1). DESIGN ONLY, no build (the PdM, from the human: write options and a recommendation into the ticket, then it returns to the human for review). Model: Sonnet. Size: S. Role: designer.

Read the ticket (the findings and the three options are there), crates/bridle/src/token_pair.rs (~92-111), crates/bridle/src/preflight_remote.rs (~212), docs/design/cli.md (token pair, project preflight), docs/context/add-a-machine.md and docs/context/windows-wsl2-host.md. Write a "## Design" section on the ticket (via the ticket thread if you cannot edit the file; comment with the full text) covering: each option's behaviour on macOS, Ubuntu and WSL2; what each does to a machine set up by add-a-machine today with no PATH edit; failure reporting ("bridle not found on <host>", never "unreachable"); how the remote path is found or recorded and where that lives (and, if it adds config, how existing machines get a default without edits); what add-a-machine and the WSL2 doc must say meanwhile; rejected alternatives with reasons; the tickets a build would split into (sizes, which share files). End with ONE recommendation. Include the c2 wiki in any web search if you search at all (rule c2-wiki-in-searches), and say if it could not be reached. Do not change code. Acceptance: the design is on the ticket thread; mark it done and stop; the human reviews.

### note · agent:5cru · 2026-10-14T03:55:46.936Z
## Design (5cru, designer worker; design only, no code changed)

### What runs bridle over ssh today

- `token_pair.rs` `Exec::run` (line ~111): `ssh -o BatchMode=yes <host> bridle <args>`. `Exec` is the one Runner shared by `token pair`, `project stop --machine` and `project start` (they import it from token_pair), so one fix there covers all three.
- `preflight_remote.rs` `Ssh::run`: `ssh -o BatchMode=yes <host> <script>`, with the script `bridle --version`. Its other snippets use git, tmux, claude and so on, not bridle.
- `projects sync` uses only `cat`/`mv`: unaffected.
- The remote shell is the user's login shell run as `$SHELL -c '<args joined by spaces>'`, non-interactive and non-login. zsh reads only the zshenv files then; bash reads nothing (or ~/.bashrc on some distros). The cause is the same on macOS, Ubuntu and WSL2: the PATH is the system default and lacks `~/.cargo/bin` (`just install`) and `~/.local/bin` (`install-release.sh`).
- Two reporting defects, separate from the PATH one: pair prints "unreachable, skipped" for any failure (line ~275), and preflight's "bridle version" check says "bridle not found" for every non-zero or unparsable result, including a bridle that exists but errors. ssh itself exits 255 on a connection failure; a shell that cannot find the command exits 127. That split is reliable and is what the fix should use.

### Options

**A. Shell setup by the user (today's interim).** One `path=(...)` line in `/etc/zshenv` (macOS) or `/etc/zsh/zshenv` (Ubuntu, WSL2). Works on all three, but needs sudo, differs per OS, breaks for a user whose shell is bash/fish, and the thoughtbot dotfiles even warn on PATH edits in the user's own zshenv. A machine set up by add-a-machine fails until it is done. This is the thing the human wants to stop asking of users. Keep it only as a documented fallback.

**B. Try known directories at the call site (the ticket's first option).** The remote command becomes a small snippet that prepends the known install dirs to PATH and then execs bridle:

    PATH="$PATH:$HOME/.local/bin:$HOME/.cargo/bin"; exec bridle <quoted args>

It uses only `PATH=..; exec` and `$HOME`, which parse the same in zsh, bash and sh, so it does not matter which shell sshd starts (macOS zsh, Ubuntu bash or zsh, WSL2 Ubuntu). It prefers a bridle already on PATH (see Order below). A machine set up by add-a-machine today with no PATH edit works at once, with no config and no migration. Args must be single-quoted for the remote shell (today they are joined raw, so an arg with a space or `$` already misbehaves; pair's args are plain words, but quoting is the correct thing to add while touching this line).

Order: prepending the two dirs would shadow a PATH bridle with the install dir copy. Appending them after `$PATH` keeps "PATH wins, then known dirs", which is the ticket's wording ("try bridle on PATH, then the known directories") and avoids surprising a user who keeps another build elsewhere. So: append.

Failure reporting: exit 127 from the remote means not found (the snippet's `exec bridle` fails with 127 in all three shells), 255 means ssh failed, anything else means bridle ran and failed (its stderr is the reason). The Runner returns a typed error (an enum, or at least a marker the callers match) instead of a bare string, so pair says `machine nuc: bridle not found on nuc (looked on PATH, ~/.local/bin, ~/.cargo/bin); install it, or see docs/context/add-a-machine.md` and `machine nuc: unreachable, skipped: <ssh error>` only for 255. Preflight does the same for its version check (127 -> "bridle not found", other non-zero -> the real error, unparsable output -> "unexpected output: ..."), instead of folding everything into "not found".

Reporting which one was used: ask the snippet to print nothing extra (stdout of the helper is parsed); instead run `command -v bridle` once in preflight's version check (`PATH=..; command -v bridle; bridle --version`) and show the path in the pass message ("0.9.2 at /home/u/.cargo/bin/bridle"). Pair does not need to show it. This is the only place the user benefits from knowing.

**C. Record the path per machine in `[machines]`.** `[machines]` today is `name = "host"` (a bare string, read by `bridle_api::machines`, written by `projects sync`/add-a-machine). Adding a path means either a new table form (`nuc = { host = "nuc", bridle = "/home/u/.cargo/bin/bridle" }`, which every reader and `projects sync`'s byte comparison must handle) or a parallel `[machine_bridle]` table. Existing machines get no value, so a default (B's probing) is needed anyway; the config then only duplicates what probing finds, can go stale after a reinstall in a different place, and has to be kept in step across machines by `projects sync`. Cost without a benefit that B lacks. Rejected (see below), though a per-machine override is the right later step if anyone installs bridle somewhere unusual (YAGNI now).

**D. Login shell: `ssh host '$SHELL -lc "bridle ..."'`.** zsh -l reads zprofile/zlogin, not zshrc, so the thoughtbot setup (PATH in `.zshrc`) still fails; to cover it you need `-ilc`, which runs the user's whole rc file, can print banners, ask questions or hang (BatchMode only stops ssh prompts). It also fails on fish and csh, and needs `$SHELL` quoting layers. Unpredictable, as the ticket says. Rejected.

**E. Ask sshd for a PATH (`ssh -o SendEnv`, `~/.ssh/environment`, `PermitUserEnvironment`).** Needs server-side sshd config changes (root, off by default, and macOS/Windows OpenSSH differ). Worse than A. Rejected.

**F. Install a symlink in a PATH dir (`/usr/local/bin/bridle`) from `just install`/`install-release.sh`.** Works on all of them (it is on the default PATH of macOS and Ubuntu) but needs sudo, and a daemon self-upgrade then has to keep the link valid. Better as a convenience in the future than a requirement. Rejected for now.

### Where the path lives; config defaults for existing machines

Nowhere: B computes it on every call from two constants (`~/.local/bin`, `~/.cargo/bin`), defined once in a small helper (for example `remote_bridle_cmd(args) -> String` in `token_pair.rs`, used by `preflight_remote.rs`). No config key, no migration, no change for machines that already work. If the human later wants an override, C's table form can be added then with the default being B's list.

### Meanwhile: what add-a-machine and the WSL2 doc must say (until the fix lands)

Both docs get a short "Remote PATH" step in the ssh section: the two zshenv lines from the ticket (macOS `/etc/zshenv`, Ubuntu and WSL2 `/etc/zsh/zshenv`), the check `ssh <host> 'command -v bridle'`, and the note that `bridle token pair`, `project stop --machine`, `project start` and `project preflight --to` need it. After B lands, the step becomes "not needed unless bridle is installed outside `~/.local/bin` and `~/.cargo/bin`", and `docs/design/cli.md` line 148 ("bridle on the remote PATH") changes to "bridle on the remote PATH or in `~/.local/bin` or `~/.cargo/bin`". The windows-wsl2-host line 242 export snippet only fixes the interactive shell; say so.

### Rejected alternatives (summary)

A as the product answer (sudo, per-OS, per-shell); C (config that duplicates probing, needs a format change in a file read by many); D (rc files, unpredictable); E (server config); F (sudo; fine as an extra later). Also rejected: caching the found path in a state file (same staleness as C) and widening "unreachable" to cover everything (the defect being reported).

### Tickets a build would split into

1. **Remote bridle command + typed not-found error** (size S; files: `token_pair.rs`, `preflight_remote.rs`, plus tests in both). `Exec` builds the PATH-extended, quoted snippet; the Runner error says not-found (127) vs ssh-failed (255) vs bridle-failed; pair prints "bridle not found on <host>" instead of "unreachable"; preflight's version check separates not found, unparsable and too-old, and shows the path it found. Callers in `project_stop.rs` and `project_start.rs` only need to show the new error text (their tests use a fake Runner, so a small signature change at most). Tests: fake-ssh scripts or the existing fake Runner cover 127/255/other; one test of the snippet string and its quoting.
2. **Docs: Remote PATH step and cli.md line** (size XS, Haiku; files: `docs/context/add-a-machine.md`, `docs/context/windows-wsl2-host.md`, `docs/design/cli.md`, CHANGELOG). This one can be done now, ahead of 1, with the zshenv instructions, and edited by 1's worker (or a follow-up line) when the fix lands. It shares no source files with 1, only the cli.md line 148 text, so the order does not matter; do not run it in parallel with another cli.md editor.

No ticket for C or F now (YAGNI); note C as the follow-up if someone installs bridle elsewhere.

### Recommendation

**Option B**: the remote command is `PATH="$PATH:$HOME/.local/bin:$HOME/.cargo/bin"; exec bridle <quoted args>` (PATH first, then the two known install dirs), built in one helper used by pair, stop and start and by preflight; exit 127 reports "bridle not found on <host>", 255 "unreachable", anything else bridle's own error. No config, no per-machine record, no change to existing machines. Ship ticket 2 (docs, the interim zshenv step) at once, ticket 1 after the human approves.

Search note: no web search was done; the design rests on the code and on POSIX/ssh behaviour (ssh exit 255, shell exit 127), so the c2 wiki was not consulted.

### note · agent:5cru · 2026-10-14T03:55:48.849Z
built: 16702723 (branch bridle/5cru)

### note · agent:5cru · 2026-10-14T03:55:48.902Z
done: design written as a full comment on br-5cru (recommend B: remote snippet appends known install dirs to PATH, 127 = bridle not found, 255 = unreachable; docs ticket can go first); no code, branch at 16702723
