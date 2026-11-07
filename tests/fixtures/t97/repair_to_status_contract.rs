// Test 97: user-facing repair-flow engine contract for D07/D26.
// Runner-only injection into crates/engine/src/cmd/drawing_dimension_tests.rs.
// A green result is NOT a real GUI acceptance test.

fn t97_status_row(s: &mut Session, dimension: &Value) -> Value {
    let reply = run(s, "drawing.status", json!({}));
    reply["dimensions"]
        .as_array()
        .unwrap_or_else(|| panic!("status dimensions missing: {reply}"))
        .iter()
        .find(|r| r["dimension"] == *dimension)
        .unwrap_or_else(|| panic!("dimension {dimension} missing: {reply}"))
        .clone()
}

fn t97_front_edge_candidate(s: &Session) -> Value {
    json!({"edge": {"body": "Box", "index": edge_index(s, [0.0, 0.0, 25.0], [80.0, 0.0, 25.0])}})
}

/// One real lost dimension appears in status with its ID/reason and in inspect
/// with historical value and candidate picks. Only explicit repair changes it.
/// The same dimension is ok afterwards, lost on Undo, ok again on Redo.
#[test]
fn t97_lost_warning_candidate_repair_undo_redo_is_consistent() {
    let (mut s, drawing, dimension) = read_once();
    lose_the_edge(&mut s);
    run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));
    let id = json!(ids::encode(dimension));
    let lost = inspect_of(&mut s, dimension);
    assert_eq!(lost["status"], "lost", "{lost}");
    assert!(lost.get("value").is_none() && lost.get("text").is_none(), "{lost}");
    assert_eq!(lost["last_good"], json!(80.0), "{lost}");
    assert!(!lost["why"].as_str().unwrap_or("").is_empty(), "{lost}");
    let row = t97_status_row(&mut s, &id);
    assert_eq!(row["status"], "lost", "{row}");
    assert!(!row["why"].as_str().unwrap_or("").is_empty(), "{row}");
    let pick = t97_front_edge_candidate(&s);
    assert!(lost["candidates"].as_array().is_some_and(|a| a.contains(&pick)), "{lost}");

    let unchanged = s.drawing("test").unwrap().to_json().unwrap();
    let steps = s.undo.len();
    let _ = inspect_of(&mut s, dimension);
    let _ = t97_status_row(&mut s, &id);
    assert_eq!(s.drawing("test").unwrap().to_json().unwrap(), unchanged, "reading a repair card must not alter the drawing");
    assert_eq!(s.undo.len(), steps, "inspecting must not create undo steps");

    let repaired = run(&mut s, REPAIR, json!({"dimension": id, "slot": "edge", "pick": pick}));
    reads(&repaired, 80.0, "80.00");
    assert_eq!(repaired["dimension"], id);
    assert_eq!(repaired["view"], lost["view"]);
    assert_eq!(s.undo.len(), steps + 1, "a confirmed repair is one undo step");
    assert_eq!(t97_status_row(&mut s, &id)["status"], "ok");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(t97_status_row(&mut s, &id)["status"], "lost");
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(t97_status_row(&mut s, &id)["status"], "ok");
    println!("T97_PASS=LOST_STATUS_REPAIR_UNDO_REDO_STABLE_ID; GUI_IMPLEMENTED=NO");
}

/// Ambiguous differs from lost. A real notch generates several candidates.
/// The user-selected one must be accepted; status returns ok and Undo restores
/// ambiguous. Merely listing candidates never chooses one automatically.
#[test]
fn t97_ambiguous_warning_waits_for_explicit_candidate_choice() {
    let (mut s, drawing, dimension) = read_once();
    run(&mut s, "solid.box", json!({"length": 10, "width": 5, "height": 5, "corner": [35, -1, 26], "operation": "cut", "targets": ["Box"]}));
    run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));
    let id = json!(ids::encode(dimension));
    let ambiguous = inspect_of(&mut s, dimension);
    assert_eq!(ambiguous["status"], "ambiguous", "{ambiguous}");
    assert_eq!(t97_status_row(&mut s, &id)["status"], "ambiguous");
    let picks = ambiguous["candidates"].as_array().expect("candidate list");
    assert!(picks.len() >= 2, "the UI must offer a choice: {ambiguous}");
    let chosen = json!({"edge": {"body": "Box", "index": edge_index(&s, [0.0, 0.0, 30.0], [35.0, 0.0, 30.0])}});
    assert!(picks.contains(&chosen), "{ambiguous}");
    let doc = s.drawing("test").unwrap().to_json().unwrap();
    let steps = s.undo.len();
    let again = inspect_of(&mut s, dimension);
    assert_eq!(again["candidates"], ambiguous["candidates"]);
    assert_eq!(s.drawing("test").unwrap().to_json().unwrap(), doc);
    assert_eq!(s.undo.len(), steps);
    let result = run(&mut s, REPAIR, json!({"dimension": id, "slot": "edge", "pick": chosen}));
    reads(&result, 35.0, "35.00");
    assert_eq!(t97_status_row(&mut s, &id)["status"], "ok");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(t97_status_row(&mut s, &id)["status"], "ambiguous");
    println!("T97_PASS=AMBIGUOUS_REQUIRES_EXPLICIT_CHOICE; GUI_IMPLEMENTED=NO");
}

