// T98: native egui frames on Loohi's PR #13211 (not only Engine mocks).
// Runner-only injection. T98_GAP means a confirmed missing UI behavior,
// never a passing implementation of the D07/D26 status card.

fn t98_dim_id(h: &mut Harness, front: &str) -> Value {
    let view = h.app.run("drawing.view.inspect", json!({"view": front})).unwrap();
    let dimensions = view["dimensions"].as_array().expect("view dimensions array");
    assert_eq!(dimensions.len(), 1, "fixture has exactly one dimension: {view}");
    dimensions[0].clone()
}

fn t98_drawing_id(h: &Harness) -> String {
    solidcraft_engine::doc::ids::encode(h.app.session.open_drawing.expect("drawing is open"))
}

/// Positive native-UI acceptance: a real model edit while the drawing is
/// closed is reflected in the *rendered* sheet and its existing dimension.
/// Same dimension ID, current value, 2 drawn views, not just an Engine reply.
#[test]
fn t98_native_sheet_after_3d_model_edit_keeps_updated_dimension() {
    let (mut h, front) = plate_sheet();
    let id = t98_dim_id(&mut h, &front);
    let before = h.app.run("drawing.dimension.inspect", json!({"dimension": id})).unwrap();
    assert_eq!(before["status"], "ok", "{before}");
    assert_eq!(before["value"], json!(100.0), "{before}");
    let drawing = t98_drawing_id(&h);
    let first_page = page(&mut h);
    assert_eq!(first_page["views"].as_array().map(Vec::len), Some(2), "{first_page}");
    assert!(crate::scenario::rect_of("sheet:paper").is_some(), "real paper drawn");
    h.app.run("drawing.close", json!({})).unwrap();
    let feature = h.app.session.doc.features[0].id;
    h.app.run("timeline.edit", json!({"feature": feature.to_string(), "set": {"length": 120}})).unwrap();
    h.app.run("drawing.open", json!({"drawing": drawing})).unwrap();
    settle(&mut h);
    let after = h.app.run("drawing.dimension.inspect", json!({"dimension": id})).unwrap();
    assert_eq!(after["status"], "ok", "{after}");
    assert_eq!(after["dimension"], id, "{after}");
    assert_eq!(after["value"], json!(120.0), "dimension must follow the 3D change: {after}");
    let shown = page(&mut h);
    assert_eq!(shown["views"].as_array().map(Vec::len), Some(2), "{shown}");
    assert!(shown["views"].as_array().unwrap().iter().all(|v| v["status"] == "drawn" && v["out_of_date"] == false), "{shown}");
    assert!(shown["error"].is_null(), "{shown}");
    let sheet = h.app.drawing_workspace.page.sheet.expect("sheet remains visible");
    let scene = solidcraft_engine::drawing_scene::extract(&mut h.app.session, sheet).unwrap();
    assert!(scene.scene.groups.iter().any(|g|
        g.kind == ObjectKind::Dimension && solidcraft_engine::doc::ids::encode(g.object) == id.as_str().unwrap()
    ), "updated dimension must be represented in the sheet scene");
    println!("T98_PASS=REAL_EGUI_SHEET_3D_EDIT_DIMENSION_100_TO_120_SAME_ID; UI_CARD_IMPLEMENTED=NO");
}

/// A deliberately *diagnostic* GUI gap: all views can look current while a
/// deferred dimension itself is stale. The engine can see it, but PR #13211
/// has no D07 per-dimension status card. Do not count this as GUI fixed.
#[test]
fn t98_diagnostic_deferred_dimension_only_missing_from_sheet_page() {
    let (mut h, front) = plate_sheet();
    let id = t98_dim_id(&mut h, &front);
    let drawing = t98_drawing_id(&h);
    h.app.run("drawing.set", json!({"updates": "deferred"})).unwrap();
    h.app.run("drawing.update", json!({})).unwrap();
    settle(&mut h);
    h.app.run("drawing.close", json!({})).unwrap();
    h.app.run("sketch.create", json!({"plane": "XY", "name": "Note"})).unwrap();
    h.app.run("sketch.finish", json!({})).unwrap();
    h.app.run("drawing.open", json!({"drawing": drawing})).unwrap();
    h.frames(3);
    let status = h.app.run("drawing.status", json!({})).unwrap();
    assert_eq!(status["held"], true, "{status}");
    assert_eq!(status["waiting"], 0, "the views look current: {status}");
    let dim = status["dimensions"].as_array().unwrap().iter()
        .find(|row| row["dimension"] == id).expect("our dimension in status");
    assert_eq!(dim["status"], "ok", "{dim}");
    assert_eq!(dim["out_of_date"], true, "dimension needs update even with current views: {dim}");
    let p = page(&mut h);
    assert!(!p.is_null(), "the native sheet exists");
    assert!(p["views"].as_array().unwrap().iter().all(|v| v["out_of_date"] == false), "{p}");
    assert!(p.get("dimensions").is_none(), "expected current PR #13211 gap changed: inspect and update acceptance plan");
    println!("T98_GAP=PR13211_UI_INSPECT_HAS_NO_DIMENSION_STATUS_WHEN_VIEWS_CURRENT; ENGINE_OUT_OF_DATE=TRUE; D07_PENDING");
    h.app.run("drawing.update", json!({})).unwrap();
    settle(&mut h);
    let clean = h.app.run("drawing.status", json!({})).unwrap();
    assert!(clean["dimensions"].as_array().unwrap().iter().all(|row| row["out_of_date"] != true), "{clean}");
}

/// A second deliberately *diagnostic* gap: an actual lost model reference
/// can be read via drawing.status and inspect, but the sheet page does not
/// yet expose a per-dimension Lost/why/Repair card (Nathan's D07/D26 slice).
#[test]
fn t98_diagnostic_lost_dimension_has_no_visible_repair_card_yet() {
    let (mut h, front) = plate_sheet();
    let id = t98_dim_id(&mut h, &front);
    let drawing = t98_drawing_id(&h);
    h.app.run("drawing.close", json!({})).unwrap();
    let feature = h.app.session.doc.features[0].id;
    h.app.run("timeline.delete", json!({"feature": feature.to_string()})).unwrap();
    h.app.run("drawing.open", json!({"drawing": drawing})).unwrap();
    settle(&mut h);
    let lost = h.app.run("drawing.dimension.inspect", json!({"dimension": id})).unwrap();
    assert_eq!(lost["status"], "lost", "{lost}");
    assert!(!lost["why"].as_str().unwrap_or("").is_empty(), "{lost}");
    assert!(lost.get("value").is_none(), "historical value may not be current: {lost}");
    let status = h.app.run("drawing.status", json!({})).unwrap();
    let row = status["dimensions"].as_array().unwrap().iter()
        .find(|row| row["dimension"] == id).expect("lost dimension in status");
    assert_eq!(row["status"], "lost", "{status}");
    let p = page(&mut h);
    assert!(!p.is_null(), "the sheet remains open after feature deletion");
    assert!(p["error"].is_null(), "{p}");
    assert!(p.get("dimensions").is_none(), "the D07 card may now exist; review this diagnostic");
    println!("T98_GAP=PR13211_SHEET_PAGE_HAS_NO_LOST_DIMENSION_REASON_OR_REPAIR_CARD; ENGINE_LOST_PRESENT; D07_PENDING");
}
