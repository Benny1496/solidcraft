# SolidCraft W23a Test #42 – verified results

**RESULT: PASS — 128/128 native Rust tests, 0 failures.**

## GitHub evidence and exact source

| Item | Evidence |
| --- | --- |
| GitHub Actions runner | [Original Test #42 run 41134272831](https://github.com/Benny1496/ESP32-Victron-SolarSense-Bridge/actions/runs/41134272831) |
| Runner repository | `Benny1496/ESP32-Victron-SolarSense-Bridge` — **external Actions runner**, not an Actions job in the SolidCraft fork |
| Job | `124332720044`; `completed / success` |
| Verified time | 30 October 2026, 21:53:06–22:28:37 UTC (22:53:06–23:28:37 Berlin) |
| Exact **tested** SolidCraft fork commit | `f6dddeccbb29a675a49c4896975c1532d24a4a3b` |
| Official upstream baseline | `tangency-labs/solidcraft drawings@fefca6bfa0c295408f66c4859bb8204ab0560552` |
| Test module | [128 executable Rust tests](w23a_t42_integration_tests.rs), injected only into an external checkout at CI build time |
| Detailed individual PASS lines | [All 128 distinct test cases](verified-128-native-test-results.log) (extracted from original public GitHub job log) |
| Runner commit | `8b320b8bf972e672c79f3db0089fc9f362b5696f` |
| Runner workflow | [Pinned workflow source](https://github.com/Benny1496/ESP32-Victron-SolarSense-Bridge/blob/8b320b8bf972e672c79f3db0089fc9f362b5696f/.github/workflows/solidcraft-w23a-e2e01-02-fork-test42.yml) |
| Official W23a acceptance PR | [Draft PR #8288](https://github.com/tangency-labs/solidcraft/pull/8288), held at `5dc72344b3da09951b4e7df6f3178d5c950cf0b9` in this run; **unchanged** |

The original complete CI result is:

```text
TEST42_SUITE_STRUCTURE_OK=128 (32 oracle matrix + 32 targeted + 64 E2E-01/02 dual-view probes)
PRODUCTION_SOURCE_UNCHANGED_VS_OFFICIAL_DRAWINGS_BASELINE
STATIC CONTRACT PASS: four W23a recipes match source oracle metadata.
REAL_ORACLE84_CLI_BASELINE_PASS 156858.92917653147
ORACLE85_REFERENCE_PASS
NATIVE_W23A_E2E01_02_FORK_PASS=128 FAIL=0
test result: ok. 128 passed; 0 failed; 0 ignored; 0 measured; 2178 filtered out; finished in 71.63s
```

The actual Oracle84 CLI volume **156858.92917653147 mm³** differs slightly from the hand-derived expected **156858.4073464102 mm³** but passes the test's established **0.1%** analytical tolerance; it must not be described as exactly equal.

## Coverage confirmed

- **64/64 existing Test #41 regressions preserved:** independent diameter references, saved authored drawing record, public `drawing.scene`, Oracle84/85 analytic checks, undo/redo, and session-only `last_good`.
- **64/64 new dual-purpose tests:** 8 widths (85–120 mm) × 4 hole-diameter pairs (12/8, 8/12, 14/6, 6/14 mm) × deleting either Bore1 or Bore2.
- Real model parameter commands followed by `drawing.dimension.inspect`: two identities are retained; inspected Ø values follow the model (E2E-01 *engine subset*).
- Real `timeline.delete` makes precisely the deleted dimension `lost` with the last session measurement, while the other dimension remains valid (E2E-02 *engine subset*).
- Current two-view geometry through public `drawing.scene` matches the engine scene provider at each checkpoint; model undo/redo restores all expected measurements and scenes.
- Production-code equivalence to upstream pinned baseline checked under `crates`, `Cargo.toml`, `Cargo.lock`, with no patched SolidCraft production code.

## What this run **does not** establish

- It did **not** execute the four complete acceptance recipes in upstream draft PR #8288. Their full E2E acceptance status remains **NOT RUN**.
- It did **not** prove an open, already rendered UI drawing automatically updates (U2), nor that annotations and text are actually drawn on paper and refreshed atomically.
- It did **not** save a physical design file, close the file and reopen it to prove persisted `historical_last_value_mm` (E2E-04). An in-session `drawing.close` → `drawing.open` is different.
- The filtered Rust command excluded **2,178 other engine library tests**, so 128/128 is a focused success, not whole-project CI.
- No body-rename or no-feature-ID-reuse acceptance is claimed.

## Artifact and retention

GitHub Actions attached artifact **`13017575909`**, `solidcraft-w23a-test42-fork-e2e01-02-128-results`, **12,556 bytes**, reported SHA256 `42a86a3ec90e47c58bf38b5be67ee37c83506507a0fffb5f0c658eb0e821f9b2`. The original ZIP was **not** independently downloaded or hash-verified here. GitHub lists expiry **6 November 2026, 22:28:26 UTC**.

The permanent fork records in this directory preserve the result summary and all 128 distinct PASS lines. The original full job log remains available through the original GitHub Actions run according to GitHub's retention policies.

## Recommendation

The two already-executable, scoped E2E-01/E2E-02 checks are now convincingly tested. Before claiming the complete drawing workflow, await the relevant official UI, paper annotation and last-good file-persistence implementations, or CamWight's requested upstream test location. Keep official PR #8288 in Draft until acceptance criteria are runnable.
