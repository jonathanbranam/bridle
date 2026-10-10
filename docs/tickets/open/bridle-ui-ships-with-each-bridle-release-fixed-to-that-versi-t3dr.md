---
id: t3dr
title: bridle-ui ships with each bridle release, fixed to that version, and self-upgrade moves both
kind: feature
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [fv86, tc7t, chvf]
tasks: [br-t3dr, br-k3jk]
---

## The ask

The human, 2026-10-10 ~2:00 PM ET, verbatim (to advisor product-manager):

> What I expect from this, if it's not in this epic, we should add it, is that I can install
> bridle without doing a clone, and that can run and self-upgrade. It includes the bridle daemon,
> the bridle gateway, and, of course, the bridle-ui as part of that, all fixed to a version that
> comes from bridle. ... If the UI is not done, that could be a separate epic because I understand
> it's a different piece of work.

## Where it stands (PdM, 2026-10-10)

- The gateway serves `~/.bridle/ui/` and checks the UI build's recorded API version against its
  own: warn or refuse (`docs/design/human-web-ui.md`, task 8).
- bridle-ui is its own repo and project. Its build is put in `~/.bridle/ui/` by hand from a
  bridle-ui clone. A bridle release carries only the `bridle` binary, so a machine installed from
  a release (fv86) has no UI, and a self-upgrade doesn't move the UI.
- tc7t option C (br-785a, pending): a landing that needs a UI install shows without a manual step.
  It is about the dev machine following landings, not releases.

## The ask

The UI ships with bridle, fixed to a bridle version: a machine installed from a release gets the
UI that matches it, and a self-upgrade moves both together. For a designer pass: where the UI
build comes from (bridle's release workflow builds a pinned bridle-ui commit, or bridle-ui
publishes its own release that bridle names), how the pin is recorded, how the daemon installs it
into `~/.bridle/ui/` on upgrade, and how this fits tc7t on the dev machine.

## The human's requirements for the short term (2026-10-10)

The human, 2026-10-10 ~1:30 PM ET, verbatim (to the aide):

> And on the UI, I'm okay if that's a separate command, or if it's a separate tarball or a
> separate download, or if I'm going to send this somewhere else. Yeah, I can do that.
>
> For shipping the UI, I lost it. X, shoot. There's a ticket that says that the UI is shipped
> along with Bridle, upgraded along with Bridle on a machine that doesn't have a clone. I think
> it's going to go in a new epic for the product manager.
>
> What I want to say is that I'm okay if that's slightly separate, or I don't care if it's a
> `git clone`, a read-only clone or something, a shallow clone, if that's the way to do it. Maybe
> that's fine, or if it needs a local build of Node, a local compile, I don't really care that
> much. Long term, it should be kind of part of the Bridle installation, but if it's a separate
> thing, that's fine.
>
> What I really want is that the short-term deliverable does not require cloning the Bridle UI
> and setting up Bridle on it. It should also be automated, and it should be tied to the Bridle
> version so that the two stay in sync. Those are kind of the main requirements.

So, for the designer: **must** (short term) — no hand-made bridle-ui clone with bridle set up on
it; automated; tied to the bridle version so the two stay in step. **May** — a separate command,
tarball or download; a shallow or read-only clone made by bridle itself; a local Node build.
**Long term** — part of the bridle installation.

## Design

Focus: internal architecture (release pipeline and the upgrade path), with one small user-facing
piece (what a release machine types: nothing).

### Problem in a sentence

A machine installed from a bridle release has no UI, and a self-upgrade can't move one, because
the UI is built by hand from a bridle-ui clone (`npm run install-ui`) and the release carries
only `bridle`.

### What exists today

- `release.yml`: on a `v*` tag, builds `bridle` for three targets, packages
  `bridle-<tag>-<target>.tar.gz` (just `bridle`, `scripts/package-release.sh`), publishes with
  `SHA256SUMS`.
