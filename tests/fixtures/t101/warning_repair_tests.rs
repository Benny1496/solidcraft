// T101: native headless egui actions, not only status text. All code CI-only.
struct T101Enabled;
impl T101Enabled {
    fn new() -> Self { super::t101_warning_poc::enable(); Self }
}
impl Drop for T101Enabled {
    fn drop(&mut self) { super::t101_warning_poc::disable(); }
}
fn t101_dim(h: &mut Harness, front: &str) -> Value {
    let inspected = h.app.run("drawing.view.inspect", json!({"view":front})).unwrap();
    let dims = inspected["dimensions"].as_array().expect("view dimensions");
    assert_eq!(dims.len(),1, "{inspected}");
    dims[0].clone()
}
fn t101_drawing(h: &Harness) -> String {
    solidcraft_engine::doc::ids::encode(h.app.session.open_drawing.unwrap())
}
fn t101_state(h: &mut Harness) -> Value {
    page(h)["t101_warning_poc"].clone()
}
fn t101_open_card(h: &mut Harness) {
    h.step(&json!({"click":{"handle":"t101:status_pill"}})).unwrap();
    h.frames(2);
    assert_eq!(t101_state(h)["expanded"],true);
    assert!(crate::scenario::rect_of("t101:status_details").is_some());
}
fn t101_make_fillet(h: &mut Harness, drawing: &str) {
    h.app.run("drawing.close",json!({})).unwrap();
    h.app.run("solid.fillet",json!({"body":"Plate","radius":5,"edges":[[50,0,20]]})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    settle(h);
}
fn t101_front_fillet_candidate(h: &mut Harness, dim: &Value) -> usize {
    let detail=h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap();
    assert_eq!(detail["status"],"lost","{detail}");
    let expected_a=solidcore_record::Vec3::from([0.0,0.0,15.0]);
    let expected_b=solidcore_record::Vec3::from([100.0,0.0,15.0]);
    let state=h.app.session.model.state();
    let mesh=state.body("Plate").expect("Plate after fillet").mesh();
    let edge=mesh.edges.iter().position(|line| {
        let a=line[0]; let b=line[line.len()-1];
        (a.dist(expected_a)<1e-6 && b.dist(expected_b)<1e-6)
            || (a.dist(expected_b)<1e-6 && b.dist(expected_a)<1e-6)
    }).expect("front replacement edge must exist on the filleted Plate");
    let desired=json!({"edge":{"body":"Plate","index":edge}});
    detail["candidates"].as_array().expect("repair candidates").iter()
        .position(|candidate|candidate==&desired)
        .unwrap_or_else(||panic!("the replacement front edge must be offered: {detail}"))
}

/// Real UI click to update, not a direct engine call in the test. A dimension
/// can be stale despite the two views being current.
#[test]
fn t101_native_update_button_clears_deferred_dimension_only_after_click() {
    let _guard=T101Enabled::new();
    let (mut h,front)=plate_sheet();
    let id=t101_dim(&mut h,&front);
    let drawing=t101_drawing(&h);
    h.app.run("drawing.set",json!({"updates":"deferred"})).unwrap();
    h.app.run("drawing.update",json!({})).unwrap();
    settle(&mut h);
    h.app.run("drawing.close",json!({})).unwrap();
    h.app.run("sketch.create",json!({"plane":"XY","name":"Unrelated note"})).unwrap();
    h.app.run("sketch.finish",json!({})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    h.frames(3);
    let status=h.app.run("drawing.status",json!({})).unwrap();
    assert_eq!(status["waiting"],0,"{status}");
    assert_eq!(status["held"],true,"{status}");
    assert_eq!(t101_state(&mut h)["findings"][0]["dimension"],id);
    t101_open_card(&mut h);
    let before=t101_state(&mut h);
    assert_eq!(before["label"],"1 need attention");
    assert_eq!(before["update_buttons"],1);
    assert_eq!(before["repair_buttons"],0);
    assert!(before["last_action"].is_null(),"no implicit update");
    assert!(crate::scenario::rect_of("t101:update_drawing").is_some());
    h.step(&json!({"click":{"handle":"t101:update_drawing"}})).unwrap();
    settle(&mut h);
    let after=t101_state(&mut h);
    assert_eq!(after["last_action"]["success"],true,"{after}");
    assert_eq!(after["last_action"]["command"],"drawing.update");
    assert_eq!(after["label"],"All current","{after}");
    println!("T101_PASS=NATIVE_CLICK_UPDATE_CLEARS_DIMENSION_ONLY_STALE; TEST_ONLY_UI");
}

/// An actual 3D fillet loses an edge. The UI lists inspect candidates; the
/// operator chooses a specific front-edge lead. Repair keeps the same dimension
/// ID, is one undo step and can be undone and redone.
#[test]
fn t101_native_lost_edge_repair_candidate_click_undo_redo() {
    let _guard=T101Enabled::new();
    let (mut h,front)=plate_sheet();
    let dim=t101_dim(&mut h,&front);
    let drawing=t101_drawing(&h);
    t101_make_fillet(&mut h,&drawing);
    let choice=t101_front_fillet_candidate(&mut h,&dim);
    let engine=h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap();
    assert_eq!(engine["status"],"lost","{engine}");
    assert!(engine.get("value").is_none(),"lost dimension may not have a live value");
    assert!(!engine["why"].as_str().unwrap_or("").is_empty());
    t101_open_card(&mut h);
    let p=t101_state(&mut h);
    assert_eq!(p["findings"][0]["kind"],"Lost","{p}");
    assert_eq!(p["update_buttons"],0,"lost needs a repair, not update");
    assert!(p["repair_buttons"]>choice,"candidate must have a real button: {p}");
    assert!(p["last_action"].is_null(),"never auto repair");
    let undone=h.app.session.undo.len();
    let target=format!("t101:repair:0:{choice}");
    assert!(crate::scenario::rect_of(&target).is_some(),"specific candidate button not painted: {target}");
    h.step(&json!({"click":{"handle":target}})).unwrap();
    h.frames(3);
    let result=t101_state(&mut h);
    assert_eq!(result["last_action"]["success"],true,"{result}");
    assert_eq!(result["last_action"]["command"],"drawing.dimension.repair");
    let fixed=h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap();
    assert_eq!(fixed["status"],"ok","{fixed}");
    assert_eq!(fixed["dimension"],dim);
    assert_eq!(fixed["value"],json!(100.0),"front fillet edge still 100 mm");
    assert_eq!(h.app.session.undo.len(),undone+1,"one undo step per explicit repair");
    h.app.run("edit.undo",json!({})).unwrap();
    h.frames(2);
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap()["status"],"lost");
    h.app.run("edit.redo",json!({})).unwrap();
    h.frames(2);
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap()["status"],"ok");
    assert_eq!(t101_state(&mut h)["label"],"All current");
    println!("T101_PASS=NATIVE_REPAIR_CANDIDATE_CLICK_UNDO_REDO_SAME_ID; TEST_ONLY_UI");
}

