#!/usr/bin/env python3
"""T107 cross-version source CONTRACT audit, not a product runtime test.

Read-only, exactly pinned upstream drawings and CamWight PR #13055 source trees.
Detect whether newer SheetScene produces annotation ids that PR's on_sheet
still interprets with the old view/dimension-only assumption.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

PR = "5c17c14947d9422a0610fe71fd2a8c21dc4c5417"
DRAWINGS = "773d5d7fbc3b2e03199b58d0d2748fc162198495"


def main():
    if len(sys.argv) != 4:
        raise SystemExit("usage: t107_contract_audit.py PR_CHECKOUT DRAWINGS_CHECKOUT REPORT_JSON")
    pr, updated, result_path = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    heads = []
    for root, expected in [(pr, PR), (updated, DRAWINGS)]:
        actual = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
        assert actual == expected, f"source pin changed: {actual} != {expected}"
        heads.append(actual)
    original = (pr / "crates/engine/src/cmd/drawing_check.rs").read_text()
    latest = (updated / "crates/engine/src/drawing_scene.rs").read_text()
    latest_schema = (updated / "crates/engine/src/schema/drawing_scene.rs").read_text()

    results = {
        "run": "T107",
        "scope": "static compatibility analysis; cannot establish a runtime bug by itself",
        "cam_pr": 13055,
        "source_sha": heads[0],
        "upstream_drawings_sha": heads[1],
        "findings": [],
    }

    def checked(key, title, evidence, condition):
        results["findings"].append({"code": key, "title": title, "evidence": evidence, "source_mismatch_detected": bool(condition)})
        if not condition:
            raise AssertionError(f"{key}: source pattern changed; re-review the code before drawing conclusions")

    old_switch = re.search(
        r"for id in shown\.out_of_date\s*\{\s*let kind = if d\.view\(id\)\.is_some\(\)\s*"
        r"\{\s*CheckedKind::View\s*\}\s*else\s*\{\s*CheckedKind::Dimension\s*\}",
        original,
    )
    note_producer = "stale_notes" in latest and "held_by_exploded" in latest and "notes" in latest_schema
    checked(
        "NOTE_KIND_CONTRACT",
        "A newer out_of_date id can be a note but the PR uses view-or-dimension classification",
        "PR on_sheet: else CheckedKind::Dimension; upstream drawing_scene: stale_notes and held_by_exploded",
        bool(old_switch) and note_producer,
    )

    generic_update = 'drawing.update draws it from the model as it is now' in original
    deliberately_held = "held_by_exploded" in latest and "draws_exploded" in latest
    checked(
        "UPDATE_GUIDANCE_CONTRACT",
        "PR offers a generic drawing.update explanation, while upstream intentionally holds annotations in exploded views",
        "PR on_sheet: generic update reason; upstream held_by_exploded",
        generic_update and deliberately_held,
    )

    results["conclusion"] = (
        "Two static compatibility risks detected. This output is not a runtime reproduction. "
        "The native T107 probe and upstream engine tests must be inspected separately."
    )
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps(results, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
