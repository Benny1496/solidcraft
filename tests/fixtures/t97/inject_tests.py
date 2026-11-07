#!/usr/bin/env python3
"""Inject T97 regression tests only into temporary pinned upstream runner source."""
from pathlib import Path

target = Path("crates/engine/src/cmd/drawing_dimension_tests.rs")
source = Path("../overlay/tests/fixtures/t97/repair_to_status_contract.rs")
original = target.read_text(encoding="utf-8")
addition = source.read_text(encoding="utf-8")
for anchor in (
    "fn read_once()", "fn lose_the_edge(", "fn front_view()",
    "fn edge_index(", "fn inspect_of(", "fn reads(", "fn add(",
    "fn a_repair_is_one_undo_step_and_undone_is_lost_again()",
    "fn an_ambiguous_length_is_repaired_to_one_piece()",
):
    assert anchor in original, f"T97 requires adaptation to changed upstream helper: {anchor}"
for name in (
    "t97_status_row", "t97_front_edge_candidate",
    "t97_lost_warning_candidate_repair_undo_redo_is_consistent",
    "t97_ambiguous_warning_waits_for_explicit_candidate_choice",
    "t97_invalid_repair_preserves_lost_warning_and_document",
    "t97_repair_one_dimension_does_not_touch_another",
):
    assert ("fn " + name + "(") not in original, name
    assert addition.count("fn " + name + "(") == 1, name
assert "GUI_IMPLEMENTED=NO" in addition
target.write_text(original.rstrip("\n") + "\n\n" + addition.rstrip("\n") + "\n", encoding="utf-8")
print("T97_INJECTED=4_NEW_STATUS_REPAIR_INTEGRATION_TESTS; PRODUCT_CODE_UNCHANGED", flush=True)
