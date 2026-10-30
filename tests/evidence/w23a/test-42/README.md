# SolidCraft W23a – Test #42: runnable E2E-01/E2E-02 engine subsets

## Purpose

This separate fork-only branch holds real, reproducible Rust integration tests for the **two subsets CamWight explicitly identified as runnable now** in [#4892](https://github.com/tangency-labs/solidcraft/issues/4892#issuecomment-6522518442) and review of [draft PR #8288](https://github.com/tangency-labs/solidcraft/pull/8288). It does **not** modify or claim completion of that official acceptance PR.

The code is derived from the 64/64 successful externally executed Test #41 baseline ([run 41113166666](https://github.com/Benny1496/ESP32-Victron-SolarSense-Bridge/actions/runs/41113166666)). Test #42 retains all 64 and adds 64 new real model/drawing probes (**128 tests**).

## Executable acceptance scope

- **E2E-01, engine subset:** real `parameters.change {name,expression}` commands edit Oracle84 plate width and two independent hole diameters. Read `drawing.dimension.inspect`, assert `status: ok`, new diameter, unchanged dimension IDs, unchanged other bore, and verify `drawing.scene` two-view lines against the incremental scene provider.
- **E2E-02, engine subset:** real `timeline.delete {feature}` on **Bore1 or Bore2** while drawing is closed. Assert `status: lost` only for the deleted bore, unchanged ID and separate `last_good` *within the same session*, no live measurement value, unaffected other bore, and correct two-view changes. Undo, redo and undo again must restore expected states, without reassigning a reference.
- Restore the initial Oracle84 design through three parameter undos, then redo the edits; verify real 3D CAD volumes against analytic oracle and corresponding actual public 2D geometry at each checkpoint. Stored authored drawing record must remain identical.
- New matrix: **8 nonbaseline widths × 4 nonbaseline bore pairs × 2 deletion directions = 64 new independent Rust tests**. Existing Test41 suite: 32 named tests and 32 parameter-matrix tests.
- 128 tests total; **not** 128 test suites and not a full-project CI pass.

## Source and execution provenance

The test module is stored **in this SolidCraft fork**:
`tests/evidence/w23a/test-42/w23a_t42_integration_tests.rs`.

Until Actions in this fork work again, the execution is run by the external ESP32 Actions repository, which **checks out a pinned commit from this fork** and copies the module into the temporary engine checkout. The native `cargo test --locked -p solidcraft-engine --lib w23a_t42_ -- --nocapture` then compiles and runs the real SolidCraft code, with no upstream production changes. The separate runner workflow preserves CI evidence including the exact tested source SHA.

**Evidence should not be misrepresented:** The GitHub Actions run exists under the ESP32 runner repository; this branch provides canonical test source and an archived result summary. The build's checked-out fork commit is the authority.

## Explicitly not proved by Test #42

- Complete official E2E acceptance flows in #8288 (still NOT RUN).
- Automatic U2/UI refresh while a drawing stays visible.
- On-paper rendered dimension text, balloons and annotations updating atomically with lines.
- Real design-file save/close/reopen, including persistent last-good values (E2E-04).
- Body rename identity, non-reused feature IDs and later R1/R2 functionality as specified in #4892.

## Operational limits and next step

CI job maximum **90 min**; native Rust test group target below **75 min**; no artificial waiting. Prefer a shared compilation. After a verified completion archive link, tested commit, original result line and test logs to this branch before claiming PASS/FAIL.

No comment or status change in draft PR #8288 is implied by this test-only branch.
