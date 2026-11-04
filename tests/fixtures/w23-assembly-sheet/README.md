# W23 assembly sheet

The test assembly for #11773 (part of #4894): a small assembly from plain shapes whose parts list,
balloons and revision table are checked against values counted by hand. Parts lists and balloons
are planned on #3874, the revision table with the title block on #3875.

`assembly.json` builds it by commands (`Session::run_script`). A step's `as` names the occurrence
its reply returns, so a test can find the holders without counting steps.

## The assembly

| Part | Part number | Shape | Placed |
|---|---|---|---|
| Base plate | BP-100 | box 120 x 80 x 10 mm | once |
| Screw | SC-M6 | cylinder Ø6 x 20 mm | 4 on the plate, and 2 in each holder |
| Holder | HL-300 | assembly | twice, the second 40 mm along Y |
| Block | BL-200 | box 20 x 20 x 30 mm | once in each holder |

## Counted by hand

- **Parts only:** BL-200 2, BP-100 1, SC-M6 8 (4 + 2 x 2).
- **First level:** BP-100 1, HL-300 2, SC-M6 4.
- **All levels:** 1 BP-100 1; 2 HL-300 2; 2.1 BL-200 1 each, 2 in all; 2.2 SC-M6 2 each, 4 in all;
  3 SC-M6 4.
- **SC-M6 purchased:** the same counts, its rows say purchased.
- **HL-300 phantom:** no row of its own; the first level is BL-200 2, BP-100 1, SC-M6 8.
- **The second holder deleted:** parts only is BL-200 1, BP-100 1, SC-M6 6.

## What runs

In `crates/engine/src/cmd/bom_tests.rs`:

- `the_assembly_sheets_parts_list_matches_the_hand_counts`
- `the_assembly_sheets_item_numbers_hold_through_edits_undo_and_reopening`
- `a_purchased_screw_keeps_the_assembly_sheets_counts`
- `a_phantom_holder_puts_what_it_holds_on_the_assembly_sheets_first_level`

In `crates/engine/src/cmd/drawing_balloon_tests.rs`, on what `drawings` already has for balloons
and revision rows:

- `balloons_on_the_assembly_sheet_follow_the_model_and_its_revision_rows_stay`: four balloons on
  the top view at 1:2 (the plate, a screw, and the block through each holder); model edits made
  with the drawing closed; the second holder moved, its arrow follows; deleted, its balloon is
  `lost`, keeps its occurrence and record and is never moved to the first holder; undo and redo;
  save and reopen; revision rows 1 and 3 after row 2 is deleted, the next still 4, and the
  drawing's record unchanged by every model edit.

## NOT RUN yet

Turned on as each piece lands on `drawings`:

- the parts list placed on a sheet, its quantities after a model edit made with the drawing closed;
- balloons showing their row's item number, a deleted holder's balloon lost (never moved or removed);
- the revision table: `drawing.revise` (#11803), revisions numbered 1, 2, 3 (#11813) and revision
  tags on a sheet (#11821);
- the sheet export after save and reopen.
