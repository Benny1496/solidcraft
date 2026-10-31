# W23a: nine focused native tests for E2E-01/02 and new E2E-04 file persistence subset

## Why this branch exists

CamWight's [W19d last-good persistence PR #9764](https://github.com/tangency-labs/solidcraft/pull/9764) **merged on 30 October 2026** into `drawings`. It saves a dimension's last valid value when its referenced model geometry is gone, even across a real design-file save and reopen. Previous W23a E2E-04 checks correctly remained `NOT RUN` because that functionality did not exist.

This independent **fork-only** test branch starts from `tangency-labs/solidcraft drawings@693cf911f12cd9e419456df6f253f6113696f298` (31 October 2026). It does not change the original draft [PR #8288](https://github.com/tangency-labs/solidcraft/pull/8288).

## Test contents and official-suite integration

- **Seven** native W23a tests selected from earlier 128/128 successful Test42, registered directly in `crates/engine/src/lib.rs`.
- **Two new** workshop-plate file I/O tests: real `file.save_as` and a **different fresh `Session::default()`** calling `doc.open`; one run for native `solidcraft_io::DESIGN_EXT`, one for `.json`.
- The test setup replays **25 original Oracle84 model commands** from `tests/fixtures/w23a-drawing-updates/oracle84-commands.json` (committed here), not a mocked model. `include_str!` keeps it self-contained.
- After changing Bore1 Ø10→Ø12 using `parameters.change`, the code inspects the diameter to establish `last_good = 12`, deletes Bore1 using `timeline.delete`, and verifies the `lost` state while Bore2 remains Ø10.
- The actual file roundtrip must preserve the Bore1 dimension ID, `lost` state, stored `record.last_good=12`, independent Bore2 dimension ID/value, manually authored `text.at`, and the two current 2D scenes. A lost dimension must have **no live value/text**. A subsequent inspection must not mark the reopened file dirty.

## Run

```sh
cargo test --locked -p solidcraft-engine --lib w23a_official_ -- --nocapture
```

The nine tests run as ordinary native Rust engine tests against the real SolidCraft source; the temporary external GitHub Actions runner in `Benny1496/ESP32-Victron-SolarSense-Bridge` only supplies CI capacity because fork Actions are restricted.

**At preparation time, these nine tests are NOT verified on this newer upstream commit.** The preceding 7/7 passed on an older pinned `drawings` source before PR #9764 landed; new run results must be read from the original native job log.

## Limits

This verifies the now-executable **W23a E2E-04 design-file persistence and reference subset**, not the complete official GUI E2E-04 recipe. The drawing UI still lacks automatic U2 redraw and rendering of on-paper dimension labels/balloons. Complete PR #8288 E2E-01 through E2E-04 remains `NOT RUN` until all their UI, annotation and paper workflow steps can actually execute.

No new upstream PR has been opened; prefer CamWight's feedback on the seven-test submission location before submitting.
