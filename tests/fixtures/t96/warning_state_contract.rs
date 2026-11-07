// T96 (D07/D26): Engine-to-UI warning contract. CI-only injection into
// crates/engine/src/drawing_scene_tests.rs on pinned official drawings.
// These tests deliberately do NOT claim that the GUI is implemented.

fn t96_warning_for(status: &Value, dim: &Value) -> Value {
    status["dimensions"]
        .as_array()
        .expect("drawing.status dimensions array")
        .iter()
        .find(|row| row["dimension"] == *dim)
        .unwrap_or_else(|| panic!("dimension {dim} missing from drawing.status: {status}"))
        .clone()
}

/// D07: after the sheet is drawn, a valid dimension must not trigger
/// a "Needs update" or "Lost" warning merely by being present.
#[test]
fn t96_clean_dimension_has_no_false_warning() {
    let (mut s, sheet, [front, _]) = front_and_top();
    let added = run(
        &mut s,
        "drawing.dimension.add",
        json!({"view": ids::encode(front), "kind": {"length": {"edge": [50, 0, 20]}}}),
    );
    let dim = added["record"]["id"].clone();
    settled(&mut s, sheet);
    let (status, words) = status_of(&mut s);
    assert!(all_are(&words, "current"), "{status}");
    let row = t96_warning_for(&status, &dim);
    assert_eq!(row["status"], "ok", "{status}");
    assert_ne!(row["out_of_date"], true, "{status}");
    assert!(status.get("held").is_none(), "{status}");
    assert_eq!(status["waiting"], 0, "{status}");
    println!("T96_PASS=CLEAN_DIMENSION_NO_FALSE_WARNING; GUI_IMPLEMENTED=NO");
}

/// D26: a deferred drawing can hold an old dimension even while ALL views
/// are current. The UI must use the per-dimension out_of_date flag,
/// not only the count of views waiting.
#[test]
fn t96_deferred_dimension_only_waits_while_views_current() {
    let (mut s, sheet, dim_id) = held_after(|s| {
        run(s, "sketch.create", json!({"plane": "XY", "name": "Note"}));
        run(s, "sketch.finish", json!({}));
    });
    let dim = json!(ids::encode(dim_id));
    let (status, words) = status_of(&mut s);
    assert!(all_are(&words, "current"), "{status}");
    assert_eq!(status["waiting"], 0, "{status}");
    assert_eq!(status["held"], true, "{status}");
    let row = t96_warning_for(&status, &dim);
    assert_eq!(row["status"], "ok", "{status}");
    assert_eq!(row["out_of_date"], true, "{status}");
    nothing_waits_once_updated(&mut s, sheet);
    println!("T96_PASS=DEFERRED_DIMENSION_ONLY_NEEDS_UPDATE; GUI_IMPLEMENTED=NO");
}

/// D07/D26: when the actual 3D model changes with deferred updates,
/// the unchanged dimension ID is stale, not lost. Update Drawing clears
/// the warning once the paper scene has caught up.
#[test]
fn t96_deferred_model_change_marks_dimension_stale_not_lost() {
    let (mut s, sheet, dim_id) = held_after(|s| {
        let block = s.doc.features[0].id;
        run(s, "timeline.edit", json!({"feature": block.to_string(), "set": {"length": 120}}));
    });
    let dim = json!(ids::encode(dim_id));
    let (status, words) = status_of(&mut s);
    assert!(all_are(&words, "out_of_date"), "{status}");
    assert_eq!(status["held"], true, "{status}");
    assert!(status["waiting"].as_u64().unwrap_or(0) > 0, "{status}");
    let row = t96_warning_for(&status, &dim);
    assert_eq!(row["status"], "ok", "{status}");
    assert_eq!(row["out_of_date"], true, "{status}");
    nothing_waits_once_updated(&mut s, sheet);
    println!("T96_PASS=DEFERRED_MODEL_CHANGE_STALE_NOT_LOST; GUI_IMPLEMENTED=NO");
}

/// D26: deleting the real model feature loses the reference. Show the
/// dimension's ID and reason, never its historical last_good as a current
/// value. Inspect/candidate reads do not silently repair anything.
/// Undo restores the dimension and clears the lost condition.
#[test]
fn t96_deleted_feature_lost_has_reason_and_no_current_value() {
    let (mut s, sheet, [front, _]) = front_and_top();
    let added = run(
        &mut s,
        "drawing.dimension.add",
        json!({"view": ids::encode(front), "kind": {"length": {"edge": [50, 0, 20]}}}),
    );
    let dim = added["record"]["id"].clone();
    let before = run(&mut s, "drawing.dimension.inspect", json!({"dimension": dim}));
    assert_eq!(before["status"], "ok", "{before}");
    assert!(before.get("value").is_some(), "{before}");
    settled(&mut s, sheet);
    let drawing = ids::encode(s.open_drawing.expect("drawing open"));
    run(&mut s, "drawing.close", json!({}));
    let block = s.doc.features[0].id;
    run(&mut s, "timeline.delete", json!({"feature": block.to_string()}));
    run(&mut s, "drawing.open", json!({"drawing": drawing}));

    let (status, _) = status_of(&mut s);
    let row = t96_warning_for(&status, &dim);
    assert_eq!(row["status"], "lost", "{status}");
    assert!(!row["why"].as_str().unwrap_or("").is_empty(), "{status}");
    let lost = run(&mut s, "drawing.dimension.inspect", json!({"dimension": dim}));
    assert_eq!(lost["status"], "lost", "{lost}");
    assert!(lost.get("value").is_none(), "lost value misrepresented as current: {lost}");
    assert!(lost.get("text").is_none(), "lost text misrepresented as current: {lost}");
    if let Some(candidates) = lost.get("candidates") {
        assert!(candidates.is_array(), "repair leads must be structured, not guessed: {lost}");
    }
    let repeated = run(&mut s, "drawing.dimension.inspect", json!({"dimension": dim}));
    assert_eq!(repeated["status"], "lost", "inspect must not auto-repair: {repeated}");

    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "drawing.open", json!({"drawing": drawing}));
    let recovered = run(&mut s, "drawing.dimension.inspect", json!({"dimension": dim}));
    assert_eq!(recovered["status"], "ok", "{recovered}");
    println!("T96_PASS=LOST_HAS_REASON_NO_CURRENT_VALUE_UNDO_RECOVERS; GUI_IMPLEMENTED=NO");
}
