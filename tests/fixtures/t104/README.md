# SolidCraft Test #104 — multi-dimension GUI safety and repair regression

Source: pinned unmerged Loohi PR #13211 at `84ef88b10e394c9ea9f90459c242e2033c6f7133`. Existing proven T103 UI prototype is injected **only under `#[cfg(test)]` in a temporary Actions runner checkout**. Never change upstream or production UI. Nathan's D07/D26 implementation remains a separate pending task.

Six new native egui scenarios:
1. Two simultaneously out-of-date dimensions are both listed; one user-initiated Drawing Update brings both current without changing their IDs.
2. Mixed lost + stale dimensions: the update action on the stale one must not silently repair the lost reference.
3. Explicit clicked repair remains valid after closing and reopening the Drawing **in the same session** (not a disk-save proof).
4. Selected repair candidate is forwarded exactly as returned by the engine, no guessing.
5. Three repeated Undo/Redo cycles of the repair never produce ghost warnings.
6. Status mapping preserves dimension IDs and error reasons, distinguishes lost/ambiguous/stale, doesn't turn last_good into a current measurement and doesn't warn for healthy dimensions.

Runs all 6 new cases separately with dedicated logs, the prior 7 T103 cases, 4 sheet GUI controls and complete Drawing UI/Engine test subsets, strict formatting and clippy -D warnings, native compilation. For each failure the original logs and runner-only patch are uploaded as an original Actions ZIP. No artificial delays; 90-minute job cap. The 6 and older checks are also inside the full GUI suite — do not double count.

Do not call CI-only success completion of the official D07/D26 Drawing interface. Unexpected CI failures must first be triaged for test setup/prototype errors. Home Assistant archive runs independently once completed.