/// When deletion leaves no geometry, the status card must show Lost and why,
/// but must NOT invent an actionable repair candidate or suggest updating.
#[test]
fn t101_native_no_repair_candidates_does_not_offer_false_action() {
    let _guard=T101Enabled::new();
    let (mut h,front)=plate_sheet();
    let dim=t101_dim(&mut h,&front);
    let drawing=t101_drawing(&h);
    h.app.run("drawing.close",json!({})).unwrap();
    let block=h.app.session.doc.features[0].id;
    h.app.run("timeline.delete",json!({"feature":block.to_string()})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    settle(&mut h);
    let detail=h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap();
    assert_eq!(detail["status"],"lost");
    assert!(detail["candidates"].as_array().is_none_or(Vec::is_empty),"{detail}");
    t101_open_card(&mut h);
    let card=t101_state(&mut h);
    assert_eq!(card["repair_buttons"],0,"no geometry means no candidate: {card}");
    assert_eq!(card["update_buttons"],0);
    assert!(card["last_action"].is_null());
    assert_eq!(card["findings"][0]["dimension"],dim);
    println!("T101_PASS=LOST_NO_CANDIDATE_NO_FAKE_REPAIR_ACTION; TEST_ONLY_UI");
}

/// One repaired dimension cannot modify another dimension's record or status.
#[test]
fn t101_native_selected_repair_does_not_touch_other_dimension() {
    let _guard=T101Enabled::new();
    let (mut h,front)=plate_sheet();
    let dim=t101_dim(&mut h,&front);
    let drawing=t101_drawing(&h);
    let other=h.app.run("drawing.dimension.add",json!({
        "view":front,"kind":{"length":{"edge":[50,0,0]}}
    })).unwrap()["dimension"].clone();
    let other_before=h.app.run("drawing.dimension.inspect",json!({"dimension":other})).unwrap();
    assert_eq!(other_before["status"],"ok");
    t101_make_fillet(&mut h,&drawing);
    let choice=t101_front_fillet_candidate(&mut h,&dim);
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":other})).unwrap()["status"],"ok");
    t101_open_card(&mut h);
    let target=format!("t101:repair:0:{choice}");
    h.step(&json!({"click":{"handle":target}})).unwrap();
    h.frames(3);
    let other_after=h.app.run("drawing.dimension.inspect",json!({"dimension":other})).unwrap();
    assert_eq!(other_after,other_before,"other dimension must not change");
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap()["status"],"ok");
    h.app.run("edit.undo",json!({})).unwrap();
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap()["status"],"lost");
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":other})).unwrap(),other_before);
    println!("T101_PASS=REPAIR_TARGET_ISOLATED_UNAFFECTED_DIMENSION; TEST_ONLY_UI");
}

/// Healthy dimension cannot be repaired from the card. No buttons or actions
/// appear merely because there is a normal dimension on the sheet.
#[test]
fn t101_native_all_current_has_no_update_or_repair_buttons() {
    let _guard=T101Enabled::new();
    let (mut h,_front)=plate_sheet();
    t101_open_card(&mut h);
    let p=t101_state(&mut h);
    assert_eq!(p["label"],"All current");
    assert_eq!(p["repair_buttons"],0);
    assert_eq!(p["update_buttons"],0);
    assert!(p["last_action"].is_null());
    assert!(crate::scenario::rect_of("t101:status_pill").is_some());
    println!("T101_PASS=HEALTHY_STATUS_CARD_NO_MUTATING_BUTTONS; TEST_ONLY_UI");
}
