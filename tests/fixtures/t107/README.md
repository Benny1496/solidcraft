# Test #107 – targeted drawing.check integration regressions

**Target:** CamWight's open upstream PR [#13055](https://github.com/tangency-labs/solidcraft/pull/13055) after later changes to official `drawings`. Tests and workflow live only in Benny1496's isolated fork branch. They do **not** write to upstream, Cam's PR or Benny's existing PRs.

## Pinned sources

- Candidate PR #13055: `5c17c14947d9422a0610fe71fd2a8c21dc4c5417` (native pre-integration code with real `drawing.check`).
- Official `drawings`: `773d5d7fbc3b2e03199b58d0d2748fc162198495` (after PRs #13049, #13060 and #10761).

Recheck pins and changed upstream PRs before interpreting outcomes or preparing another run.

## Test matrix

1. **Contract audit**, Python: reads both exact code trees and records two source-level incompatibility risks (note objects in `out_of_date`; misleading generic `drawing.update` advice for exploded-view held-back annotations). This is a **static audit only**, not a runtime reproduction.
2. **Cam's unchanged baseline**: build and execute its existing native `drawing_check` Engine tests, recording pass/fail with exact logs and test counts.
3. **Runner-only dynamic negative regression**: append exactly one test into Cam's existing `drawing_check_tests.rs` **on the throwaway Actions runner**, without changing the published PR. It creates a real drawing and real view note, then models the *new* contract by putting the note id into `SheetScene.out_of_date`. The test exercises the **actual production** `on_sheet` function and asserts `CheckedKind::Note`. On the original PR this is expected to fail with actual `Dimension`; the job accepts only that precise expected failing assertion and records `KNOWN_CONTRACT_REGRESSION_REPRODUCED`. This is a **simulated cross-version contract scenario**, not a fully integrated current-`drawings` rebuild.
4. **Native upstream reference tests**, exact latest drawing Engine code: original `a_note_on_the_model_an_exploded_view_holds_back_is_out_of_date`, `a_dimension_an_exploded_view_holds_back_is_out_of_date_everywhere`, `a_note_on_an_edge_follows_the_model_and_keeps_its_place_when_lost`, and optionally a broader drawing-scene case. Confirms upstream behavior is tested by native code.

## Limits / interpretation

- A green workflow means **baseline native tests ran, the simulated contract mismatch was diagnosed (or explicitly logged not reproduced), and upstream reference tests passed**. It does **not** mean Cam's unmerged PR #13055 integrates or compiles with current `drawings`.
- A deliberately failing negative test is **not** a green product test. The report names its separate result and contains the original test stdout/stderr.
- Any unexpected compile, lint, fixture or CI error fails that job and needs genuine triage; never treat it as an expected bug.
- No new CI is claimed as already integrated into official SolidCraft. The test branch adds a **separate fork-only workflow** and does not touch `.github/workflows/ci.yml`. The official CI path for eventual product PRs still needs review.
- GitHub Action artifacts should be archived as untouched ZIPs via the existing Node-RED/HA worker with sizes and SHA256, even for failures. Run URL, commits, job steps and logs must be captured.

## Purpose

Share with CamWight only verified and appropriately qualified results; avoid uncoordinated fixes or a second rebase of PR #7608.
