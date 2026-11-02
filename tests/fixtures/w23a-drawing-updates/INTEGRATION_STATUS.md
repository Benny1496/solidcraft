# W23a integration staging – 2026-11-02

**Integration branch:** `Benny1496/solidcraft:integration/w23a-8288-clean-20261102`  
**Upstream base:** `tangency-labs/solidcraft:drawings@d5fd14135f0e220b45171010722a0064683c63f1`  
**Unchanged official draft:** [tangency-labs/solidcraft#8288](https://github.com/tangency-labs/solidcraft/pull/8288), head `c86b7a4c233d4903ded52ac155b1f0ee82abd79c`.

This branch is intentionally **not** a rebase of all historical commits from #8288. It is a minimal, reviewable reapplication of the five E2E Engine test cases plus their original case documentation on the current drawings base.

## Files and ownership
- `crates/engine/src/cmd/drawing_dimension_tests.rs`: unchanged upstream existing tests, plus the original body-aware edge finder under a distinct `workshop_plate_round_index` name and all five E2E tests from #8288. Keeps the original tests' exact behavioral assertions; fixes only the helper collision with upstream's Box-only `round_index` function.
- `tests/fixtures/w23a-drawing-updates/{README.md,cases.json,validate.py}`: existing, complete W23a acceptance descriptions and static validation (full U2/UI rows still NOT RUN).
- `examples/acceptance/drawing_m2.json`: **TEMPORARY DEPENDENCY**, same blob SHA `6d613c668520d93796391379f3647cabd65c3d84` as CamWight's #9821; not our implementation. Remove from the final #8288 PR delta when upstream #9821 merges.
- `.github/workflows/solidcraft-w23a-integration-56.yml`: fork-only CI/test harness, to be removed or kept outside the official contribution as appropriate. No changes to production engine code, views, schemas, locales or upstream-owned exports.

## Relevant references
- Test #55 attempt 4, upstream compatibility 93/93 actual native tests: https://github.com/Benny1496/solidcraft/actions/runs/41422663973
- Dependency #9821 (shared fixture): https://github.com/tangency-labs/solidcraft/pull/9821
- Dependent W23-2/W23-5 work #10881: https://github.com/tangency-labs/solidcraft/pull/10881
- Merged upstream view work #9254, percentage rename #10713 (`reference_shown_percent` 0–100).
- W13/W14 U2 implementation draft #4893: https://github.com/tangency-labs/solidcraft/pull/4893

## Before changing PR #8288
1. Verify this branch's Test #56 direct engine tests, five E2E results and all static contracts, without 0-matched tests. Record exact SHA and GitHub run.
2. Await/coordinate the upstream M2 plate #9821 merge; remove the temporary staged copy from our PR delta when already available on base.
3. Review diff vs latest drawings and ensure only our test functions/acceptance docs remain in the eventual official PR. The fork-only workflow does not need to go upstream.
4. Update PR #8288 **separately and deliberately**, after preserving its existing head and reviewing CamWight's guidance; do not force-update the PR in this staging step.
5. Keep U2/live UI refresh and rendered annotations **NOT RUN** until a real end-to-end browser/UI scenario is executed.
