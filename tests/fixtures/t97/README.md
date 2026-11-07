# SolidCraft Test #97 – repair from warning status to selected dimension

**Status: prepared, execution only after separate workflow push**. Follows [Nathan-Dane's assignment](https://github.com/tangency-labs/solidcraft/pull/5349#issuecomment-6712862090) and the agreed scope: render lost/out-of-date warnings, explain why and show repair options.

## What is new relative to T96

T96 proved status data classification and lost-reference safety. T97 follows an **end-to-end command path from the status result to inspect candidates and explicit user-selected repair**, including undo/redo, refusal of invalid picks and isolation of two dimension IDs.

- **T97/1:** `drawing.status` and `drawing.dimension.inspect` agree on one lost dimension and provide why, historical value, and a valid candidate. Merely inspecting cannot mutate; repair/undo/redo all affect the *same* ID.
- **T97/2:** Ambiguous split edge has multiple candidates, none auto-selected. Explicit selection repairs to 35 mm; undo restores ambiguity.
- **T97/3:** Invalid repair candidate is rejected. Original drawing, undo history, affected dimension ID and lost-warning state remain intact.
- **T97/4:** A repair targets the affected dimension only; another valid dimension retains its value/status through repair and undo/redo.
- **4 independent upstream regression controls:** `fillet_then_repair_reads_again_with_the_same_id_and_place`, `a_repair_is_one_undo_step_and_undone_is_lost_again`, `a_bad_repair_pick_changes_nothing`, `an_ambiguous_length_is_repaired_to_one_piece`.

**Eight targeted Rust engine tests**. Pinned to official read-only `tangency-labs/solidcraft:drawings@c8cbe43dd9ded5b6c0f5fbd0db750bb673bd0fa7`. Fork branch `Benny1496/solidcraft:tests/solidcraft-t97`. GitHub CI injects Rust fixture **only inside temporary runner checkout**, leaving upstream and fork product sources unmodified. 90-minute maximum job, Rustfmt/Clippy/compile, original log bundle uploaded regardless of outcome; Home Assistant auto-archive expected after completion.

**Important:** This is a repair-interface engine contract, NOT a functional native GUI test. Design PR #13159 D07/D26 remains draft; do not claim the status pill, buttons or repair panel exist. After relevant child issue and product UI implementation a separate GUI test must exercise real egui controls and rendered messages.

