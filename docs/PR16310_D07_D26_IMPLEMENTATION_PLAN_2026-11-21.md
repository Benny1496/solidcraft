# Drawing UI alignment for PR #16310 — 2026-11-21

This is preparation on the isolated working branch, **not** an update to upstream PR #16310.

## Pinned design source

Nathan-Dane/solidcraft PR #13159, head `d7512ced5cb9673c488b85a557e1ff6915ffd891`, `docs/design/drawing-ui/decisions.md` and `README.md`. The design PR was open when reviewed. Recheck its final merged version before integration.

## Verified design decisions

- **D07 (decided):** Status pill at the top centre of the free sheet area; “N need attention” or “All current”. Its card lists findings and Repair, then Check drawing and Update drawing; when clean, Export.
- **D26 (decided):** Status text plus a shape, never colour alone: Ok ✓, Overridden ⓘ, Ambiguous ⚠, Lost ✕, Needs update ⟳. Say “sheet”, not “page”.
- **D17 (decided):** A view's boundary appears near its edge, dragging via that band; clicks inside use Pick mode, with specified precedence.
- **D21 (decided):** One Dimension tool, edge hover preview, and documented Model Dimensions / Suggest / Tidy commands. This does **not** by itself specify the exact repair-picking interaction.

## Current implementation compared to design

- `crates/ui-egui/src/drawing_workspace/browser.rs` currently calls `show_attention(app, rows.ui())` inside the left Browser. This differs from D07. Moving it needs a sheet-viewport host and care to preserve the repair `app.run` callback, snapshot refresh, and repair state.
- `crates/ui-egui/src/drawing_attention_card.rs` shows translated state words but not the full D26 icon set. Use project icon masters / UI kit rather than ad-hoc glyphs if available.
- `crates/ui-egui/src/drawing_repair_ui.rs` accepts engine candidates or one pre-existing model selection; it does not yet pick directly on the drawing sheet via the same mechanism as the Dimension tool. Inspect `drawing_workspace/dimension.rs` and `drawing_scene::line_at` before changing this.
- Review cleanup: `drawing_dimension_repair::confirm` is reportedly unused outside tests; verify callers before removal.

## Safe implementation order

1. Trace the sheet viewport rendering and its input layers; decide where the D07 pill/card can be hosted without obstructing drawing interactions. Keep status read-only until the user explicitly chooses Repair.
2. Implement D26 shape + text via existing icon masters and localized strings; test each state.
3. Implement D07 pill and expandable card in the sheet area; preserve existing repair state and command path, including Undo.
4. Reuse the existing dimension-tool picking semantics for sheet edge selection. Confirm exact slot/pick mapping with the engine, do not infer geometry.
5. Remove unused helper only after a call-site search and add regression tests.
6. Run formatting, locale/list checks, compilation, targeted nextest with JUnit, and `cargo xtask known-failing --junit <report>`; then a native GUI walkthrough at 1280x720 and 1024x640.
7. Compare against the finalized design, share results with CamWight for review. **Do not push these changes to PR #16310 without separate approval.**

Test 230: https://github.com/Benny1496/solidcraft/actions/runs/44735570728 (green preflight for earlier Area A changes; does not validate the planned D07/D26 implementation).
