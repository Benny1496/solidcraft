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

## Recipes, NOT RUN yet

`cases.json` holds recipes 3 to 6 of #11773, turned on as each piece lands on `drawings`. Every
model edit in them is made with the drawing closed, then the drawing is opened again.

- **W23-AS-03, the parts list on the sheet** (waits for #3874's P8): the first level numbered 1, 2,
  3 with the script's descriptions; the second holder deleted, HL-300 reads 1 and the numbers stay.
- **W23-AS-04, balloons** (waits for #3874's balloon item numbers): one balloon through each holder
  to its block, both showing item 2; the second holder deleted, its balloon is lost (pink), never
  moved to the first holder or removed; undone, both are ok again.
- **W23-AS-05, the revision table** (waits for #11803 `drawing.revise`, #11813 numbering, #11821
  revision tags, #3875's table): revisions 1, 2, 3; 2 deleted leaves 1 and 3 with 4 next; a tag on
  3; unchanged by a model edit and through save and reopen.
- **W23-AS-06, the sheet export** (waits for the three above and text on the sheet, #4890): the
  same SVG bytes before saving and after opening again; PNG, DXF and PDF as each writer lands.

`validate.py` checks the recipes without running SolidCraft: it works the hand counts out again
from `assembly.json`, checks every model edit is made with the drawing closed, that each recipe's
expected values follow from the script (quantities, descriptions, item numbers, revisions), that
everything placed fits an A3 sheet, and that the native tests above are in their files. It never
reports a recipe as passed.

```
python3 tests/fixtures/w23-assembly-sheet/validate.py
```
