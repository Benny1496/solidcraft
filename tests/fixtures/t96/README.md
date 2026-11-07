# SolidCraft Test #96 — Drawing warning contract (PREPARED, NOT RUN)

**Task:** @Nathan-Dane invited Benny1496 to implement the Drawing UI warnings for lost and out-of-date dimensions. The design spec is [PR #13159](https://github.com/tangency-labs/solidcraft/pull/13159) (draft at preparation time). Follow **D07** (status pill/findings/repair) and **D26** (word + icon; no colour-only warnings). Relevant boards: `status-card.png`, `annotate.png`, `review-export.png`, `ready-to-issue.png`.

**Pinned upstream source:** `tangency-labs/solidcraft:drawings@c8cbe43dd9ded5b6c0f5fbd0db750bb673bd0fa7` (includes merged #12980). If the official branch changes, do not silently call the result a current-head test: examine and pin the new commit for the next test.

**Status:** isolated T96 fixture prepared for GitHub; **NO workflow or Actions run yet**. No UI product code, PR #8288 or #11881 changed.

## Four independent new engine-to-UI contract cases
1. `t96_clean_dimension_has_no_false_warning`: valid drawn dimension, current views, no false stale/lost UI status.
2. `t96_deferred_dimension_only_waits_while_views_current`: `held=true`, `waiting=0`, views current, but dimension has `out_of_date=true`. This must still show a "Needs update" warning.
3. `t96_deferred_model_change_marks_dimension_stale_not_lost`: model length changed in deferred mode; dimension must stay `status=ok` but `out_of_date=true`; `drawing.update` clears the state after render.
4. `t96_deleted_feature_lost_has_reason_and_no_current_value`: model feature genuinely deleted; `drawing.status` supplies dimension ID and `lost` reason; `drawing.dimension.inspect` returns no current value/text; inspect must not auto-repair; undo restores `ok`.

**Proposed three upstream controls:** `status_says_where_each_dimension_stands`, `status_lists_a_held_views_dimensions_with_it`, `status_lists_dimensions_out_of_date_whose_views_are_current`. Total 7 targeted cases once executed. Cases use pre-existing engine test helpers from the pinned official source.

## Execution architecture — NOT YET ENABLED
A later, explicitly approved workflow `.github/workflows/solidcraft-t96.yml` should:
- Restrict to `Benny1496/solidcraft` and `tests/solidcraft-t96`, `permissions: contents: read`, `timeout-minutes: 90`.
- Checkout the pinned upstream read-only and the isolated fork overlay; call `inject_tests.py` **only in temporary runner checkout**.
- Verify `git diff --check` and that exactly `crates/engine/src/drawing_scene_tests.rs` changed in runner; rustfmt, clippy, compile engine once, run all seven cases with precise PASS/FAIL evidence.
- Preserve original logs via `actions/upload-artifact@v4` even on failures. Archive original GitHub ZIP and SHA256 to Home Assistant afterwards as per project policy.

## Critical acceptance distinction
This is **not** a passing Drawing GUI test. At #94, the real egui workspace has only placeholders. The positive **GUI** acceptance suite (status pill rendered, affected dimension highlighted, reason/repair accessible, no historical value presented as current) must be built against actual GUI implementation **after the relevant child issue/UI integration**, following Nathan-Dane's final designs. This contract fixture is the backend input specification that the UI must honour.
