---
id: python.ruff
severity: should
roles: [worker, reviewer]
---
Where a project has ruff configured, lint with `ruff check .` and format with
`ruff format .` (through the project's package-manager binding, e.g.
`uv run ruff check .` or `pipenv run ruff check .`).

- **Optional, not pack-wide.** Not every project on this pack uses ruff —
  don't add a ruff config to a project that has none just to satisfy this
  rule; skip it there.
- **Exclude prose from formatting.** Where ruff is in use, set
  `extend-exclude = ["*.md"]` in its config so quoted code samples in docs
  don't get reformatted by a repo-wide `ruff format .`.

Why: data-contracts runs both (`Makefile` `lint`/`format` targets,
`pyproject.toml`'s `ruff extend-exclude = ["*.md"]`, tickets at7m/yhgu on why
prose code samples shouldn't move). meta-notes has no linter or formatter
config at all (its survey: "no linter or formatter config"), so this can't be
a `must` for every project on the pack.
