# W23a workshop plate: W13/W14 drawing update acceptance recipes

**Scope:** Follow-up to the merged [workshop plate oracles](../../oracle/W23a-workshop-acceptance.md)
from #4962; coordinated with [#4892](https://github.com/tangency-labs/solidcraft/issues/4892)
and [CamWight's invitation](https://github.com/tangency-labs/solidcraft/issues/4892#issuecomment-6471076927).

These are **four declarative end-to-end test recipes** in `cases.json`, **not**
new W13/W14 production code and **not** passing executable Drawing tests.
The source design states are already defined analytically in oracles 84, 86
and 87 (with 85 retained as a width-change cross-check). Do not duplicate
those oracle geometry files or their reference-value calculations.

## Intended user-facing behaviour

On a 100 × 80 × 20 mm workshop plate, `Bore1` at (20,20) and `Bore2`
at (60,50) are separate through-hole features with Ø10 each. Their centres
are exactly 50 mm apart. A future A3 landscape, 1:2 drawing must show
**model-space dimensions** Ø10 and 50 mm, not the projected Ø5 and 25 mm
paper-space sizes. Paper-space label positions are *authored* and should
survive all model refreshes.

| Acceptance recipe | Source transition | Required Drawing assertions | Pending boundary |
| --- | --- | --- | --- |
| `W23A-E2E-01` | `84 → 86`, `parameters.change(bore1_d, 12 mm)` | Existing Bore1 diameter measure changes from Ø10 to Ø12, *same authored dimension identity*; Bore2 unchanged; labels retain their paper positions | R2/R3 + R5 + U2 + W19/W20 |
| `W23A-E2E-02` | `84 → 87`, `timeline.delete(Bore1)` | Bore1 dimension becomes **lost reference**, keeps last value Ø10 *as historical*, never silently targets Bore2; centre-to-centre dimension also becomes lost; Bore2 remains valid | R2/R3 + R5 + U2 + W19/W20 |
| `W23A-E2E-03` | `84 → 86/87 → 84 → 86/87`, source-design undo/redo | Correct active/lost states return, no accidental drawing undo, no changes to authored dimension IDs or placements | R5 + U2 + W19/W20 |
| `W23A-E2E-04` | Persist and reopen in each of `84`, `86`, `87` | Same actual drawing dimension identities, source references, last-known values, and label positions; no silent repair of lost reference | R5 + U2 + W19/W20 + storage integration |

**Important distinction:** The oracle's `StockSketch` construction-circle dimensions
are **not** technical drawing dimensions. Any names in `cases.json` such as
`diam_bore1` are **test-local symbolic handles**, never invented production
IDs, serialized fields or APIs. Once the real Drawing APIs exist, tests must
capture and compare IDs returned by them rather than hard-code these labels.

## Real command boundary versus pending Drawing boundary

The model edit commands in `cases.json` match the existing engine's JSON
calling convention, checked against its native tests:
`parameters.change {"name":"bore1_d","expression":"12 mm"}`,
`timeline.delete {"feature":"Bore1"}`, `edit.undo {}` and
`edit.redo {}`.

The *oracle recipes* (declarative model fixtures) are not automatically
open engine sessions. Binding them into a real command replay first requires
a validated loader/fixture setup. The `source_edit` steps below are intended
real engine commands, **not evidence that a complete replay has run**.

Drawing operations such as placing a view, authoring a diameter dimension,
reading a reference's status, and publishing a refreshed scene are described
as expected semantics only. **Do not turn them into fake CLI commands or
pretend an assertion passed.** R5 defines stored drawing references; U2
defines the update/publication boundary; W19/W20 provide the relevant
dimension behaviours. CamWight will link the actual APIs on #4892 as the
slices land.

## Isolation and invariants

1. **Start each recipe from oracle 84**, not from the previous recipe's
   modified design. The saved drawing is paired with that design; source
   model commands are issued in the **design's** session, drawing label
   edits in the **drawing's** session.
2. **No closest-object fallback:** after deleting `Bore1`, even a new
   geometrically identical bore must not acquire its reference. The
   `Bore2` feature keeps the *original* identity, name and XY location.
3. **Out-of-date is not lost:** with deferred updates, the existing drawing
   is `out of date` until Update; after publishing, vanished references
   are `lost reference`. Automatic update must not add an authored undo step.
4. Check both drawing-open and drawing-closed paths once supported. No
   old computation may publish after an edit, undo or close. Those
   publication-order cases are already being prepared separately under
   #5430; **do not duplicate** that contribution here.
5. Save/reopen tests must preserve *actual captured* authored dimension IDs
   and manually chosen label coordinates, not only numeric output.

## Validation and honest status

- The existing model oracle fixtures already have their own analytic
  `expected.json` files. Run `python3 validate.py` for a
  **static consistency check only** (case links, parameter semantics and
  expected analytic geometry from those files).
- `cargo xtask oracle` validates the 3D models, **not** automatic drawing
  updates. No new heavy CI/Windows batch is requested with this draft.
- Every runtime Drawing acceptance row remains **NOT RUN** until owner-
  approved R5/U2/W19 API bindings have been committed, reviewed and
  executed. Until then, this PR is an **early reviewable contract**, not
  functional completion.
