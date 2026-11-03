# Assembly sheet acceptance, stage A (fork-only draft)

Draft tests for the assembly-sheet acceptance case CamWight offered in
https://github.com/tangency-labs/solidcraft/pull/8288#issuecomment-6599897742
(a child issue of #4894, building on #3874). **Not a pull request and not part of #8288.**

assembly_sheet_tests.rs is appended, unchanged, to
crates/engine/src/cmd/drawing_balloon_tests.rs of upstream drawings at a pinned commit by
.github/workflows/solidcraft-assembly-sheet-58.yml. No production code is changed.

## The test assembly (countable by hand)

- base plate BP-100, 120 x 80 x 10 mm, placed once;
- screw SC-M6, placed four times on the plate;
- holder HL-300, placed twice, each holding block BL-200 and two of the same screws.

Parts only: block 2, plate 1, screw 8 (4 + 2 x 2). First level: plate 1, holder 2, screw 4.
All levels: 1 plate; 2 holder (2.1 block 1 each / 2 in all, 2.2 screw 2 each / 4 in all); 3 screw 4.

## Stage A (runs on drawings today)

1. assembly_sheet_parts_list_counts_by_hand_and_keeps_its_item_numbers: assembly.bom in every
   view against the hand counts; assembly.bom.renumber (10, 20, 30); a new part takes 31 and keeps
   it through undo/redo; a deleted holder keeps every number; a second view numbered on its own;
   save and reopen in JSON and container formats.
2. assembly_sheet_balloons_and_revisions_follow_the_model_and_the_file: four balloons on the top
   view (plate, screw, each holder's block through its occurrence path); model edits only with the
   drawing closed; a moved holder's balloon follows; a deleted holder's balloon reads lost, keeps
   its occurrence and record, never moves to the other holder; undo/redo; save and reopen; revision
   rows (1, 3, next 4) and the drawing's record unchanged by every model edit.

## Stage B (NOT RUN: waits for the rest of #3874)

The parts list and revision table placed on the sheet, a balloon's item number from its parts-list
row (pink when lost), drawing.revise and revision tags, and the sheet export after reopening.
