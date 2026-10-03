---
id: urdm
title: "Base rule: the shell is zsh, which doesn't word-split unquoted variables"
kind: chore
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask


The human (2026-10-03, via the notes advisor on the NUC, relayed by the NUC orchestrator in
m-3971 to the bridle advisor): "it's a bug that needs to go in every cloud, every rule set
somewhere." The relay, verbatim:

> The human (2026-10-03, via the notes advisor) wants a base rule for every bridle role: the shell
> is zsh, which does not word-split unquoted variables (o='--time 11:00'; cmd $o passes ONE
> argument -> 'unrecognized arguments'). Rule: never rely on word-splitting; use arrays
> ("${args[@]}") or write flags out; run bash-idiom scripts with bash -c or a bash heredoc.

## Seen here too

The bridle advisor hit it on dalek the same day: `F="a.md b.md c.md d.md"; git add $F` passed one
path with spaces, `git add` failed ("did not match any files"), and the files had to be named
out. Agents write bash idioms by habit; the Bash tool runs the user's shell (zsh on both
machines).

## Wanted

A base rule in `workflow/base/rules/` for every role (all of `orchestrator, advisor,
product-manager, manager, worker, reviewer, prototyper`):

- The shell is zsh. It doesn't word-split unquoted variables (and an unmatched glob is an error,
  `no matches found`).
- Never rely on word-splitting: write the arguments out, or use an array (`args=(--time 11:00);
  cmd "${args[@]}"`).
- For a bash-idiom script, run it with `bash -c '...'` or a `bash <<'EOF'` heredoc.

Separately, the notes advisor is drafting feedback to Anthropic for the human's review; the
human's global `~/.claude/CLAUDE.md` is theirs to edit.
