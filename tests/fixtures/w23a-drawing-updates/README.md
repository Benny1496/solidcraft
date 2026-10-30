# W23a – seven candidate native acceptance tests for the official SolidCraft suite

This is a **test-only fork branch**, based on upstream `tangency-labs/solidcraft` `drawings@562bb379a5b426506d7fc75cc069bc0936044671` (30 October 2026). It neither changes production geometry nor pushes to the existing official [draft PR #8288](https://github.com/tangency-labs/solidcraft/pull/8288).

## Why only seven?

The preceding source-bound external [Test #42](https://github.com/Benny1496/ESP32-Victron-SolarSense-Bridge/actions/runs/41134272831) passed **128/128 native Rust checks** (64 preserved, 64 varied cases). Here we deliberately retain **seven distinctive tests** that represent CamWight's already executable E2E-01 and E2E-02 *engine-command subsets* rather than expanding the synthetic variation matrix.

The selected tests, in `crates/engine/src/w23a_model_edit_acceptance_tests.rs`, verify:

1. `bore1_10_to_12_keeps_dimension_id_and_independent_bore2`: **E2E-01** Ø10→Ø12, same dimension ID; Bore2 stays Ø10.
2. `deleted_bore1_is_lost_never_bore2_then_undo`: **E2E-02** delete Bore1, status `lost`, never rebinds to Bore2; undo restores.
3. `original_oracle84_change_undo_redo_preserves_measure_identity`: parameter edit/undo/redo and original Oracle84/86 volume anchors.
4. `user_authored_sheet_text_positions_survive_diameter_edit_and_undo_redo`: authored `text.at` stays put; sheet coordinates are converted to view-relative millimetres. **Record-only**, not on-paper rendering.
5. `public_drawing_scene_matches_two_views_after_bore_parameter_edit`: public `drawing.scene` matches actual settled engine two-view geometry after a diameter change. **Not automatic UI U2.**
6. `public_two_view_delete_undo_redo_stays_lost_without_rebinding`: two-view geometry and missing dimension status across Bore1 delete/undo/redo.
7. `last_good_12_mm_stays_with_lost_dimension_through_delete_undo_redo`: last measured Ø12 retained **within the current session**, no claim of file persistence.

## Reproducible test fixture and scope

The helper fixture is committed in the same branch at `tests/fixtures/w23a-drawing-updates/oracle84-commands.json`. It contains the **same 25 CAD commands** previously produced by the pinned official `solidcraft-cli recipe tests/oracle/84-plate-two-bores/recipe.json` converter in Test42. The `-0.0` in the CLI's negative-Z directions is normalized to `0.0`, with identical geometric meaning. The Rust module uses `include_str!` to load this fixture from the repository at compile time. **No** external runner files, `ci-reports` directory, or required pre-test conversion; the test is wired by `#[cfg(test)] mod w23a_model_edit_acceptance_tests;` in `crates/engine/src/lib.rs`.

To run from a checkout of this test branch:

```shell
cargo test --locked -p solidcraft-engine --lib w23a_official_ -- --nocapture
```

The previous 128/128 result verifies the **original external harness at the previous pinned source**, **not** these seven newly committed self-contained tests on the newer upstream base. They require their own native validation before claiming PASS. CI runs on the ESP32 repository temporarily due to the SolidCraft fork's GitHub Actions restriction. The fork remains the source of truth.

## Not tested / out of scope

- Full official E2E-01 through E2E-04 acceptance remains **NOT RUN**.
- Automatic U2/UI updates while a drawing stays visible; paper-rendered labels, balloons or dimensions.
- Save **real .sc design file** and reopen, persist/restore `historical_last_value_mm` (CamWight decided to implement separately).
- Unimplemented R1 flat-face dimensions or R2 bore-centre distances; body rename and feature-ID reuse awaiting other changes.

## Upstream submission

Prefer a small, narrowly scoped test-only PR (base `drawings`) after asking CamWight where he wants these registered. His 30 October review of PR #8288 explicitly endorsed the E2E-01/E2E-02 `drawing.dimension.inspect` subsets but **has not yet approved a test insertion path**. This branch is prepared for review, **not submitted or merged**.
