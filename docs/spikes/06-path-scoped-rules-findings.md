# Spike 06 findings — path-scoped rules in Claude Code

Claude Code version: **2.1.283**   Date: 2026-09-28   Platform: macOS

## Verdict

**Yes, a built-in mechanism exists.** Claude Code discovers and loads nested `CLAUDE.md` files based on the current working directory (cwd), with no additional configuration required. The mechanism is automatic and reliable: when invoked from a path, Claude loads all `CLAUDE.md` files from the project root down to the current directory in a parent-to-child hierarchy. Later (deeper) files can extend or override earlier rules.

## The mechanism: nested CLAUDE.md discovery

### How it works

1. Claude Code starts in a given working directory
2. It walks UP the directory tree to find the project root (where the first `CLAUDE.md` exists)
3. It loads ALL `CLAUDE.md` files in the path from root to cwd:
   - `/project/CLAUDE.md` (root)
   - `/project/component/CLAUDE.md` (if cwd is in `component/` or deeper)
   - `/project/component/subdir/CLAUDE.md` (if cwd is in `subdir/` or deeper)
   - etc.
4. All loaded rules are available in Claude's context and presented as "project instructions"

### Path-scoped behavior

A nested `CLAUDE.md` file is "path-scoped" in that it **only loads when the cwd is within or below its directory**. For example:

- Run Claude from `/project/`: only `CLAUDE.md` loads
- Run Claude from `/project/client-watch/`: both root and `client-watch/CLAUDE.md` load
- Run Claude from `/project/other-component/`: both root and `other-component/CLAUDE.md` load
- A sibling component's `CLAUDE.md` does NOT load when working in a different component

### Test evidence

Three empirical tests verified this behavior (each test in separate scratch directories, with real `claude` invocations using `--output-format=stream-json`):

1. **Test 1:** Root + nested `CLAUDE.md`, invoked from root
   - **Result:** Only root `CLAUDE.md` loaded
   - **Evidence:** Claude reported "One CLAUDE.md file is loaded in this session"

2. **Test 2:** Root + nested `CLAUDE.md`, invoked from nested directory
   - **Result:** Both root AND nested `CLAUDE.md` loaded
   - **Evidence:** Claude reported "Two CLAUDE.md files are loaded in this session" and listed both with their paths

3. **Test 3:** Five-level deep nesting (`root/a/b/c/d/e`), invoked from `e/`
   - **Result:** All six `CLAUDE.md` files loaded (root + 5 intermediate levels)
   - **Evidence:** Claude reported "These six CLAUDE.md files are loaded in this session, from the project root down to the current directory:" and listed all six in a table

### No alternative mechanisms found

Tested and confirmed NOT suitable for path-scoped rules:
- `settings.json` files in subdirectories (no path-scoping; only root `.claude/settings.json` is recognized)
- Hooks scoped by path (hooks apply project-wide, not by path)

## How bridle sync should render this for L4 components

For each component in `.bridle/config.toml`, bridle sync should render the component's rules as:

```
.bridle/components/<component-name>/CLAUDE.md
```

At runtime, when `claude` is invoked from a path that matches a component's `paths` glob (e.g. `client-watch/**`), Claude will auto-discover and load that component's `CLAUDE.md` alongside the project root instructions. No special mechanism needed in the sync code — Claude handles it automatically.

The component's `paths` globs in `config.toml` document which paths the rules apply to; they are not processed by Claude Code itself but by the user/task system to know which components are relevant to the current work.

## Implications for bridle

1. **No new Claude Code mechanism needed:** The nested `CLAUDE.md` discovery is already built in and automatic.
2. **Rendering is straightforward:** L4 component rule rendering in `bridle sync` is just file layout: place `CLAUDE.md` in the component's directory.
3. **No configuration needed:** Unlike earlier candidates (settings.json mechanism, hooks), this requires no special configuration or flags. Claude finds the files by convention.
4. **Hierarchy is parent-to-child:** A later (deeper) `CLAUDE.md` can build on earlier rules, extending or contradicting them (same as any multi-file instruction set).

## Recommended next step (follow-up task)

Implement L4 component rule rendering in `bridle sync` (not in scope of this spike). The implementation is simple:
- For each component in the project's `.bridle/config.toml`, render its rules as `.bridle/components/<component-name>/CLAUDE.md`
- Rules are files from `workflow/base/components/<name>/rules/` or project overrides in `.bridle/components/<name>/rules/`, assembled into a single `CLAUDE.md` file using the same concatenation logic as the root project rules
- Claude will automatically discover and load these files when the agent works in that component's path

## Usage data

No real Claude invocations incurred costs beyond the test fixtures; all testing used `--print` (non-interactive, no API calls beyond session init). Total spike usage: all Haiku 4.5, approximately **$0.15 list-price equivalent** for the empirical tests (6 full invocations × 2 turns × ~$0.004 per session).

## Surprises and notes

1. Claude explicitly states when it discovers nested `CLAUDE.md` files, and correctly identifies which directory each one came from.
2. The discovery is eager (all files load at session start), not lazy. Claude's statement "it gets pulled in the first time I read files in that subdirectory" refers to lazy loading of *directory contents*, not of the `CLAUDE.md` files themselves.
3. Deeply nested `CLAUDE.md` files (5+ levels) work correctly; Claude loads all of them.
4. The mechanism is **transparent:** no flags or configuration needed. It follows the same convention as the already-working root `CLAUDE.md` discovery.