- `self_upgrade = "release"` (`crates/bridle-daemon/src/release.rs`, `upgrade.rs`): finds the
  newest release over `curl`, checks the tarball against `SHA256SUMS`, swaps the binary, and
  (chvf step 3) fetches the workflow at the same tag. That is exactly the hook a UI step belongs
  beside.
- `bridle-ui/scripts/install-ui.mjs`: builds, then copies `dist/` into `~/.bridle/ui/` by atomic
  rename. `dist/api-version` records the API version it targets.
- Gateway (`crates/bridle-gateway/src/ui.rs`): serves `~/.bridle/ui/`, compares `api-version`
  with its own `API_VERSION`, warns or refuses. This stays as the safety net whatever we pick.
- design doc: "Not compiled into the `bridle` binary (no Node in bridle's build)."
- tc7t / br-785a: the dev machine runs `main`, not releases, and has a bridle-ui clone.
  fv86: the first install from a release, no clone (installer script, not built yet).

### Options

**A. Release workflow builds a pinned bridle-ui and puts it in the release.**
bridle keeps a one-line pin file (say `ui.pin`: a bridle-ui tag or sha). `release.yml` gets a
`ui` job: check out bridle-ui at the pin, `npm ci && npm run build`, upload `dist/` as an
artifact; the target jobs add it to the package.
  - A1, bundled: `ui/` goes inside each `bridle-<tag>-<target>.tar.gz` (the UI is
    platform-independent, so it is copied three times; a few hundred KB). One download, one
    checksum, one name. The fv86 installer extracts `bridle` to the PATH and `ui/` to
    `~/.bridle/ui/`. On upgrade, the daemon, after the binary swap, extracts `ui/` from the tarball
    it already downloaded and verified, to `ui.tmp` then renames over `~/.bridle/ui/` (same
    rename dance as `install-ui.mjs`).
  - A2, separate asset: `bridle-ui-<tag>.tar.gz` beside the binaries, listed in `SHA256SUMS`. One
    more asset name to look up and a second download in the daemon; the only gain is that the
    installer could skip it.
  - User sees: nothing. A release machine has the UI at install and after every upgrade; no
    command, no Node, no clone.
  - Pin: a file in bridle's git, changed by commit, so it is reviewed, tagged with the release and
    reproducible. The tarball also carries `ui/bridle-version` (tag + bridle-ui sha) so a machine
    can say what it runs. CI check (cheap): the built `api-version` equals the gateway
    `API_VERSION`, or the release fails.
  - Costs: Node in the release workflow; the workflow must be able to read the bridle-ui repo (a
    deploy key or token secret if it is private); a pin bump is a step before a release.
  - Falls short: a bridle-ui fix reaches release machines only with the next bridle release.
    That is the "tied to the bridle version" the human asked for, so it is a feature, not a gap.
  - Principles: KISS (reuses the tarball, checksum and upgrade hook); modularity (the daemon
    only unpacks a folder, it knows nothing of Node or bridle-ui); user's side first (zero
    commands). Couples release.yml to bridle-ui's repo and build command, which the doc
    already coupled by naming `npm run install-ui`.

**B. bridle-ui publishes its own release; bridle names the tag.**
bridle-ui's CI builds and attaches `bridle-ui-<v>.tar.gz` on its tags. bridle's `ui.pin` names
that tag; the daemon (and installer) downloads it from the bridle-ui repo's release.
  - User sees: nothing, same as A.
  - Costs: a second release process to build and keep green; the daemon talks to two repos (a
    second `repo` setting, two rate-limit budgets, two failure modes); the pin and the bridle
    release can drift (pin names a tag that was deleted or never published) and nothing in
    bridle's release run checks it. The upgrade also isn't atomic with the binary: the binary
    swap can succeed and the UI download fail.
  - Gains: bridle's release workflow needs no Node and no cross-repo read; bridle-ui can cut
    releases on its own schedule (YAGNI: nobody asked for that).
  - Fails: "one name per action" is fine, but A's single artifact is simpler to keep in step.

