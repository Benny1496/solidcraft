# SolidCraft Test #98 – native Drawing sheet and dimension status (PR #13211)

**Basis:** Loohi's open review PR [#13211](https://github.com/tangency-labs/solidcraft/pull/13211), pinned to `Loohi/solidcraft@84ef88b10e394c9ea9f90459c242e2033c6f7133`. This is **not** the official merged `drawings` branch and cannot establish integrated product readiness.

**Motivation:** Previous T96 (7/7) and T97 (8/8) established engine states and repair commands. Loohi has now built the first actual Drawing sheet GUI, so T98 drives **real headless native egui frames** via the app's scenario `Harness`, rather than repeating engine-only tests. Our allocated GUI feature, the D07/D26 status card for lost/out-of-date dimensions, is explicitly *not* yet in PR #13211 (Issue #13210).

## New checks: 3
1. **Positive native UI:** 100 × 80 × 20 mm plate shown with two views and 100-mm dimension → close Drawing → modify 3D feature to length 120 → reopen Drawing → egui redraw completes; same dimension ID reads 120 mm, two visible views current, dimension scene group rendered and no page error.
2. **Gap diagnostic, deferred dimension only:** After a model revision that does not change the views (a sketch), `drawing.status` reports `waiting=0`, `held=true`, `out_of_date=true` for dimension. Real `ui.inspect.sheet_page` shows current views **but lacks per-dimension status**. This test reports `T98_GAP`, **not** a GUI PASS for the warning feature.
3. **Gap diagnostic, truly lost:** Delete the model feature and reopen Drawing, inspect/status show `lost` with a reason and no current value; native sheet remains open **but has no per-dimension status/Repair card**. Again `T98_GAP`, **not** a fix.

**Upstream controls:** `the_plate_sheet_draws_its_lines_and_text`, `a_views_state_is_in_words`, `fit_sheet_shows_the_whole_sheet_and_zoom_stays_in_range`, `no_drawing_open_shows_the_note`. Total **7 targeted Rust tests**, of which *only one* new test is a positive UI model-change integration; two are negative diagnostics for what Benny must implement later. A green T98 may therefore still mean **D07/D26 missing**.

## Workflow design
- Isolated fork branch `tests/solidcraft-t98`, checkout Loohi PR head pinned and read-only; overlay T98 code in GitHub runner workspace only.
- cargo fmt --all --check; clippy -p solidcraft-engine -p solidcraft-ui-egui --all-targets --profile ci; compile UI lib-test binary once with cargo JSON artifact output.
- Run seven named tests from the single prebuilt `solidcraft_ui_egui` binary, capture each log and categorized summary, upload original ZIP even on failure. Job cap 90 min; readonly GitHub token.
- Archive original ZIP and SHA256 on HA after finish.

**Next step following findings:** after Nathan's design PR #13159 and child issue, implement D07/D26 status pill and per-dimension warnings. Change diagnostics to **positive UI acceptance assertions**, i.e. user-visible affected dimension, reason, Update/Repair action, and no historical value presented as current. Do not equate PR #13211's view-level warning with our dimension-level UI warning.