/// A stale/invalid repair pick must never mutate a drawing, replace its
/// original dimension ID, or clear a lost warning in the status card.
#[test]
fn t97_invalid_repair_preserves_lost_warning_and_document() {
    let (mut s, drawing, dimension) = read_once();
    lose_the_edge(&mut s);
    run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));
    let id = json!(ids::encode(dimension));
    let lost = inspect_of(&mut s, dimension);
    assert_eq!(lost["status"], "lost");
    let before = s.drawing("test").unwrap().to_json().unwrap();
    let steps = s.undo.len();
    let bad = json!({"dimension": id, "slot": "edge", "pick": {"edge": {"body": "Box", "index": 999}}});
    let message = s.execute(REPAIR, &bad).expect_err("invalid edge must be refused").to_string();
    assert!(message.contains("has no edge 999"), "{message}");
    assert_eq!(s.drawing("test").unwrap().to_json().unwrap(), before);
    assert_eq!(s.undo.len(), steps);
    assert_eq!(t97_status_row(&mut s, &id)["status"], "lost");
    let still = inspect_of(&mut s, dimension);
    assert_eq!(still["candidates"], lost["candidates"]);
    assert_eq!(still["dimension"], id);
    println!("T97_PASS=INVALID_REPAIR_REJECTED_WARNING_PERSISTS; GUI_IMPLEMENTED=NO");
}

/// With more than one dimension the status-card Repair action must target
/// only the affected ID. An unaffected dimension remains valid before and
/// after repair, Undo and Redo.
#[test]
fn t97_repair_one_dimension_does_not_touch_another() {
    let (mut s, view) = front_view();
    let drawing = s.drawing("test").unwrap().id;
    let affected = add(&mut s, view, json!({"length": {"edge": [40, 0, 30]}}))["dimension"].clone();
    let other = add(&mut s, view, json!({"length": {"edge": [40, 0, 0]}}))["dimension"].clone();
    run(&mut s, "drawing.close", json!({}));
    lose_the_edge(&mut s);
    run(&mut s, "drawing.open", json!({"drawing": ids::encode(drawing)}));
    let lost = run(&mut s, INSPECT, json!({"dimension": affected}));
    let still_ok = run(&mut s, INSPECT, json!({"dimension": other}));
    assert_eq!(lost["status"], "lost", "{lost}");
    assert_eq!(still_ok["status"], "ok", "{still_ok}");
    assert_eq!(t97_status_row(&mut s, &affected)["status"], "lost");
    assert_eq!(t97_status_row(&mut s, &other)["status"], "ok");
    let pick = t97_front_edge_candidate(&s);
    assert!(lost["candidates"].as_array().is_some_and(|a| a.contains(&pick)), "{lost}");
    run(&mut s, REPAIR, json!({"dimension": affected, "slot": "edge", "pick": pick}));
    assert_eq!(t97_status_row(&mut s, &affected)["status"], "ok");
    assert_eq!(t97_status_row(&mut s, &other)["status"], "ok");
    assert_eq!(run(&mut s, INSPECT, json!({"dimension": other})), still_ok, "unaffected dimension changed");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(t97_status_row(&mut s, &affected)["status"], "lost");
    assert_eq!(t97_status_row(&mut s, &other)["status"], "ok");
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(t97_status_row(&mut s, &affected)["status"], "ok");
    assert_eq!(t97_status_row(&mut s, &other)["status"], "ok");
    println!("T97_PASS=REPAIR_TARGET_ID_ISOLATED_FROM_OTHER_DIMENSIONS; GUI_IMPLEMENTED=NO");
}
