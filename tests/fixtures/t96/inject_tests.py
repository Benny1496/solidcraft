#!/usr/bin/env python3
"""Inject T96 into an ephemeral, pinned official source checkout only."""
from pathlib import Path
target = Path("crates/engine/src/drawing_scene_tests.rs")
source = Path("../overlay/tests/fixtures/t96/warning_state_contract.rs")
original = target.read_text(encoding="utf-8")
addition = source.read_text(encoding="utf-8")
for anchor in (
    "fn front_and_top()", "fn front_edge(", "fn settled(",
    "fn held_after(", "fn nothing_waits_once_updated(",
    "fn status_of(", "fn status_says_where_each_dimension_stands()",
):
    assert anchor in original, f"Upstream API changed, update T96: {anchor}"
for name in (
    "t96_warning_for",
    "t96_clean_dimension_has_no_false_warning",
    "t96_deferred_dimension_only_waits_while_views_current",
    "t96_deferred_model_change_marks_dimension_stale_not_lost",
    "t96_deleted_feature_lost_has_reason_and_no_current_value",
):
    assert original.count("fn " + name + "(") == 0, name
    assert addition.count("fn " + name + "(") == 1, name
assert "GUI_IMPLEMENTED=NO" in addition, "Do not misrepresent contract tests as GUI E2E"
target.write_text(original.rstrip("\n") + "\n\n" + addition.rstrip("\n") + "\n", encoding="utf-8")
print("T96_ENGINE_WARNING_CONTRACT_INJECTED=4_NEW_TESTS; PRODUCT_FILES_UNCHANGED_ON_GITHUB", flush=True)
