---
id: typescript.package-manager
severity: must
roles: [worker, reviewer]
---
Manage TypeScript dependencies and run commands through npm workspaces,
respecting the project's lockfile.

Install with `npm ci` (fresh worktree) or `npm install --prefer-offline`
(existing worktree), add a dependency with `npm install <pkg>`, and run tools
through `npm run` (e.g. `npm run test`). Don't add dependencies without saying
so in the handoff.

Why: npm workspaces and committed lockfiles ensure deterministic installs
across agents and machines. The setup hook (`br-42dd`) installs dependencies
in new worktrees, so agents inherit a consistent environment.
