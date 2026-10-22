# Experimental model-linked XY drawing preview

This is a development-only first slice for testing model-linked 2D manufacturing drawing behavior. It is **not** a Drawing workspace, a persisted sheet, a standards-certified projection, an HLR engine, or a replacement for the published drawing roadmap (#860). All work is isolated on the fork's `feature/browser-automation` branch.

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
