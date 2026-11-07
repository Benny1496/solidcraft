#!/usr/bin/env python3
"""Insert T92's native policy regression in a disposable, pinned GitHub runner checkout."""
from pathlib import Path
target = Path("crates/engine/src/drawing_scene_tests.rs")
source = Path("../overlay/tests/fixtures/t92/automatic_vs_deferred.rs")
text = target.read_text(encoding="utf-8")
extra = source.read_text(encoding="utf-8")
name = "fn t92_automatic_vs_deferred_3d_to_2d_dimension_updates()"
for required in (
    "fn front_and_top()",
    "fn front_edge(",
    "fn settled(",
    "fn listed(",
    "fn bytes(",
    "fn bytes_of(",
    "fn group_of(",
    "fn a_deferred_drawing_keeps_its_lines_until_it_is_updated()",
    "fn a_held_deferred_drawing_exports_the_same_from_a_scratch_copy()",
):
    assert required in text, f"Changed source fixture: {required}"
assert name not in text
assert extra.count(name) == 1
target.write_text(text.rstrip("\n") + "\n\n" + extra.rstrip("\n") + "\n", encoding="utf-8")
print("T92_TEST_INJECTED=PASS")
