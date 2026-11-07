/// Shared T93 plate: a real cut hole at X=40 mm and a horizontal dimension
/// from the box origin to that hole's actual circle centre (not a fixed point).
fn t93_hole_plate_with_centre_measure() -> (Session, u64, Value) {
    let (mut s, view) = front_view();
    let drawing = s.drawing("test").unwrap().id;
    run(&mut s, "drawing.close", json!({}));
    run(
        &mut s,
        "solid.cylinder",
        json!({
            "name": "Hole",
            "radius": 6,
            "height": 42,
            "base": [40, -1, 15],
            "axis": "Y",
            "operation": "cut",
            "targets": ["Box"]
        }),
    );
    run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));
    let centre = json!({
        "centre": {
            "body": "Box",
            "index": round_index(&s, [40.0, 0.0, 15.0], 6.0)
        }
    });
    let measure = add(
        &mut s,
        view,
        json!({"linear": {"a": [0, 0, 0], "b": centre, "direction": "horizontal"}}),
    );
    reads(&measure, 40.0, "40.00");
    (s, drawing, measure["dimension"].clone())
}

/// T93/2: Move the actual cylinder feature 15 mm, not just a point used
/// for annotation. Verify that the old dimension ID resolves to X=55 mm.
#[test]
fn t93_moving_hole_updates_real_linear_dimension_value() {
    let (mut s, drawing, dim) = t93_hole_plate_with_centre_measure();
    let before = run(&mut s, INSPECT, json!({"dimension": dim}));
    reads(&before, 40.0, "40.00");

    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "timeline.edit", json!({
        "feature": "Hole",
        "set": {"base": [55, -1, 15]}
    }));
    run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));
    let after = run(&mut s, INSPECT, json!({"dimension": dim}));
    assert_eq!(after["status"], "ok", "moving the hole lost the original dimension: {after}");
    reads(&after, 55.0, "55.00");
    assert_eq!(before["dimension"], after["dimension"], "the existing measure ID must survive");
    println!("T93_CASE_2=PASS; HOLE_CENTER_X_BEFORE=40; HOLE_CENTER_X_AFTER=55; DIMENSION_VALUE=55; DIMENSION_ID_STABLE=YES");
}

/// T93/3: Delete the actual cut feature and inspect the *same* dimension.
/// It must not present the old 40 mm as a current measurement.
#[test]
fn t93_deleting_hole_marks_attached_dimension_lost_not_current() {
    let (mut s, drawing, dim) = t93_hole_plate_with_centre_measure();
    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "timeline.delete", json!({"feature": "Hole"}));
    run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));

    let lost = run(&mut s, INSPECT, json!({"dimension": dim}));
    assert_eq!(lost["status"], "lost", "{lost}");
    assert!(lost.get("value").is_none() && lost.get("text").is_none(), "old measure must never appear as current: {lost}");
    assert_eq!(lost["last_good"], 40.0, "last known value must be marked historical: {lost}");

    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));
    let restored = run(&mut s, INSPECT, json!({"dimension": dim}));
    reads(&restored, 40.0, "40.00");
    println!("T93_CASE_3=PASS; DELETE_HOLE=LOST; VALUE_NOT_CURRENT=YES; LAST_GOOD=40; UNDO_RESTORES=YES");
}

/// T93/4: Persist both a valid hole-centre measurement and a deleted-hole
/// warning into fresh sessions. Test the native design format and JSON.
#[test]
fn t93_hole_measure_and_lost_warning_survive_file_save_and_reopen() {
    for format in [format!("valid.{}", solidcraft_io::DESIGN_EXT), "valid.json".to_owned()] {
        let (mut s, drawing, dim) = t93_hole_plate_with_centre_measure();
        let mut opened = reopened(&mut s, drawing, &format);
        let read = run(&mut opened, INSPECT, json!({"dimension": dim}));
        reads(&read, 40.0, "40.00");
        assert!(!opened.is_dirty(), "{format}: inspecting must not change a saved design");
    }
    for format in [format!("lost.{}", solidcraft_io::DESIGN_EXT), "lost.json".to_owned()] {
        let (mut s, drawing, dim) = t93_hole_plate_with_centre_measure();
        run(&mut s, "drawing.close", json!({}));
        run(&mut s, "timeline.delete", json!({"feature": "Hole"}));
        run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));
        let before_save = run(&mut s, INSPECT, json!({"dimension": dim}));
        assert_eq!(before_save["status"], "lost", "{before_save}");
        let mut opened = reopened(&mut s, drawing, &format);
        let read = run(&mut opened, INSPECT, json!({"dimension": dim}));
        assert_eq!(read["status"], "lost", "{format}: {read}");
        assert!(read.get("value").is_none() && read.get("text").is_none(), "{format}: stale measure displayed as current: {read}");
        assert_eq!(read["last_good"], 40.0, "{format}: historical value must persist: {read}");
        assert!(!opened.is_dirty(), "{format}: saved state must stay clean");
    }
    println!("T93_CASE_4=PASS; VALID_AND_LOST_MEASURE_PERSIST=YES; FORMATS=NATIVE_AND_JSON; LAST_GOOD_RESTORED=YES");
}
