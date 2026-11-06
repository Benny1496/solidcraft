#!/usr/bin/env python3
"""Inject our reviewed PR #11881 W23 regression into a temporary CI checkout.

Only the checkout inside the workflow is modified; the upstream PR is not touched.
"""
from pathlib import Path

target = Path("crates/engine/src/cmd/drawing_balloon_tests.rs")
overlay = Path("../overlay/tests/fixtures/t90/w23_hide_regression.rs")
body = target.read_text(encoding="utf-8")
regression = overlay.read_text(encoding="utf-8")
name = "the_assembly_sheet_refuses_a_balloon_on_its_hidden_second_holder"
required = (
    "fn assembly_sheet()",
    "fn balloons_on_the_assembly_sheet_follow_the_model_and_its_revision_rows_stay()",
    "fn balloons_on_the_assembly_sheet_read_their_holders_row()",
    "fn a_balloon_is_refused_on_a_part_its_view_leaves_out()",
)
missing = [needle for needle in required if needle not in body]
assert not missing, f"Expected W23 and upstream regression infrastructure is missing: {missing}"
assert name not in body, "Regression already exists: refusing duplicate insertion"
assert regression.count("fn " + name + "()") == 1, "Regression body has an unexpected name or duplicate"
assert target.is_file(), "Expected native Rust balloon test file"
target.write_text(body.rstrip("\n") + "\n\n" + regression.rstrip("\n") + "\n", encoding="utf-8")
print("T90_INSERTED=" + name, flush=True)
