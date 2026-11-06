#!/usr/bin/env python3
"""Inject the exact regression scenario from theironchef's PR #10431 comment.
Test-only code; does NOT repair product code and only modifies CI checkout.
https://github.com/tangency-labs/solidcraft/pull/10431#issuecomment-6688015020
"""
from pathlib import Path

file = Path("crates/engine/src/drawing_scene_tests.rs")
s = file.read_text(encoding="utf-8")
marker = (
    "/// A deferred drawing draws nothing until `drawing.update`: then from the model as it is now, until\n"
    "/// that changes. Reopened, it waits again. While it waits, every command that reads its sheet reads\n"
    "/// what the page shows and lists what is out of date, and `drawing.status` says it is held.\n"
    "#[test]\n"
    "fn a_deferred_drawing_keeps_its_lines_until_it_is_updated() {"
)
name = "a_held_deferred_drawing_exports_the_same_from_a_scratch_copy"
assert s.count(marker) == 1, "Unexpected drawing scene test source; refusing insertion"
assert name not in s, "Regression already present; refusing duplicate injection"
test = """
/// Reviewer-provided regression case for a held deferred drawing when exported
/// through the worker's scratch Session (PR #10431); expected FAIL until fixed.
#[test]
fn a_held_deferred_drawing_exports_the_same_from_a_scratch_copy() {
    let (mut s, sheet, _) = front_and_top();
    let drawing = json!({"drawing": ids::encode(s.open_drawing.unwrap())});
    run(&mut s, "drawing.set", json!({"updates": "deferred"}));
    extract(&mut s, sheet).unwrap();
    run(&mut s, "drawing.update", json!({}));
    settled(&mut s, sheet);
    run(&mut s, "drawing.close", json!({}));
    let block = s.doc.features[0].id;
    run(&mut s, "timeline.edit", json!({"feature": block.to_string(), "set": {"height": 40}}));
    run(&mut s, "drawing.open", drawing);
    settled(&mut s, sheet);
    let p = json!({"sheet": ids::encode(sheet), "format": "svg"});
    let here = run(&mut s, "drawing.export", p.clone());
    let there = s.scratch().execute("drawing.export", &p).unwrap();
    assert_eq!(there, here, "scratch background export should match synchronous export");
}
"""
file.write_text(s.replace(marker, test + "\n" + marker, 1), encoding="utf-8")
print("INJECTED_TEST=" + name)
