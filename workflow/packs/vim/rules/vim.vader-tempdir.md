---
id: vim.vader-tempdir
severity: must
roles: [worker, reviewer]
---
A vader test that touches the file system works in a temporary directory,
never in the repo or the user's real files, and removes it at the end. Name it
`g:test_dir` everywhere (not `g:test_root`) so setup, tests and cleanup agree.

```vim
Execute (Setup - create temporary test directory):
  let g:test_dir = tempname()
  call mkdir(g:test_dir, 'p')
  let g:original_dir = getcwd()
  execute 'cd' fnameescape(g:test_dir)

Execute (Test that writes a file):
  call mkdir(g:test_dir . '/note/path', 'p')
  call writefile(['# Sample Note'], g:test_dir . '/note/path/Filename.md')

Execute (Cleanup):
  execute 'cd' fnameescape(g:original_dir)
  call delete(g:test_dir, 'rf')
  unlet g:test_dir g:original_dir
```

- Create every directory before writing into it (`mkdir(..., 'p')`).
- Change back to `g:original_dir` *before* deleting `g:test_dir`; deleting the
  current directory leaves later tests in a dead cwd.
- Put cleanup in an `Execute` block that always runs, and `unlet` the globals
  so one file's state doesn't leak into the next.

Why: vader runs every file in one Vim session, so leftover globals or a
changed cwd break unrelated tests.
