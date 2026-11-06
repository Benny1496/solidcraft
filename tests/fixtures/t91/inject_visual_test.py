#!/usr/bin/env python3
"""Add visual export evidence test to an isolated exact-commit CI checkout."""
from pathlib import Path

target = Path("crates/engine/src/cmd/drawing_export_tests.rs")
source = Path("../overlay/tests/fixtures/t91/visual_export_regression.rs")
body = target.read_text(encoding="utf-8")
snippet = source.read_text(encoding="utf-8")
name = "t91_a_3d_edit_redraws_the_2d_sheet_and_keeps_its_dimension_in_png_and_svg"
assert body.count("fn front_and_top") == 0, "The helper belongs to drawing_scene_tests"
for fragment in (
    "use crate::drawing_scene::tests::{front_and_top, nest};",
    "fn a_sheet_exports_as_its_scenes_svg()",
    "fn a_sheet_exports_as_its_scenes_png()",
    "fn a_path_gets_the_svg_written_to_it()",
):
    assert fragment in body, f"Unexpected upstream test structure: {fragment}"
assert name not in body, "A test with this exact name already exists"
assert snippet.count("fn " + name + "()") == 1
target.write_text(body.rstrip("\n") + "\n\n" + snippet.rstrip("\n") + "\n", encoding="utf-8")
print("T91_VISUAL_TEST_INSERTION=PASS; " + name, flush=True)
