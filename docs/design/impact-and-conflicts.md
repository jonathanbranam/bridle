# Impact and conflicts

## The impact registry

Every planned task declares its impact:

```
bridle impact set tw-7fa2 \
  --modify s-b310 --add-under r-7fa2 --remove s-11c0 \
  --files 'client-watch/**' 'packages/ratings/**'
```

**Built:** the registry only. `bridle impact set` replaces a task's whole declared impact
(`POST /v1/tasks/{id}/impact`) and `bridle impact show` prints it. It is a field on the task
record, stored in the state branch frontmatter (storage.md) and restored by `bridle rebuild`. Only
an open, planned or claimed task can be set; ids are checked by shape (`r-`/`s-`/`g-`/`a-` plus
hex), not for existence. Diffing actual against declared impact is not built.

**Built: `bridle impact check [--specs DIR] [--json]`** (`POST /v1/impact/check`, pure
`bridle-daemon/src/impact.rs`). It compares every planned or claimed task that declared an
impact, pairwise, over `modify`, `remove` and `add-under` ids, and exits 1 if any overlap is a
conflict. It builds the first four rows of the table below; the `git merge-tree` row is not built.
- A scenario in `modify`/`remove` of both tasks: conflict.
- Two tasks touching the same requirement (an `r-` id, or the parent of an `s-` id), other than
  through a scenario already reported as a conflict: warn.
- Both touching a capability but no common requirement: info.
- File globs overlap when the literal text before the first wildcard (`* ? [ {`) of one is a
  string prefix of the other's. Coarse on purpose: an early warning, not a proof.

The CLI reads `--specs` (default `design/specs`) with `bridle-spec` and sends the id -> (requirement,
capability) map; the daemon never reads specs. Best effort: a missing or unparseable directory or
file yields no map, so `s-` ids only match themselves, `r-` ids still match, and there is no info level.

`bridle impact check` compares every in-flight task and reports:

| Overlap | Level | Action |
|---|---|---|
| Same scenario modified/removed by two tasks | **conflict** | open a `conflict` thread between the claimants |
| Same requirement, different scenarios | warn | notify both; usually fine |
| Same capability, different requirements | info | nothing |
| Same files | warn | early warning (research 09 tier 1) |
| Real textual conflict from `git merge-tree` | **conflict** | same as the first row (research 09 tier 2) |

**Declared impact is checked against actual impact.** When a branch changes,
bridle diffs the spec ids and file paths it touched and flags anything the task
didn't declare. An agent can't avoid a conflict by under-declaring.

## The conflict protocol

1. Bridle opens a `conflict` thread between the two claimants (or the manager,
   for unclaimed tasks) and injects it into both sessions.
2. They decide between them, and one records the outcome:
   - `bridle conflict resolve C12 --compatible "…"`: not a real conflict; the
     reason is recorded.
   - `bridle conflict resolve C12 --order tw-7fa2,tw-a9d0`: adds a `blocks`
     edge. The blocked task's claimant writes a handoff note, releases or
     parks the claim, and takes other ready work.
   - `bridle conflict resolve C12 --merge-into tw-7fa2`: one task absorbs the
     other's scenario change.
3. If they disagree, or the conflict is a product question rather than an
   ordering question, it escalates to the manager, and to the human only if it's
   a product question.
4. **After a merge**, bridle sends `system: spec changed under you` to every
   in-flight task whose impact overlaps what just merged, so those agents rebase
   and re-read before building on stale text.

**Built: steps 1 and 2.** `impact check` (`POST /v1/impact/check`) opens one conflict per
conflict-level overlap, and each overlap only once, resolved or not (unique on the two tasks,
kind and key). Conflicts live in the SQLite `conflicts` table only: a conflict is not a
task thread and has no state-branch record, so `bridle rebuild` drops them and the next
`impact check` reopens any still-real overlap. Each task's claimant gets one `system`
message naming the other task and the overlap, and a note lands on both task threads. For
an unclaimed task the message goes to the running `manager` agents. `bridle conflict list`
and `resolve` (`GET /v1/conflicts`, `POST /v1/conflicts/{id}/resolve`) do the rest:
`--order A,B` adds the `A blocks B` edge, and `--compatible` and `--merge-into` record only
the outcome; the agents make the change. Not built: escalation (step 3), the `git merge-tree`
conflicts, and the handoff for the blocked task's claimant.

This replaces the same-spec pile-up rule. Serialisation happens only when two
tasks actually collide, and the agents involved decide the order.
