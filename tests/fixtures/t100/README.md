# SolidCraft Test #100 – native Drawing UI foreground status card + complete Drawing regressions

**Status:** Prepared for first GitHub Actions execution (result not yet known).

**Why Test #99 failed, precisely:** After the earlier source-formatting and Clippy problems were fixed, [T99 run 42465665412](https://github.com/Benny1496/solidcraft/actions/runs/42465665412) compiled both crates and ran the suites. All **395** Drawing Engine checks passed; in the native GUI Drawing suite **47 of 50** passed. Exactly three T99 prototype tests failed because `expanded` stayed `false` after a scenario click on the test-only pill (the Lost test failed because no expanded detail card was painted). The two direct D07/D26 mapping tests and four existing focused GUI tests passed. This is a problem with **our test-only click layer**, not evidence of an engine regression. The original artifact is GitHub `13613430642`.

**Fix for #100:** Replace a hand-drawn `ui.interact` overlapping Loohi's full-canvas `click_and_drag` sheet with a real `egui::Button` inside its **own `egui::Area` at `Order::Foreground`**. Publish the button's actual response rectangle as `t100:status_pill`. The scenario harness uses this real target for clicks. Publish `t100:status_details` only when the card is visibly opened. Keep all prototype code strictly `#[cfg(test)]`.

**Pinned checkout:** Loohi's still-unmerged PR [#13211](https://github.com/tangency-labs/solidcraft/pull/13211), commit `84ef88b10e394c9ea9f90459c242e2033c6f7133`, read only. The fork branch holds only overlay fixtures and its own workflow. No production source or other developer's branch is changed.

## Six new native status-card checks

1. Current Drawing: actual foreground status button `All current`, clickable open then clickable close, and status detail rectangle appears only when expanded.
2. Deferred dimension with all views current: one `Needs update` finding with correct ID, opening the card, then `drawing.update` clears the warning.
3. Lost model feature: user-visible `Lost` with affected ID and reason, open the card; undo makes `All current`; **no falsely implemented Repair button**.
4. D26 state priority/shape: Lost and Ambiguous take precedence over an out-of-date flag; textual label plus ✕/⚠/⟳; no false alerts on valid measures.
5. D07 fallback explanation: missing `why` gets an honest generic message; historical `last_good` never represented as a fresh measurement.
6. Disabled proof of concept: Loohi's normal sheet stays drawn, no test-only widget appears, and no accidental active state leaks.

Four targeted existing sheet UI controls, plus the *complete matching Drawing test subsets* from native `solidcraft-ui-egui` (at least 15) and `solidcraft-engine` (at least 70). Suites may re-run some focused checks for broad regression coverage; the goal is meaningful coverage, not artificially using the maximum runtime.

**Gates:** 90-minute job cap; Rustfmt; Clippy `-D warnings` for UI + engine; compile both lib-test targets once using Cargo JSON artifacts; focused native egui tests; UI Drawing suite and Engine Drawing suite; always upload original per-case logs and runner-only diff/artifacts to GitHub. All writes restricted to `Benny1496/solidcraft:tests/solidcraft-t100`; workflow has `contents: read`.

**Hard distinction:** A green Test #100 would demonstrate *only the feasibility of a runner-only interactive status-card prototype*, not that D07/D26 has landed in the actual GUI, nor that real repair click flows exist. Nathan-Dane's design PR #13159 / child issue remains the product handoff.
