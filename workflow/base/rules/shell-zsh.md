---
id: shell-zsh
severity: must
roles: [orchestrator, advisor, project-manager, manager, worker, reviewer, prototyper]
---
The shell is zsh (the human, 2026-10-03: "it's a bug that needs to go in every cloud, every rule
set somewhere"; ticket urdm).

- zsh doesn't word-split unquoted variables: `o='--time 11:00'; cmd $o` passes one argument, and
  `F="a.md b.md"; git add $F` one path. An unmatched glob is an error (`no matches found`).
- Never rely on word-splitting. Write the arguments out, or use an array:
  `args=(--time 11:00); cmd "${args[@]}"`.
- For a bash-idiom script, run it with `bash -c '...'` or a `bash <<'EOF'` heredoc.
