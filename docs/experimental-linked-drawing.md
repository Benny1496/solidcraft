# Experimental model-linked XY drawing preview

This is a development-only first slice for testing model-linked 2D manufacturing drawing behavior. It is **not** a Drawing workspace, a persisted sheet, a standards-certified projection, an HLR engine, or a replacement for the published drawing roadmap (#860). All work is isolated on the fork's `feature/browser-automation` branch.

## Primary product goal: native, upstream-ready 2D manufacturing drawings

**This is the governing objective of this experimental work**, not merely
increasing test counts or shipping a stand-alone preview. Build a usable,
professional **2D technical drawing workspace inside the existing SolidCraft
application and architecture** so that the fork maintainer can demonstrate it
to the original SolidCraft developer and propose it for inclusion upstream.
It must also be practically useful to the contributor and other end users.
The contributor wants to present the first working implementation to the
developer as early as feasible, without trading away reliability.

**Integration requirement — non-negotiable for the proposed deliverable:**
- Implement inside the normal SolidCraft repository, engine/model, egui
  interface, file/project workflows and commands; do not create a separate
  CAD application, external companion, or mandatory independently launched
  service.
- Keep changes isolated and reviewable in \`Benny1496/solidcraft\` on
  \`feature/browser-automation\`. Do not change upstream or fork \`main\`
  without an explicit separate decision. Aim for a conventional upstream
  pull request with tests, docs, and a reproducible build. Do not assume the
  original maintainer has accepted it or that zero upstream changes will
  be required; architecture and code review decide that.
- Model geometry is the source of truth. Build credible orthographic
  projections (top/front/side), actual outlines and holes, reliable manually
  placed dimensions, a persistent drawing/sheet, and practical print/PDF
  output. Add real curves, visible/hidden edge treatment, and eventually
  sections, tolerances, scales and title blocks without misleading geometry.
- Changing the 3D body must mark its saved linked drawing as stale, including
  when the drawing window was closed. On reopening show an explicit
  **accept or decline** update dialog. Declining preserves the previous
  drawing; accepting updates referenced geometry and measurements without
  silently relinking to wrong faces or losing user-arranged dimensions.
- Give the maintainer a compelling **working, user-testable end-to-end demo**,
  not only command results: create a simple 3D plate with bores/fillets,
  open its 2D workspace, switch views, place dimensions, save/reopen,
  edit 3D, handle the confirmation dialog, and export a useful drawing.
  Demonstrate with screenshots, clear documentation, reproducible steps,
  regression evidence and a reviewable, native-code change set.

**Prioritization rule:** finish this first reliable, integrated vertical
slice as quickly as technically reasonable, and expand coverage around
actual user workflows. Do not defer a usable GUI indefinitely in favor of
ever larger numerical test matrices. Never claim a finished manufacturing
drawing or upstream acceptance before it is demonstrated and verified.

**Current boundary (22 October 2026):** the linked XY preview remains
experimental and contour samples are not certified HLR/silhouettes.
The standard distributed SolidCraft release does not automatically
include fork development-branch changes. The full native drawing workspace
and production-quality sheets/exports remain to be implemented.

## First use

Create or open a 3D plate and run this command through the browser/CLI/engine command API:

```json
{"command":"drawing.preview","params":{"body":"Plate"}}
```

The reply contains `{status:"created",requires_confirmation:false,drawing}`.
`drawing` holds:

- A versioned `solidcraft-drawing-preview/1` record.
- A **bounding rectangle envelope** of the measured 3D body (not a certified silhouette).
- Measured XY circles from vertical cylindrical B-rep faces, with physical diameters; duplicate split cylinder faces are consolidated.
- Overall X and Y dimensions in millimetres.
- An SVG preview with width/height in **physical millimetres** (model dimensions are 1:1 in the SVG coordinate system).
- Positions of the two manually movable overall-dimension labels and an explicit source geometry signature.

The SVG can be displayed by a future drawing viewport or exported by a dedicated adapter. This command does not itself save a file or open a UI window.

## Updating after the model changes

Store the returned `drawing` as a snapshot **outside** the model, then edit the 3D model. Query again with the original drawing:

```json
{"command":"drawing.preview","params":{"body":"Plate","previous":{"format":"solidcraft-drawing-preview/1","body":"Plate","source":{"...":"..."}}}}
```

Pass the **complete** original drawing object from the first reply, not the abbreviated example above.

If the measured source geometry differs, the command returns:

- `status:"stale"`, `requires_confirmation:true`
- `message:"The original 3D model has changed. Update this drawing?"`
- `drawing`: the **unchanged** original drawing
- `proposed`: the new drawing, ready for review

No changes are applied automatically. After the operator confirms, call `drawing.preview` again with that same original `previous` drawing and `accept:true`. The result is `status:"updated"` and its `drawing` contains the measured new geometry. Overall-dimension label positions remain as specified in the previous drawing. With no model change, the command returns `status:"current"`.

**No actual dialog is implemented yet.** The `stale` and `requires_confirmation` values form the backend contract from which the future UI can show the German confirmation window when opening a drawing.

## Supported subset and cautions

- Works on a body with a valid 3D bounding box; models with a different outline (e.g. round, filleted or C-shaped) still show a *bounding envelope*, **not** their true contour.
- Cylinders must be aligned with the Z axis. Side bores currently return an error rather than being misrepresented.
- A cylindrical exterior boss is not automatically classified as a drilled hole; the API deliberately calls the objects `cylinders`.
- Cylindrical faces currently lack sufficient certified provenance for stable hole callouts. We deliberately **do not persist positional hole-number labels** because adding/removing a hole could silently reattach one to the wrong bore. Later work must tie drawing dimensions to stable feature and face identities, with warnings for missing references.
- The measured source signature is rounded to 0.001 mm. Changes smaller than that threshold are outside this prototype's stale-detection guarantee.
- The `previous` snapshot is passed as command input; it is **not yet stored in a `.solidcraft` design**. Saving, revisiting, and acknowledging a drawing is the next integration layer.
- No standard drawing sheet/title block, print tolerance verification, hidden-line removal, associative diameter/centre distance callouts, or 2D drawing workspace exists yet.
- The independent `file.export_view_dxf` browser test is kept isolated because it previously blocked Chrome DevTools for 240 seconds.

## Automated verification

`xtask/src/browser_drawing_preview_test.js` is run with the existing compiled SolidCraft browser application by `cargo xtask web-api-test --site dist/web`.

The test creates a 150 x 80 x 10 mm plate with four Ø8 holes; verifies the SVG/geometry/dimensions; changes the 3D length to 180 mm and a linked bore to X165; requires explicit confirmation; verifies the original snapshot and manually placed overall-dimension labels survive; and ensures that drawing commands do not mutate the CAD timeline. Do not claim success until the new workflow run has passed.

## Experimental Rust/egui drawing window (fork branch only)

The preview now has a first on-screen Rust UI connected to **INSPECT → DRAWING**. A user can select a body, create a versioned provisional drawing snapshot inside the model metadata (`org.solidcraft.experimental_drawing`), view the rectangle/bore geometry with dimension lines, and drag the two overall-dimension labels. The sketch snapshot survives design save/open using the existing metadata mechanism; it is **not** yet a first-class drawing sheet.

Opening the panel re-reads the saved snapshot and checks the current model via `drawing.preview`. A stale drawing opens an explicit German confirmation window with **Zeichnung aktualisieren** and **Alte Zeichnung beibehalten**. Accept updates the stored snapshot only after explicit confirmation; keeping the old version leaves it unchanged and suppressed for the current model revision. The UI rechecks the status after later revisions. Changing dimensions while the window remains open is monitored through the engine revision.

The browser suite additionally tests saving the snapshot in document metadata, changing a parameter, detecting stale status, accepting and re-storing the proposed geometry, and preserving a moved dimension-label position.

This UI is **unverified until a GitHub Action builds the new code**. The workflow currently checks browser command semantics, not human-screen interaction or screenshots. A future GUI scenario must exercise opening the panel, clicking both dialog choices, and dragging the label. Do not mistake this bounding-box XY pilot view for full technical drawing standards, persistent native sheet entities, or certified bore dimension associations.

## First automated on-screen interaction tests (pending CI validation)

The existing `crates/ui-egui/src/scenario.rs` harness renders the **actual egui application** in an offscreen context and drives controls as a human would. Tests are now embedded in `crates/ui-egui/src/drawing_ui.rs`:

1. Start the on-screen Drawing window via the `drawing.preview` command, click the **Zeichnung im Modell speichern** button, modify the 3D parameter, verify the German popup appears, click **Alte Zeichnung beibehalten**, verify the old number is preserved, reopen, click **Zeichnung aktualisieren**, and check the new number is saved and no warning remains.
2. Save a drawing, drag the real overall X dimension's screen-space interaction target, and verify its stored label coordinates change while the physical model remains unchanged.

A CI step has been added to the manually triggered SolidCraft Browser Smoke workflow:

```sh
cargo test -p solidcraft-ui-egui drawing_ui::tests --lib -- --nocapture
```

This runs alongside the pre-existing Rust-to-WebAssembly build and browser API tests. **These newly added GUI tests have not yet passed in CI**; run a new manual workflow, inspect the result and fix any failures before claiming the on-screen interaction is verified.

A pointer-movement bug identified during review was also corrected: the drag handler applies the *per-frame* pointer delta, not the cumulative delta from the beginning of the drag. The test harness now receives a stable named interaction target for the overall dimensions. Opening a stored sheet whose source body was deleted now selects a fresh, available body instead of sending a mismatched snapshot back to the preview command.

## Evidence reconciliation at 21 October 2026

The authoritative last verified GitHub run is [#12](https://github.com/Benny1496/solidcraft/actions/runs/39656227739): `success`, browser API and plate checks pass, all ten optional CAD scenarios pass, including model-linked drawing freshness and stored metadata. Earlier run [#11](https://github.com/Benny1496/solidcraft/actions/runs/39655360907) failed Rust UI compilation and was corrected before successful #12.

The separate Home Assistant project-status file could not be re-read at this stage because its connector permission was rejected. The latest **confirmed** update to that file was the note on failed run #11 and its compiler fix; run #12 and subsequent on-screen UI test additions have **not** been confirmed to be present there. This GitHub document therefore records the reconciliation facts pending an authorized recheck, without claiming the HA copy is already synchronized.

## Consolidated drawing regression batch (next manually triggered workflow, not yet validated)

Following successful workflow #13 (WebAssembly build, browser CAD suite and 2/2 real egui interaction tests), the experimental branch now includes a larger **single-run** verification batch:

- **15 named browser CAD scenarios** (five new: `drawing-revision-matrix`, `drawing-bore-matrix`, `drawing-bad-inputs`, `drawing-snapshot-undo`, `drawing-batch-bodies`), plus the original four-hole plate and browser command API smoke test. The new scripts cover repeated length/width changes, undo/redo and second confirmations; three dissimilar diameter bores and changed inset; twelve rejected malformed drawing API inputs; metadata no-op/undo/redo/remove/restore; and four plates of different proportions with 0/2/4/6 bores.
- **Six native egui interaction tests** in `crates/ui-egui/src/drawing_ui.rs`, run via `cargo test -p solidcraft-ui-egui drawing_ui::tests --lib -- --nocapture`. Four new tests cover warning reappearance after undo/redo, moving the Y dimension and reopening, empty-design recovery, and resetting the transient drawing on a new document.
- **Six native engine drawing tests** in `crates/engine/src/cmd/drawing_preview.rs`, run via `cargo test -p solidcraft-engine drawing_preview --lib -- --nocapture`. Additional tests cover malformed snapshot rejection, label positions independent of model geometry, invalid or changed measured signatures, and a real `.solidcraft` save → reload → 3D-edit → stale-detection roundtrip.

The entire batch is added to the **existing manual** `.github/workflows/solidcraft-browser-test.yml`. As before, there is **no automatic scheduled/push run** and no modification to upstream. A new manual GitHub Actions run must prove the tests pass. If it fails, read the individual scenario or Rust test name and correct the actual cause; do not count unexecuted or failed tests as successful.

**Note on HA status documentation:** The latest accessible account of `/share/GitHub SolidCraft/PROJEKTSTATUS.md` is still the earlier verified content including failed run #11 and its correction. The Home Assistant connector denied access on the subsequent recheck. GitHub's documentation provides the current reliable change history until the user authorizes access again. Do not claim the HA copy is synchronized without rereading it.

## GitHub Actions run #14 — partial failure isolated and fixed (21 October 2026)

- [Run #14](https://github.com/Benny1496/solidcraft/actions/runs/39669359298) (commit `5f7be21b23127f1c151a7c87b085a33a1923e122`) finished **failure**.
- WebAssembly web build: **PASS**. Browser API smoke and original four-hole plate: **PASS**. All **15/15 named browser CAD scenarios**: **PASS**. All **6/6 actual egui drawing window UI interaction tests**: **PASS**.
- Of **6 native Rust engine drawing-preview tests**, **5 PASS**, **1 FAIL** (`cmd::drawing_preview::tests::drawing_stays_current_until_model_changes_then_requires_confirmation`). The real .solidcraft file save/reopen test **passed**.
- Precise cause of the failure: the native test used direct JSON equality to compare the normalized f64 coordinates `[99.0,-22.0]` against literal integer JSON coordinates `[99,-22]`; mathematically identical but different `serde_json::Number` variants. It was a test assertion error, not a CAD geometry/behavior failure.
- Fixed on branch in commit `5b22551c3ea404cd42e80226d12885fe3277992f`: now assert `as_f64() == Some(99.0)` and `as_f64() == Some(-22.0)`.
- **Not yet confirmed by a fresh CI run.** Trigger a **new manual workflow** on `feature/browser-automation` (expected run #15); do not click "rerun" on #14 since that would test the old commit.
- HA `/share/GitHub SolidCraft/PROJEKTSTATUS.md` remains unverified because the Home Assistant connector declined access on the last attempt. Keep this GitHub note as the verified source until access is restored.

## Verified success: GitHub Actions run #15 (21 October 2026)

- [Run #15](https://github.com/Benny1496/solidcraft/actions/runs/39675912802), SHA `b1121e47f8164d7c074e0a2e357e7c063337c5a5`, completed **success** (21:48:12 UTC). GitHub branch: `feature/browser-automation`.
- WebAssembly production build **PASS**; browser command-API smoke **PASS**; reference 150 × 80 × 10 mm four-hole plate **PASS**.
- **15/15 named browser CAD scenarios PASS**, including all five recently added matrices: multi-axis edits and undo/redo, three bore diameters with dependent positions, twelve rejected invalid requests, stored metadata undo/redo, and four plate sizes with 0/2/4/6 bores.
- **6/6 actual egui drawing window user-interaction tests PASS**, covering acceptance/rejection dialog, X and Y dimension label dragging, undo/redo warnings, empty design, switching to a new design.
- **6/6 native engine drawing tests PASS**, including actual `.solidcraft` disk save/reopen and stale detection, provenance signature checks, and the corrected numerical label-coordinate assertion.
- **28 explicitly counted test cases plus the browser command-API smoke** all pass; count is 15 named scenarios + reference plate + 6 GUI + 6 engine. The browser app and scripts passed CI compilation/syntax.
- One non-fatal warning concerns GitHub's Node.js-20 deprecation of `actions/upload-artifact@v4`, unrelated to SolidCraft's drawing behavior.
- This success confirms the experimental XY bounding-box drawing and its current behavior. It **does not** yet supply exact projected outer contours, native dimension references to individual holes, a professional drawing sheet, title block, precision tolerancing, or validated DXF exports.
- HA project status file `/share/GitHub SolidCraft/PROJEKTSTATUS.md` was not updated/reverified because the last request was denied by the connector. The GitHub documentation records the current verified state until access is explicitly reauthorized.

## Step toward 3D→2D manufacturing measurements (pending CI)

**Verified predecessor:** workflow #15 passed 15/15 named browser scenarios, the reference four-hole plate, 6/6 drawing UI interactions, and 6/6 drawing engine tests. The current branch adds code and tests but **has not yet passed its own CI run**.

The `drawing.preview` command now derives read-only `drawing.geometry.cylinder_dimensions[]` from real measured vertical B-rep cylinder surfaces, not copied raw dimensions or a sketch feature's unverified parameters. Each entry contains:
- `center_mm: [x,y]` in world XY;
- `diameter_mm`, derived from twice the measured cylinder radius;
- `x_from_left_mm` and `y_from_bottom_mm`, measured from the body's XY *bounding envelope*, in millimetres;
- `reference_status: "unverified"` (crucial: **NOT a permanently stable hole identity**).

The 2D egui preview now paints a temporary `Ø` diameter leader by each circle, plus a read-only measured-dimensions table listing the provisional cylinder number, Ø, X, and Y. The table and drawing are visible after saving; existing saved preview snapshots are refreshed for display when current, and the old drawing remains visible when stale until the operator confirms. Existing tests for stale warnings and saved overall-dimension positions remain.

### What is actually associative?

The measured **values** are derived anew from current 3D geometry (and after a model edit, appear as `proposed` until accepted). In this limited sense, the 3D-derived measurements *follow the model*. They are **not yet true manufacturing drawing dimensions anchored to persistent feature identities**. Their ordinal table numbers change when geometry changes; they have no persistent dimension layout, and exterior cylinders/bosses may appear. The XY rectangle is still a bounding-envelope approximation, not a certified silhouette. Do not call this a completed Fusion 360-style drawing workspace. Work remaining includes stable model-reference provenance, selectable and editable dimensional annotations, measured centre-to-centre spacings, angles and radii, proper sheets with title blocks, views/projections and print/DXF.

### Consolidated verification suite, next workflow

The existing manual workflow now includes **19 named browser CAD scenarios**, plus its reference plate and browser command-API smoke, **8 real egui interaction tests**, and **8 drawing engine Rust tests** (36 counted cases plus API). Four new named browser scenarios exercise:
1. Four parametrically placed Ø8 cylinders whose plate width and diameters change (Ø8→Ø10, 150→180 mm): exact 3D-measured Ø/X/Y transfer, unchanged old drawing, explicit accepted preview.
2. A translated plate with negative world coordinates: X/Y offsets are measured from the actual left/bottom envelope, rather than treating the global origin as zero.
3. 0→1→2 new bore cylinders with undo/redo: stale detection and preservation of unconfirmed drawing values.
4. A twelve-cylinder matrix with Ø4/Ø6/Ø8: real measured diameter, centre offsets and consistency between drawing circles and measured data.

The additional egui tests verify that the Ø/X/Y measurement table is actually shown and that a second 3D cylinder is withheld until the user clicks to accept the stale drawing. The two new native engine tests check translated XY coordinate offsets and accepted-vs-pending dimensions after adding a cylinder.

All added scripts are included in the existing browser suite; the Rust GUI/engine suites are run in the same workflow. A manual GitHub Actions run on `feature/browser-automation` must still validate the new code and tests before claiming success. The Home Assistant project-status file remains **unverified / unsynchronized** because its connector last denied access.

## Consolidated non-fail-fast CI: run #17 findings and next-run policy (22 October 2026)

[Manual workflow run #17](https://github.com/Benny1496/solidcraft/actions/runs/39719878000) on branch `feature/browser-automation` SHA `ff82b82dce335b7b3b544de4b8ad79c61b62c74e` finished **failure**, but importantly all independent suites continued:
- **19/19 named browser CAD scenarios PASS**, including the four new 3D-derived Ø and XY dimension scenarios; browser API + four-hole plate **PASS**.
- **8/8 drawing-engine tests PASS**, including actual model-derived cylinder diameter and XY edge offsets.
- **6/8 drawing GUI interaction tests PASS**, two regression failures: `drag_overall_dimension_in_real_egui_window_saves_new_label_coordinates` and `moving_y_dimension_preserves_placement_after_closing_and_reopening`. The saved label coordinates remained unchanged after egui drag.
- Cause: current-snapshot drawing window display favored the `draft` (recomputed drawing) over the mutable `snapshot` on **each frame**, resetting the in-progress manually dragged label before persistence. Fixed the display-selection logic to prefer stored `snapshot` for interaction and merge the newly added provisional `cylinder_dimensions` display field from the `draft` only for legacy snapshots that lack that field. Existing stale drawing remains unchanged until confirmed. **Fix awaits a new manual CI validation**.
- Added a ninth native egui regression test: a saved older drawing without the new measured-dimension table must show current Ø data **without silently rewriting metadata**, must retain its label position upon reopening, and must still persist dragged dimension text.
- The `xtask/src/web_offline_test.rs` browser suite now collects errors from the initial API smoke test and four-hole plate as well as all 19 named browser scenarios, reports each independently, then fails if any check was unsuccessful. A fatal unavailable browser/session remains a genuine infrastructure blocker.
- The GitHub Actions workflow now collects each independent **JavaScript syntax, WebAssembly build, browser API/CAD, egui GUI, drawing engine** test group with `continue-on-error: true`, and intentionally runs later suites with `if: !cancelled()`. Browser CAD checks run only if the WebAssembly build succeeded. A final `GITHUB_STEP_SUMMARY` table presents all five statuses and a final step **fails the overall GitHub run** if **any** group failed or was skipped. This preserves all errors without producing false green workflow status.
- The consolidated suite for the next manual run: **19 named browser scenarios + reference plate + 9 GUI + 8 engine = 37 test cases**, plus API smoke. The next run has **not been started or validated** as part of this documentation update.
- Changes are limited to `Benny1496/solidcraft` branch `feature/browser-automation`. No modification to upstream or the user's local computer. HA project status remains unsynchronized, since previous read/write connector requests were refused.

## Verified green run #18: 38/38 independent browser/UI/engine checks (22 October 2026)

[Manual GitHub Actions run #18](https://github.com/Benny1496/solidcraft/actions/runs/39724896543) executed on `feature/browser-automation` at commit `9314e521a4134074ba9fb3f0821aba2a4718c20b`, completed **SUCCESS** at `2026-10-22 05:35:26 UTC`:
- Browser API smoke and reference four-hole plate: **2/2 PASS**.
- Named browser CAD tests: **19/19 PASS**, including all new 3D-derived cylinder Ø/X/Y measurement cases.
- Real egui window tests: **9/9 PASS**, including regressions for X/Y dragged dimension labels and the new legacy saved drawing case.
- Rust engine drawing tests: **8/8 PASS**, including drawing save/reopen, geometry signature validation, stale confirmation and 3D-derived measurements.
- **38/38 total independent checks passed** (21 browser checks + 9 GUI + 8 engine). WebAssembly build and JavaScript syntax tests passed. All five CI report groups completed successfully; the concluding aggregated summary step reported **0 nonpassing test groups**.
- New permanent GitHub workflow behavior is **confirmed**: all independent checks use non-fail-fast evaluation and produce a consolidated GitHub Actions Step Summary with a final failing status only if any check failed/skipped. The previous run #17 exercised actual fail-and-continue.
- Scope caveat: cylinder labels in the preview are provisional and lack stable persistent per-hole association. Accurate projected outer shape, selectable persistent drawing dimensions, angle/radius and GD&T, multiple orthographic views and a formal title block remain to implement.
- The user-selected project documentation under Home Assistant is still **not verified updated**, due to previously denied connector access; GitHub documentation tracks validated progress instead.

## Selected face-linked manufacturing dimensions — next large manual CI block (unverified; 22 October 2026)

The previous manual [GitHub run #18](https://github.com/Benny1496/solidcraft/actions/runs/39724896543) passed **38/38 independent checks**. The current feature branch now adds the next substantial increment; **do not claim these changes pass until a new GitHub Actions run succeeds**.

### Implemented experimental features

- The existing `drawing.preview` engine inspects real model faces, records `face_name_candidates` only for history-verified *exact* cylindrical-face names (`model.faces.faces[].exact === true`), and exposes one `eligible_face_name` per cylindrical group only when unique among all other detected cylinders. Geometric shape, cylinder centre and measured diameter are still obtained from the B-rep, not a user-declared raw value.
- New read-only command **`drawing.annotation.add`** accepts `body`, an existing `previous` drawing, a selected verified `face_name`, one dimension `kind` (`diameter`, `x_from_left`, `y_from_bottom`), and an optional manual `position_mm`. It refuses invalid, duplicate or nonunique/unverified face references and requests from stale drawings. It returns a newly annotated drawing; egui persists it via the existing versioned document metadata key (`org.solidcraft.experimental_drawing`).
- Each annotation carries a unique `linked_cyl_N` ID, history-verified face name, dimension kind, derived numerical measurement, saved centre and independent draggable manual text position. It is re-resolved **by face name, never cylinder array index or distance-to-another-centre**. If the cylinder face is removed, uncertain or its name cannot be resolved uniquely, the annotation remains in the saved drawing with `reference_status: "missing_reference"`, `value_mm: null` and its last known historic value. The stale-model confirmation dialogue must be accepted before the saved drawing is updated; rejected changes preserve the original.
- The real 2D drawing window offers **“Ø setzen”, “X setzen”, “Y setzen”** controls for eligible cylinder rows after the drawing is saved and current, renders callout leaders and user-draggable text, and warns of lost references. The SVG snapshot also draws actual placed Ø/X/Y callout leaders; a historic unresolved dimension receives a visible question mark instead of being passed off as a valid new measurement.
- The current `solidcraft-drawing-preview/1` metadata format is retained for backward compatibility. Older saved drawings without `annotations` continue to work, and previous dimensions and exterior text positions remain.
- Important limitation: unique exact cylindrical face history is a prerequisite; a cylinder without it is **not selectable**. Cylinder surfaces are not yet certified as drilled *holes* (they may be bosses), and there is no certified full 2D silhouette, orthographic front/side views, tolerances, hole diameter annotations from threaded/countersunk features, GD&T, title block or manufacturing-grade print/DXF output. The X/Y offset origin remains the XY bounding-box envelope, not a user-selected datum face.

### Large consolidated next-run test scope

The existing manual `.github/workflows/solidcraft-browser-test.yml` already has **five independent test groups** (JavaScript syntax, WASM production build, browser CAD/API, actual egui GUI, native Rust engine), *non-fail-fast* test continuation, individual logged errors and a final truthful PASS/FAIL summary. New scope is now:

- **23 named browser CAD scenarios** (19 previously successful + 4 new), plus **two baseline browser checks** (API smoke, four-hole plate). New browser cases: (1) a selected exact-name hole face followed through diameter and position parameter changes and manual SVG label placement, (2) independently chosen Ø/X/Y annotations on the same exact source face and duplicate prevention, (3) deletion of the specifically selected second hole, preserving a marked unresolved annotation rather than silently relinking to the surviving equally sized hole, (4) eight invalid/double/stale annotation requests rejected.
- **11 native real egui GUI tests** (9 prior + 2 new). These click the actual “Ø setzen” widget and move a linked dimension by mouse; and verify that pending 3D changes require user confirmation without modifying the stored drawing.
- **14 native Rust engine drawing tests** (8 prior + 6 new), covering unique face-name association, historical-value persistence, missing/ambiguous references, malformed and duplicate annotation data, real-model parameter changes and an actual `.solidcraft` disk save/reopen of the linked drawing.

**Total 50 independent checks:** 25 browser checks (23 named scenarios + 2 baselines), 11 real GUI tests, and 14 native engine tests. The workflow will still run independent suites and report all failures in its final GitHub Actions step summary. A new manual run must execute them on `feature/browser-automation`. These files were authored directly in the user's fork using GitHub; no local computer work or upstream modifications.

### Home Assistant project status

The requested Home Assistant project file is `/share/GitHub SolidCraft/PROJEKTSTATUS.md`. Previous Home Assistant connector access had been denied; this GitHub document remains the actual written progress record. **After the next run**, recheck whether the connector is accessible without retrying a previously explicitly refused tool path unless access is reauthorized. If reauthorized, reconcile the HA project status with the GitHub log and CI results; do not claim an HA update until its write succeeds.

## Verified success: named-face drawing annotations — Run #19 (22 October 2026)

[GitHub Actions Run #19](https://github.com/Benny1496/solidcraft/actions/runs/39731900004), commit `3c665607fa3a045a9293ee0c1ef330c4651e5286` on `feature/browser-automation`, finished **SUCCESS** at `2026-10-22 06:52:05 UTC`. The full execution logs confirm:

| Group | Result |
| --- | --- |
| Named browser CAD scenarios | **23/23 PASS** |
| Browser API smoke + real four-hole reference plate | **2/2 PASS** |
| Actual egui GUI drawing tests | **11/11 PASS** |
| Native Rust engine drawing tests | **14/14 PASS** |
| **Independent checks total** | **50/50 PASS** |
| JavaScript syntax / WebAssembly build / consolidated summary | **All PASS; 0 nonpassing groups** |

In particular, all four new browser scripts passed: `linked-face-diameter` (same exact face from Ø8 to Ø12 after model changes), `linked-face-dimensions` (three separate Ø/X/Y annotations), `linked-face-removed` (deleted 3D hole leaves a safely unresolved manual annotation) and `linked-face-rejections` (eight invalid/duplicate/stale requests refused). Their actual egui widget tests and Rust disk save/reopen tests passed too.

**This validates the implemented, expressly limited prototype behavior, not a finished manufacturing drawing suite.** The XY bounding rectangle is still only an envelope; real silhouette projection, free direct selection of arbitrary edges/points, midpoint-to-midpoint and angle/radius dimensions, robust all-topology provenance, title blocks, multi-view layout and production export remain open.

### Private project handoff

Project handoff and conversation notes are kept privately in the user's approved Home Assistant project folder, not in the public GitHub repository. This GitHub document contains technical implementation and test results only. Do not reintroduce a public `Chatlauf.md` or include private conversation notes in GitHub commits. The project's Home Assistant status documentation was updated and verified following passing run #19.

## Direct 2D cylinder selection and 60-check regression bundle (22 October 2026; **not yet CI-verified**)

**Verified predecessor:** [Run #19](https://github.com/Benny1496/solidcraft/actions/runs/39731900004) completed successfully with 50/50 checks. The changes described below are committed on `feature/browser-automation` but have **not** been validated by a new complete GitHub Actions run.

### Incremental UI feature

- An exact-history and unambiguously identified cylinder can be clicked **directly inside the existing egui 2D drawing preview**. The click selects the corresponding certified `eligible_face_name`; the selected circle receives a visible accent outline.
- A contextual editor offers **“Ø aus Auswahl”, “X aus Auswahl”, “Y aus Auswahl”**. Each action delegates to the already tested `drawing.annotation.add` Rust command, which creates a persisted, movable drawing annotation using the named-face identity. Duplicate dimensions are disabled.
- The circle hitbox remains usable for small projected diameters (clamped to 11–32 px radius) and its real mouse coordinates are exposed to the native scenario harness as `drawing:cylinder:N`.
- **Safety:** Circle interaction is enabled only after saving a nonstale drawing, and only if the 3D face's exact name is verified and unique. The transient selection is never saved in the drawing metadata; it can be cleared, is reset on body change / stale geometry, and is not retained on reopening the drawing.
- The pre-existing measurement table actions still work. No different 3D kernel, native drawing document type or global CAD viewport model-selection behavior was modified.
- Caveat: this is **direct circle selection only** (not arbitrary edge or line selection); detected cylinders are not certified as through-holes; real 2D silhouette, datum selection, orthographic view sets and production-quality dimensions remain future work.

### Extended manual verification

The same non-fail-fast manual GitHub Action now includes:

| Group | Previously verified | Newly prepared |
| --- | ---: | ---: |
| Browser API + reference plate + named CAD scenarios | 25 | **29** (27 named + 2 base checks) |
| Native egui drawing window mouse interactions | 11 | **15** (+4 real circle click / select / add / change tests) |
| Rust drawing core | 14 | **16** (+2 unique-provenance / face-order regression tests) |
| **Total independently reported checks** | **50** | **60** |

New browser scenarios: six cylinders with 18 independently set annotations; manually positioned Ø/X/Y labels retained through parametric 3D edits and explicit acceptance; rejection of five kinds of malformed/falsified annotation data; and real measured XY offsets of selected named faces on a plate with negative world-space coordinates.

Native egui cases: clicking an unsaved then a saved real bore, selecting and placing Ø, selecting the second of two cylinders for all three dimension types, safely blocking selection while a drawing is stale, and clearing/transient state after reopening. Core cases: only unique verified cylinder faces offered for 2D selection, and changing cylinder iteration order without rebinding existing annotations to a wrong face.

All four new JavaScript source files were **syntax-checked statically**, the scenario registration and workflow continuation rules checked in-source. **The new Rust code and real browser/GUI tests still require a new manual GitHub Actions run**; do not describe them as already passing.

The private project/chat handoff remains stored outside GitHub; only technical implementation and CI status are recorded here.


## GitHub Actions run #20: 56/60 checks, four GUI click-target failures (22 October 2026)

**This is the verified result for the implementation in commit `12b707a0c32b5c3d7f8c32427196f3fe77e15e3e` on `feature/browser-automation`:** [GitHub Actions #20](https://github.com/Benny1496/solidcraft/actions/runs/39744835825), `completed / failure`, 22 Oct 2026 08:47:53 UTC. The non-fail-fast workflow successfully ran **all** independent suites despite the GUI regression.

| Test group | Confirmed outcome |
| --- | --- |
| JavaScript syntax and production WASM build | **PASS** |
| Browser API smoke + reference plate + named CAD scenarios | **29/29 PASS** (27/27 named) |
| Real egui drawing-window interaction tests | **11/15 PASS; 4 FAIL** |
| Native Rust drawing-core tests | **16/16 PASS** |
| GitHub consolidated end-of-job summary | **FAIL**, exactly one non-passing suite |
| **Total independently checked** | **56/60 PASS** |

The four **new** egui tests fail before they can verify the intended 2D selection behavior, each with the same locator error:
`"drawing:cylinder:0" is not on screen: {"error":"not on screen: \"drawing:cylinder:0\"","ok":false}`
(and index `1` for the multi-hole test).

Test names:
- `clicking_2d_cylinder_selects_verified_face_and_adds_diameter_dimension` — `drawing_ui.rs:838`
- `direct_2d_selection_can_be_cleared_and_is_not_saved_as_drawing_metadata` — `drawing_ui.rs:946`
- `direct_2d_selection_chooses_exact_hole_not_neighbor_and_three_dimension_types` — `drawing_ui.rs:875`
- `unsaved_or_stale_drawing_cannot_select_a_new_2d_dimension` — `drawing_ui.rs:912`

**Highly likely root cause, verify before editing:** The new tests pass the handle name as a **raw string**, e.g. `h.step(&json!({"click":"drawing:cylinder:0"}))`. But `Harness::at` in `crates/ui-egui/src/scenario.rs` treats arrays as physical XY pairs and forwards nonarray values to `ui.at`, whose documented handle locator is a **structured object**, `{"handle":"drawing:cylinder:0"}`. The preview already publishes each target with `scenario::publish_target("drawing:cylinder:N",rect)`, and the initial test confirms `handle_at("drawing:cylinder:0").is_some()`. Thus the failure is consistent with an incorrectly shaped locator in the tests, **not proof that cylinder hit targets are absent**. Resolve by replacing the four new click steps with the structured handle syntax or with explicit `handle_at(...)` coordinates as used by passing drag tests; verify every occurrence (including `drawing:cylinder:1`). If they still fail, inspect `ui.at`, mouse interaction rect geometry, clipping and egui overlays. **No code fix has been made for this failure in the current handoff.**

Scope/caveats: The compiled WASM and Rust core pass, and 11 previous GUI tests pass. However the actual mouse-driven direct-cylinder-selection workflow is **not yet validated**, until those four tests pass. Preserve all 60 checks and the final red status for any remaining failure. A **new manual GitHub Actions run** is required after the code correction; a rerun of the old SHA is not sufficient for validating new code.

**Next conversation handoff:** Private `Chatlauf.md` and `PROJEKTSTATUS.md` live exclusively in the user-authorized Home Assistant folder `/share/GitHub SolidCraft/`; neither is stored in public GitHub. The user requested documentation/transfer first and a new chat afterward. Major architectural modifications require prior user confirmation.


## Run #20 locator correction (22 October 2026; validation pending)

Source inspection confirms that `Harness::at` forwards non-array locators to `ui.at`, and `control.rs` resolves published targets only through the `handle` object field. A bare string cannot resolve a published cylinder target.

All **nine click steps across the four failing GUI tests** now use `{"click":{"handle":"drawing:cylinder:N"}}` (indices 0 and 1). Production UI code and assertions are unchanged. All 15 GUI tests and the full 60-check non-fail-fast bundle are retained.

Static verification checked the nine replacements, absence of the invalid cylinder click syntax, and preservation of test names/assertions and production code. No Rust build or GUI execution was performed for this correction yet; the latest executed result remains Run #20, **56/60**. Start a new manual `SolidCraft Browser Smoke (manual)` run on `feature/browser-automation` to validate the corrected commit, rather than rerunning the old SHA.


## Verified success: corrected direct-selection GUI tests — Run #21 (22 October 2026)

[Run #21](https://github.com/Benny1496/solidcraft/actions/runs/39752171411) completed **successfully** at 09:33:37 UTC on `feature/browser-automation`, tested commit `b017c114e52a5d460fe6da1b95c931879ce0e662`. Job `119735151506` and its execution log were checked.

| Group | Verified result |
| --- | --- |
| Browser CAD scenarios + API smoke + reference plate | **29/29 PASS** (27 + 2) |
| Real egui drawing interaction tests | **15/15 PASS** |
| Native Rust drawing core | **16/16 PASS** |
| Total | **60/60 PASS** |
| JavaScript syntax, WASM build, artifact upload, final summary | **PASS** |

All four GUI tests that failed in #20 now pass with the nine structured handle locators. This verifies actual circle selection, exact selection of the second cylinder and its diameter/X/Y annotations, switching selection, blocking edits on unsaved/stale drawings, resuming after explicit update, clearing selection and resetting it on reopening without persisting transient state. No production-code change was needed for this correction.

The earlier pending-validation notices are superseded by this result for the tested SHA. The prototype remains limited to the XY bounding envelope and verified cylinder references; true silhouette projection, arbitrary edge selection, centre-to-centre dimensions and production drawing sheets/export remain outside this test claim. This update changes documentation only.


## Two-centre associative distance — prepared after Run #21 (22 October 2026)

**Not yet CI-verified.** The last confirmed run remains #21 (60/60). This increment adds the straight-line **XY centre-to-centre distance**, including diagonal pairs, between two uniquely verified cylinder faces. It does not introduce separate horizontal/vertical pair dimensions or arbitrary edge selection.

In the drawing window: click a circle, choose **Ersten Mittelpunkt merken**, click the other circle, then **Mitte-zu-Mitte setzen**. The saved annotation stores both exact face identities. A reversed pair is the same dimension and cannot be duplicated. The first-point selection is transient and cleared on stale geometry, body changes, clearing the selection, and reopening.

The existing `drawing.annotation.add` command accepts `kind: "center_distance"` and `second_face_name`. Both names must resolve uniquely to different measured centres. UI/SVG show an M–M label, the two referenced centres and connecting lines; labels remain draggable. This remains prototype graphics, not standards-certified dimension styling.

**3D-to-2D confirmation is mandatory:** changing the model does not mutate the saved drawing. When the closed drawing is reopened, the existing warning offers **Zeichnung aktualisieren** or **Alte Zeichnung beibehalten**. Acceptance recomputes the distance from both source faces and preserves manual label coordinates. A missing/ambiguous face leaves the annotation unresolved with null current value, historical value and a question mark, never reassigned by proximity. With the drawing closed, no popup interrupts 3D work.

Tests prepared in the existing non-fail-fast workflow:
- One browser scenario: analytic 3–4–5 distance, parametric change, explicit acceptance, position retention, reverse duplicates/invalid references, Undo/Redo, removed second bore and repeated preview.
- Two Rust tests: both references and reordered cylinders, lost first/second references and malformed pairs; real save/open followed by a parameter edit, unchanged saved snapshot until acceptance and retained layout.
- Two real egui tests: two-circle mouse selection, label drag, closing before a 3D edit, popup on reopening, declining/reopening/accepting; clearing/stale selection reset.

Expected bundle: **30 browser checks (28 named + 2 base), 17 GUI, 18 Rust = 65 checks**. Existing 60 tests and workflow continuation rules retained. JavaScript syntax checked with Node; source diff reviewed. Rust compilation, full tests and visual validation remain pending on GitHub (no local Rust toolchain). A pending design-format note records the new persisted annotation fields; no migration of older drawings is needed. Run a new manual workflow on `feature/browser-automation`.


## Verified success: centre-distance and confirmation workflow — Run #22

[Run #22](https://github.com/Benny1496/solidcraft/actions/runs/39759469065) completed **successfully** on 22 October 2026 at 10:26:10 UTC, tested commit `4b878cc724b79a8c3a98e52e710b1bc3308b5c10` on `feature/browser-automation`. Job `119759751940`, all steps and its execution log were checked.

| Group | Result |
| --- | --- |
| Named browser CAD scenarios | 28/28 PASS |
| Browser API smoke + four-hole reference plate | 2/2 PASS |
| Real egui drawing interactions | 17/17 PASS |
| Native Rust drawing core | 18/18 PASS |
| **Total** | **65/65 PASS** |

Syntax, WASM build, artifact upload and final consolidated summary also passed (zero nonpassing groups). All five new tests pass: browser centre-distance scenario; GUI two-point selection and label drag with close/edit/reopen/decline/reopen/accept, GUI stale/cleared anchor; Rust dual-identity/missing-reference handling and real save/open followed by an edit.

This confirms that a model edit leaves the stored drawing unchanged until explicit acceptance, reopening displays the update prompt, closed drawings do not interrupt 3D work, and acceptance preserves both references and manual label placement. The previous pending-CI notice is superseded for this tested SHA.

Scope remains an experimental XY bounding-envelope drawing with verified cylinder references and straight-line centre distances, not a finished multi-view manufacturing drawing system or exhaustive validation of SolidCraft. True contours/hidden edges, broader dimension types, annotation editing/deletion, sheets/title blocks/scales, production PDF/DXF and manual visual evaluation remain open. This result update changes documentation only.


## First actual B-rep edge-projection test slice (not yet CI-verified, 22 October 2026)

A new read-only experimental command, \`drawing.projected_edges\`, takes \`{body:"Plate"}\`.
It measures \`model.edges\` from the actual 3D B-rep and projects only straight,
non-point XY segments. It normalizes endpoint order, removes matching top/bottom
duplicates and sorts the result deterministically. The result includes measured
bounds, segments \`{start_xy_mm,end_xy_mm}\` and counters for skipped curved
edges, point projections and duplicates. A curved arc is **not** drawn as a
misleading straight chord. The output is explicitly
\`unclassified_straight_brep_edges_only\`: there is **no silhouette proof**,
hidden-line classification, arc/spline projection, full pocket visibility or
certified manufacturing geometry yet.

This initial API deliberately does not modify \`drawing.preview\`, stored drawing
snapshots, annotation identities, the on-screen 2D workspace or the update
confirmation dialog. The existing 65-regression suite is retained. Six new,
independent browser scenarios cover rectangles, dimensions, a chamfer,
bores/curves, parameters/undo/redo, input validation and isolation; five new
engine unit tests cover geometry, deterministic ordering, edits, errors and
saved-drawing isolation. The intended bundle is 36 browser (34 CAD scenarios
plus 2 baseline), 17 egui and 23 Rust = **76 checks**. Do not record them as
successful before running and inspecting a fresh manual GitHub Actions workflow
on \`feature/browser-automation\`. The old browser DXF test remains isolated
because it previously stalled CDP.

## GitHub Actions run #23: B-rep edge projection matrix PASSED (22 October 2026)

- [Manual run #23](https://github.com/Benny1496/solidcraft/actions/runs/39767894612) on `feature/browser-automation`, code SHA `00c4d9e00bf05799e3a3d3463698fea682b2afa6`. Completed **success** on 22 October 2026 at 11:50:16 UTC (13:50:16 CEST). Job `119787810296`. Entire job log and individual test result lines verified.
- All **76/76 checks passed**, with no failed test groups: browser **36/36** (34 named CAD scenarios plus API and the four-hole plate), real egui **17/17**, Rust engine **23/23**; **0 failed, 0 ignored** in Rust/egui test subsets. JavaScript checks, release WebAssembly build and artifact upload also succeeded.
- All six new browser B-rep edge scenarios passed individually: `drawing-edges-box` (4 unique XY lines, no timeline change), `drawing-edges-edit` (parametric edit plus undo/redo), `drawing-edges-chamfer` (5 projected segments with an actual diagonal chamfer), `drawing-edges-bores` (curved hole edges not misrepresented as straight line chords), `drawing-edges-matrix` (20×12, 75×40, 150×80, 210×55 mm plates; projected perimeters 64, 230, 460 and 530 mm), `drawing-edges-safety` (4 invalid requests refused, existing 2D revision retained).
- Five new Rust tests passed in the 23/23 engine total, confirming straight projections, repeatability, ignored vertical-only projections, model edits/undo, invalid inputs and existing linked-drawing isolation.
- The newly demonstrated `drawing.projected_edges` API reports conservative **unclassified straight B-rep candidates only**. This is NOT yet a silhouette/hidden-line-removal engine, nor does it display true contours in the 2D egui drawing window or handle actual curved arcs and pockets as printable outlines. `drawing.preview` intentionally keeps the existing rectangle-based view and safely linked dimensions unchanged.
- Next proposed increment (not yet implemented or tested): correct XY contour identification, true arc curves and visibility classification, then integrate a properly marked contour into the 2D GUI while preserving prior saved-document format, 3D update confirmation and 76 regression checks. Keep DXF timeouts isolated.


## Run #24 development slice: sampled upper B-rep contours in the experimental 2D view (NOT YET TESTED, 22 October 2026)

The existing \`drawing.preview\` now exposes \`geometry.top_edge_candidates\`,
a sorted set of measured top-Z B-rep edge polylines, with
\`points_xy_mm\`, \`kind: straight|sampled_curve\`, and \`closed\`.
Curved edges are tessellated to bounded, millimetre-based samples; they
are *not* replaced with straight endpoint chords. The experimental
drawing SVG displays these paths distinctly, and the real egui window
shows a candidate-edge overlay while continuing to show its bounding
rectangle as an envelope. Existing cylindrical measures, annotations,
layout and stale-model confirmation logic are preserved.

New drawings include \`source.top_edge_paths_xy_mm\` for changes that
alter the top contour but not bbox or bore measurements. Legacy
drawings lacking this source field remain compatible. The persisted
format stays \`solidcraft-drawing-preview/1\`.

**Limits**: only edges whose full sampled curve lies in the topmost
horizontal plane; no general HLR, silhouette topology, concave pocket
visibility, cutting-plane support, exact arc representation, or
standards-certified manufacturing output. As such the preview uses
\`bbox_envelope_with_unclassified_upper_edge_samples\` and
must not be described as a complete contour-projection engine.

All previous 76 checks remain. Added six browser scenarios
(rectangle, real chamfer, fillet, bores, edited revision and
legacy/malformed snapshots), two egui window tests and four Rust tests.
Planned #24 is **88 checks: 42 browser + 19 egui + 27 Rust** in the
same manual, non-fail-fast workflow; only a successful future run
can validate those counts and the new functionality.


## Run #24 failure corrected and run #25 expanded regression suite (22 October 2026)

Official [run #24](https://github.com/Benny1496/solidcraft/actions/runs/39779700382)
on code `0e9c828d39c94dffbb280c9267ba8cb3b9590c74` completed
**failure**, despite successful WASM build, JS syntax, GUI **19/19** and
engine **27/27**: browser **41/42**. The sole failing check was the older
`drawing-dim-offset` assertion expecting the retired
`bbox_envelope_and_measured_cylinders_only` accuracy string after the
new upper-edge candidate overlay updated the documented accuracy to
`bbox_envelope_with_unclassified_upper_edge_samples`. All six #24
upper-contour scenarios were PASS. The run was correctly reported red.

The legacy browser assertion is now updated to check the new *explicitly
unclassified* accuracy, still-rectangular bounding envelope, upper
B-rep paths and warning against claiming certified silhouettes.
**No production-code change is needed for this regression.**

Added eight independent browser scenarios: negative/transformed XY
coordinates; four-size contour matrix; rounded-fillet undo/redo;
same-bounds chamfer preserving moved dimensions; saved metadata;
JSON numeric-signature roundtrip; chamfer plus two bore rims; and
two separate bodies. Added two real egui cases: saved contour reopen
without false warning, and user-declined same-bounds chamfer prompting
again on reopen. Added two Rust engine cases for actual contour-only
chamfer/undo and number-type-insensitive contour signatures.

Proposed next run **#25** has **100 independent checks**: **50 browser**
(48 CAD scenarios + API smoke + four-hole plate), **21 egui**, **29 Rust**.
The existing 88 are still included. The new run must be triggered
manually on `feature/browser-automation`; tests must not be reported
as passed until its GitHub Actions log verifies success. General
hidden-line removal, certified silhouette and DXF export remain
outside this experimental scope.

## Upstream RFC #860 coordination and contribution scope (22 October 2026)

The canonical SolidCraft project has a **public, active Drawings plan** in
[upstream RFC #860](https://github.com/tangency-labs/solidcraft/issues/860).
Its staged roadmap distinguishes M1 independent geometry/snapshot tools, M2
integrated, saved/reopened/updated single-part drawings, and later industrial
part-and-assembly coverage. The contributor's goal is a practical native
3D-to-2D workflow that fits this design, not a competing application.

**Do not claim or duplicate packages already underway.** On 22 October 2026
CamWight has active issues/draft PRs for sheet/paper primitives and exports
(W10/W21/W22, [#4890](https://github.com/tangency-labs/solidcraft/issues/4890)),
drawing references/updates (W13/W14, [#4892](https://github.com/tangency-labs/solidcraft/issues/4892)),
the native workspace/end-to-end M2 checkpoint (W16/W23,
[#4894](https://github.com/tangency-labs/solidcraft/issues/4894)),
dimension formatting/annotations (W09/W19/W20,
[#4896](https://github.com/tangency-labs/solidcraft/issues/4896)),
view frames ([#4816](https://github.com/tangency-labs/solidcraft/issues/4816)),
and storage/records/commands ([#3890](https://github.com/tangency-labs/solidcraft/issues/3890)).
Other contributors own section geometry and hidden-line work.
Work-package ownership is subject to maintainer guidance; a draft is not a
finished or merged product. Do not assume upstream will accept this prototype's
temporary `drawing.preview` metadata contract.

**Proposed cooperative contribution** (pending acknowledgement in #860):
small, original, analytic workshop-oriented plate fixtures; repeatable
native/browser acceptance testing of saved/reopened drawings, deferred update,
rejected/accepted changes, lost references, undo and hand-placed dimensions;
and hands-on UX feedback about the confirmation flow and drawing output.
If upstream requests an isolated code slice, first align with its current
W11/W12/W13/W14/W16/W17/W19 contracts and avoid an alternate document model.

**Connection status:** A first substantive comment was drafted for #860 with
the public fork and the verified [run #25](https://github.com/Benny1496/solidcraft/actions/runs/39787639840)
(**100/100 scoped checks passing: 50 browser, 21 egui, 29 Rust**).
The connected GitHub integration was denied permission to comment on the
upstream issue (HTTP 403 `Resource not accessible by integration`).
Consequently **no upstream contact has been posted yet**; the owner must
paste the prepared comment using their own logged-in GitHub account or grant
an appropriate permission path separately. No upstream PR or merge has been made.

## Test #26 development: native three-panel orthographic reference preview (22 October 2026)

For a genuinely more useful in-app workflow, the existing egui
`2D-Zeichnung (Testversion)` now shows **three related views**, without a
new external program: the original manually dimensionable XY top view,
a read-only *front XZ* reference panel and a read-only *right YZ*
reference panel. The two new panels are derived from sampled **real
B-rep edges lying on the frontmost (`Y=min`) and rightmost (`X=max`)
extreme planes**, with measured physical bounding sizes. Each panel
explicitly labels its projection, coordinates and uncertified accuracy.

`drawing.preview` exposes `geometry.reference_views.front/right`,
each with `projection`, `plane_mm`, `bounds` and `paths`; the
separate numeric signature `source.orthographic_edge_paths` ensures
changes to these planar edge candidates are not silently treated as
current. Original drawings created before this field continue to work.
Read-only panels display the **stored previous snapshot** while the
original model is stale; they show new 3D geometry only after a user
confirms an update. The linked top annotations and the metadata format
remain unchanged. No released SolidCraft executable is implied.

**Accuracy and limitations:** these are extreme-plane edge candidates,
*not* visibility-classified complete orthographic drawings. In
particular, no hidden-line removal, tangent silhouettes, behind-face
hole outlines, sections or side-view dimensions are claimed. The
top-panel rectangle and both reference rectangles are bounding
envelopes, not verified manufacturing contours. Upstream RFC #860
and W17 view definitions remain authoritative for future integration.

All previously green **100 checks** remain in the unchanged manual
GitHub workflow. **8 additional browser** scenarios (box, translated
stock, changed length, changed thickness and undo, bores, legacy
snapshot, persisted dimensions, chamfer), **2 real egui** tests and
**4 Rust** tests have been added. Planned **#26: 114 checks**
(58 browser, 23 GUI, 33 engine) — *not yet compiled or run in GitHub
Actions*. A passed workflow is required before calling these
additions successful.

## Test #26 result and proposed upstream-oriented #27 (22 October 2026)

Real [workflow run #26](https://github.com/Benny1496/solidcraft/actions/runs/39800272899)
finished **failed**: 109 of 114 tests passed (57/58 browser, 21/23
real egui, 31/33 Rust); WASM compiled and all eight newly added
three-view browser scenarios passed. The only browser failure was an
outdated warning-text assertion in `drawing-dim-offset`. Two Rust
and two UI tests compared JSON integer literals with `f64` JSON
numbers instead of asserting equivalent physical numeric values.
The functional geometry and dimensions passed their browser tests.

The next commit corrects those **five test assumptions without
changing the engine or screen's drawing geometry**. Float-typed
expected values are retained at full exact numeric precision in the
Rust/UI checks; the browser warning check is updated to require the
*actual explicit no-HLR / bounding-envelope disclosure*, not removed.

Four new browser, two native egui and two Rust checks implement a
coherent subset of the workshop acceptance fixtures requested by
[upstream RFC #860](https://github.com/tangency-labs/solidcraft/issues/860#issuecomment-6344334038)
for [W16/W23, #4894](https://github.com/tangency-labs/solidcraft/issues/4894):
two similar bores with measured and separately placed diameter and
centre-to-centre dimensions; stored/reopened drawing state; model
parameter edits, deferral and user-accepted refresh; closing/reopening
the real drawing window; undo/redo; deleting one of two similar bores
without misattributing its dimensions; and preserved historical values
and manually placed callouts. All tests stay within the temporary
experimental `drawing.preview` metadata model pending upstream W11
storage and W13 reference contracts. They do not claim upstream merge,
certified HLR or PDF export.

**Planned manual workflow #27: 122 checks** =
62 browser (60 CAD + 2 baselines), 25 egui, 35 Rust.
Do not describe this batch as successful until GitHub CI logs confirm it.

## Consolidated #27 manual workflow: experimental Drawing + upstream W23a oracle

Following the independently claimed upstream contribution
[#4950](https://github.com/tangency-labs/solidcraft/issues/4950),
with [draft PR #4962](https://github.com/tangency-labs/solidcraft/pull/4962),
the *existing fork-only* manual GitHub Actions workflow now has two
independent jobs, one launch:

1. **Experimental Browser Smoke:** run the previously prepared #27 suite
   on \`feature/browser-automation\` (planned 122 scoped checks across
   browser/API, real egui UI and Rust Drawing engine).
2. **Official workshop W23a oracle:** separately check out
   \`feature/upstream-4950-oracle\` from the same fork, verify every
   analytic \`tests/oracle/84-*\` through \`95-*\` reference via
   \`python3 tests/oracle/W23a-verify.py\`, build the genuine SolidCraft
   native CLI and run **all** \`cargo xtask oracle\` reference cases.
   Upload the resulting \`docs/oracle.md\` report.

Both jobs are independent; a failure in one does not prevent the other
from reporting. Do not mistake a passing independent formula check for
successful B-rep geometry replay or an accepted upstream PR. The oracle
branch remains free of the experimental Drawing code, and the extra
job is **not** part of upstream draft PR #4962.

The 12 new oracle cases include a chamfered plate and a separate
chamfer-plus-fillet reference with closed-form volume, area and centroid.
Kernel replay is still required to validate the actual edge selections
and topology; production 2D command acceptance remains gated by the
upstream Drawing API and storage decisions.

This workflow is manual and must be dispatched by a GitHub account with
Actions access. Selecting branch \`feature/browser-automation\` triggers
the combined job list; do not select the oracle-only branch, where this
fork-only workflow is intentionally absent.

