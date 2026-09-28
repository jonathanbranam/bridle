---
id: vim.vader
severity: must
roles: [worker, reviewer]
---
Test Vimscript behaviour with [vader.vim](https://github.com/junegunn/vader.vim),
in `test/*.vader`.

- **Runner.** The project's own `run_tests.sh` runs them:
  `./run_tests.sh [file] [--quiet|--debug|--interactive]`. The script belongs to
  the project; this pack states the convention, it doesn't supply the script.
- **Clean runtimepath.** The runner starts `vim -es` with a clean runtimepath
  (no user vimrc or plugins) and adds only the plugin under test and vader, so
  a test can't pass because of something in the developer's own setup.
- **`$VADER_PATH`.** Vader is loaded from `$VADER_PATH`, default
  `~/.vim/pack/testing/start/vader.vim`. If it isn't there, say so; don't
  install it or edit the runner.

Why: it's the framework meta-notes uses; `-es` keeps runs headless and
scriptable for agents.

Pointer, not built: turning specs into vader tests (a vader adapter for the
spec-to-tests pipeline) is future work; see meta-notes'
`docs/gherkin-compiler-testing.md`.