**C. bridle makes a shallow clone and builds on the machine.**
`bridle ui install` (and the upgrade) does `git clone --depth 1` of bridle-ui at the pin into
`~/.bridle/ui-src`, runs `npm ci && npm run install-ui`.
  - Allowed by the human's "may", and needs nothing in CI.
  - Costs: Node and npm on every machine (and a network to npm on every upgrade); a build that
    can fail halfway on a machine nobody is watching, inside the daemon's upgrade; minutes of
    build at the quiet point; git access to bridle-ui from every machine (SSH key or token on
    each). It reintroduces a clone, just hidden. Fails the spirit of "no clone" and fv86's "no
    Rust toolchain, no clone" install.
  - Useful only as a stopgap if A can't be done soon.

**D. Embed the build in the `bridle` binary** (`include_dir!`, served by the gateway).
Truest to "long term, part of the installation": one file, one version, nothing to unpack.
  - Costs: every `cargo build`, including every worker worktree and `just check`, needs a UI build
    or a stub; Node enters bridle's build, which the design doc rules out; the binary grows with
    UI churn; and the dev machine can no longer update the UI without a Rust rebuild.
  - Not now. A's tarball puts the same files on disk from the same release, so moving to D later
    changes where the gateway reads from and nothing else.

**E. Do nothing.** Release machines have no UI and the human hand-builds it; fails the human's
must ("no hand-made clone, automated, tied to the version"). Honest only for the dev machine.

### The dev machine (tc7t)

The dev machine follows `main` (`self_upgrade = true`), not tags, and has a bridle-ui clone, so
it is outside A's path and should stay there: pinning it to the last release's UI would hide
new bridle-ui work. br-785a keeps doing its own thing (at a quiet point after a bridle-ui
landing, run `install-ui` from the clone; restart the gateway). The two meet at one folder,
`~/.bridle/ui/`, and one check, the gateway's `api-version` compare. If a machine is on
`self_upgrade = "release"`, the daemon owns the folder; if on `true`, the clone owns it. The
daemon should only write the folder in release mode, so a dev install is never overwritten.

### Recommendation: A1 (bundle `ui/` in the existing tarball, pinned by `ui.pin`)

Why: it is the only option where the binary, workflow and UI come from one verified download at
one tag, so they cannot drift and the upgrade cannot half-succeed on a second fetch. It adds no
command (nothing for the user to type), no Node or clone on the machine, and one small step to
the daemon's existing release path. It matches chvf (binary and workflow move together by
release) and fv86 (installer extracts both).

Accepts: Node and a cross-repo read in `release.yml`; a pin-bump step before each release; the
UI is copied into three tarballs; release machines get UI changes only at a bridle release.

Rejected: B (two release processes and two fetches for a need nobody has; drift between pin and
published tag), C (puts Node, git access and a long build on every machine; a clone in disguise),
D (Node in bridle's build; revisit as the long-term step once A works), E.

Not included (YAGNI): a `bridle ui install` command. The installer (fv86) and the upgrade both
unpack the folder; add the command only when a machine needs a manual repair, and then as the
same unpack code the daemon uses.

### Build shape (for a later plan, not a plan)

1. `ui.pin` plus a release check that the pinned build's `api-version` matches `API_VERSION`.
2. `release.yml`: a `ui` job; `package-release.sh` adds `ui/` (and `ui/bridle-version`).
3. Daemon release path: after the swap, extract `ui/` if the tarball has one (older tags don't;
   leave the folder alone then), atomic rename, only in `self_upgrade = "release"` mode.
4. fv86 installer extracts `ui/`. Docs: `human-web-ui.md`, `daemon.md` "Upgrade", the release
   section of `docs/README.md`.

### Questions for the human

1. Is bridle-ui's repo private? If so, `release.yml` needs a read-only deploy key or token as a
   secret. OK to add one?
2. Bundled (A1, recommended) or a separate asset (A2)?
3. Is "UI changes reach release machines only with the next bridle release" acceptable, with a
   pin bump before each tag? (It is what "tied to the bridle version" implies.)
4. Should the dev machine stay on its clone and `install-ui` (recommended), with the daemon not
   touching `~/.bridle/ui/` unless `self_upgrade = "release"`?
