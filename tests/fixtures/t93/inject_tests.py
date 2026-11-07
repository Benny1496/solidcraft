#!/usr/bin/env python3
"""Stage exactly the four new T93 cases into temporary pinned upstream checkouts."""
from pathlib import Path
for target_name, source_name, functions, anchors in [
    ("crates/engine/src/drawing_scene_tests.rs", "open_drawing_regression.rs",
     ["t93_open_sheet_guards_model_edits_and_restores_live_dimension"],
     ["fn front_and_top()", "fn front_edge(", "fn settled(", "fn a_job_is_dropped_when_its_view_or_the_model_changed_unannounced()"]),
    ("crates/engine/src/cmd/drawing_dimension_tests.rs", "hole_dimensions_regression.rs",
     ["t93_moving_hole_updates_real_linear_dimension_value",
      "t93_deleting_hole_marks_attached_dimension_lost_not_current",
      "t93_hole_measure_and_lost_warning_survive_file_save_and_reopen"],
     ["fn front_view()", "fn round_index(", "fn reopened(", "fn reads(", "fn add(", "fn a_lost_dimension_keeps_its_last_good_value_after_reopening()"]),
]:
    target = Path(target_name)
    source = Path("../overlay/tests/fixtures/t93") / source_name
    original = target.read_text(encoding="utf-8")
    addition = source.read_text(encoding="utf-8")
    for anchor in anchors:
        assert anchor in original, f"Source changed: {target_name}: {anchor}"
    for function in functions:
        needle = "fn " + function + "()"
        assert needle not in original, f"Already present upstream: {needle}"
        assert addition.count(needle) == 1, f"Missing or duplicated: {needle}"
    target.write_text(original.rstrip("\n") + "\n\n" + addition.rstrip("\n") + "\n", encoding="utf-8")
    print(f"T93_INJECTED={target_name}; ADDED_TESTS={len(functions)}", flush=True)
print("T93_FOUR_NEW_TEST_CASES_INJECTED=PASS", flush=True)
