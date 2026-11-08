# Test #99 — Proof-of-concept of the Drawing status pill in native egui

**Purpose:** Move beyond engine-only verification and the T98 UI gap diagnosis: test an actual rendered, clickable status-pill prototype that reads `drawing.status` on Loohi's **unmerged** sheet page. This is **entirely test-runner-only**, behind `#[cfg(test)]`, and **does not implement or submit product UI**. Respect Nathan-Dane's assigned D07/D26 work and await his design integration / child issue before permanent code.

**Pinned source:** Loohi/solidcraft PR #13211 head `84ef88b10e394c9ea9f90459c242e2033c6f7133`.

**Three new native egui UI proof-of-concept checks:**
1. A fully current drawing paints the `All current` status pill; a real click opens the card, verified by a published screen rectangle and the last-frame inspect state.
2. A deferred drawing with all views current (`waiting=0`) but dimension `out_of_date=true` shows `1 need attention`, ID and `Needs update`; clicking opens the card, and running `drawing.update` clears the warning.
3. A deleted model feature causes `Lost` with the affected dimension's ID, reason, text glyph symbol `✕`, and a clickable card; undo clears the warning. No invented live measurement or fake repair operation. The prototype reports `real_repair_button=false` explicitly.

**Control tests (4):** upstream `the_plate_sheet_draws_its_lines_and_text`, `a_views_state_is_in_words`, `fit_sheet_shows_the_whole_sheet_and_zoom_stays_in_range`, `no_drawing_open_shows_the_note` — these check Loohi's existing sheet interactions. **Seven total** when run. Positive prototype assertions mean **feasibility**, not an integrated feature, and no real repair-click E2E.

**Isolation:** GitHub runner checks out Loohi's exact PR head read-only, overlays one Rust file `t99_warning_poc.rs` and patches only `drawing_sheet_page.rs` and its existing tests in a *temporary checkout*. The official repository, Loohi's branch, and our production code remain unchanged. The prototype is visible only to the test binary, not users. 90m timeout, no secrets / write token, ZIP logs and a CI patch archived.

**Status:** Prepared. Test results require actual Actions execution. Do not claim a working integrated D07 UI or production-ready repair.
