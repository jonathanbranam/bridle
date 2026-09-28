---
id: vim.no-after-ftplugin
severity: should
roles: [worker, reviewer]
---
Don't make a plugin's behaviour depend on its `after/ftplugin/` files being on
the runtimepath.

Whether `after/` directories are loaded depends on how the plugin is
installed and on the user's runtimepath (plugin managers, packages, a clean
`vim -es` test run). Put filetype behaviour in `ftplugin/` or `plugin/`, or
wire it up from the plugin's own code (e.g. an `autoload/` function called
from a `FileType` autocmd the plugin registers), so it works wherever the
plugin itself loads.

Why: a mapping or setting that lives only in `after/ftplugin/` silently
vanishes under a clean runtimepath, including the one the vader runner uses
(`vim.vader`). Project-specific mapping conventions belong in the project's
own rules, not here.
