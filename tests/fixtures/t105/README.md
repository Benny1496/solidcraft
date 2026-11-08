# SolidCraft Test #105 – long-form Drawing integration, actual disk roundtrips, two profiles

## Why this test adds value
T104 tested an experimental native egui warning card, but only reopened the drawing **inside the same session**. T105 closes this gap at the native Engine level by actually saving a design on disk, loading it in a **fresh Session**, and rechecking the exact dimension IDs, values and statuses. It deliberately keeps the production Drawing UI unchanged. The source under test is **pinned** to Loohi's PR #13211 revision `84ef88b10e394c9ea9f90459c242e2033c6f7133`, with additional tests injected **only** into the isolated Actions runner.

## Three new engine tests
- `t105_repaired_edge_and_healthy_dimension_survive_disk_roundtrip_in_both_formats`: after a real fillet and an explicitly selected repair, native and JSON saves/reopens preserve the repaired reference, 80-mm value and exact dimension ID; an unrelated healthy dimension remains unchanged.
- `t105_lost_and_healthy_dimensions_remain_distinct_after_disk_load_and_update`: lost refs still show `lost`, offer repair leads, never present `last_good` as a current dimension after loading; a generic drawing update must not silently repair them. Another valid dimension stays valid.
- `t105_copy_dimension_to_second_sheet_survives_real_disk_load_with_distinct_ids`: copy a dimensioned view to **Sheet 2**. After model fillet and saving/reopening in both file formats, original and copied dimensions remain distinctly identified, belong to different views and stay `lost` rather than auto-bind to incorrect geometry.

## Broad checks (substantive workload, not artificial waiting)
1. Rustfmt, strict Clippy `-D warnings` across Engine/UI.
2. CI-profile compile and execute 3 new disk/native Engine cases, 6 T104 cases, 7 T103 native interactive GUI cases and four sheet controls. The entire UI Drawing subset and Engine Drawing subset run too.
3. **Full Engine native library suite** (thousands of tests). Existing failures documented in `tests/KNOWN_FAILING_TESTS` are explicitly **skipped and reported, not falsely marked passed**. Any new failure is fatal.
4. Recompile and execute the same critical Drawing acceptance checks under the **normal Rust dev/test profile**. This checks both CI-optimized and debug-assertion configurations. Additional full native GUI Drawing suite and Engine Drawing suite in the dev profile.

The target is a longer than one-hour **meaningful workload**, with a hard CI timeout of 125 minutes and NO sleep, looping for time, deliberate throttling or idle padding. An exact 60+ minutes cannot be guaranteed: if the real workload finishes sooner it should report honestly rather than be delayed. If the job times out, preserve artifacts and identify the expensive step; do not claim tests passed.

The large library suite's pre-existing known failures are never hidden from the log. This is not an official E2E test of the merged production UI and does not prove visual rendering in an installed desktop app. All source changes are test-only; the official PRs remain untouched.

Original logs, summaries, debug/CI test results and the test-only source patch are uploaded even on failure, then archived to Home Assistant by existing automation.
