> **T103 root-cause fixes / 08 Nov 2026.** Previous [T102 run 42479389608](https://github.com/Benny1496/solidcraft/actions/runs/42479389608) failed at `cargo clippy -D warnings` before UI tests executed. The **full job log** confirms exactly two warnings in test-only `t102_warning_poc.rs`: `clippy::missing_const_for_thread_local` (LAST_ACTION RefCell) and `clippy::collapsible_if` (nested inspect/candidates if-let). Both are corrected in this new isolated T103 fixture. Type mismatch from T101 remains fixed. New injector assertions prevent the two known Clippy regressions before compiling. This patch is runner-only and never changes upstream production files. No success is assumed until CI finishes.

> **T103 fix:** T101 failed before tests on Rust E0369 because `serde_json::Value` was compared to `usize`. T103 converts the JSON count via `.as_u64()` and compares to a numeric index; then exercises seven CI-only native UI cases plus broad drawing regressions. This does not prove product UI is implemented.\n\n# Test #103 — native GUI: click-to-update and explicit dimension repair candidate

## Why this test matters
Previous [Test #100, run 42471557003](https://github.com/Benny1496/solidcraft/actions/runs/42471557003) passed all 6 native UI proof-of-concept checks, 4 focused GUI controls, the native Drawing GUI suite (51/51) and Drawing Engine suite (395/395), including strict Rustfmt, Clippy and compilation. It took ~39 min. The status pill and clicks work, but the proof-of-concept had **no real Repair action**.

**Test #103 adds clickable, mutating UI actions** to a temporary test-only overlay:
- `Update drawing` appears **only for `out_of_date`**, acts through `drawing.update` when clicked, and clears a deferred-dimension-only warning even if all views were already current.
- For `lost` and `ambiguous`, the card reads `drawing.dimension.inspect` and shows **only the actual edge candidates the engine supplies**, each as its own clickable `Repair candidate N` button. It passes the selected candidate unmodified to `drawing.dimension.repair`. No automatic repairs.
- If the entire 3D feature was deleted and no candidate remains, the card offers **no fake Repair button**. It also does not suggest `drawing.update` as a fix for `lost`.
- A real fillet of the plate's upper front edge creates a reproducible lost reference with a real candidate. The user-selected front-edge candidate must repair the **same dimension ID** in **one undo step**; Undo must restore `lost`, Redo restore `ok`.
- Repair of one dimension must preserve another valid dimension's record/status.

## Seven new native headless egui tests
1. `t103_native_update_button_clears_deferred_dimension_only_after_click`
2. `t103_native_lost_edge_repair_candidate_click_undo_redo`
3. `t103_native_no_repair_candidates_does_not_offer_false_action`
4. `t103_native_selected_repair_does_not_touch_other_dimension`
5. `t103_native_all_current_has_no_update_or_repair_buttons`

Plus two additional status-mapping checks: `t103_warning_message_prioritises_lost_ambiguous_and_stale` and `t103_warning_message_ignores_healthy_dimensions`.\n\nPlus **four existing native sheet GUI controls** and both large Drawing regression suites from the engine and GUI test binaries (previous run: 395 Engine + 51 GUI tests; counts can change only if the source checkout changes). Format/Clippy `-D warnings`, compile both crates only once, run named tests with per-case logs, preserve ZIP even on failure, cap job at 90 minutes without artificial delay.

## Source and isolation
- Own fork branch: `Benny1496/solidcraft:tests/solidcraft-t103`; a distinct workflow `solidcraft-t103.yml`.
- Source under test: **Loohi's not-yet-merged PR #13211**, pinned to `Loohi/solidcraft@84ef88b10e394c9ea9f90459c242e2033c6f7133`; read-only checkout. PR #13159 (Nathan UI design, D07/D26) remains Draft.
- The UI under test is **injected only into GitHub runner** with `#[cfg(test)]`. The real `egui::Area` foreground button and candidate buttons work only in test binaries; no product implementation or developer branch is changed.
- GitHub workflow has read-only permissions and validates the branch/commit and modified files before the build. GitHub Actions ZIP and sidecar SHA256 must be archived on Home Assistant after completion.

**Limits:** The prototype's candidate UI is specifically for length/edge dimensions; no claim of complete D07/D26 across all dimension types, real production-ready controls, or visual browser E2E. A green Test #103 would prove the actual native-UI **command path**, not completion of the official Drawing UI slice. The real implementation must wait for Nathan-Dane's assigned child issue and be reviewed alongside Loohi's sheet page.
