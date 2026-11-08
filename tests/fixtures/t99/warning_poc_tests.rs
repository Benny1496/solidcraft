// Test #99: a temporary, explicitly marked UI prototype on Loohi's
// drawing page. It is NOT product UI and never lands in official code.

struct T99Enabled;
impl T99Enabled {
    fn new() -> Self {
        super::t99_warning_poc::enable();
        Self
    }
}
impl Drop for T99Enabled {
    fn drop(&mut self) { super::t99_warning_poc::disable(); }
}

fn t99_id(h: &mut Harness, front: &str) -> Value {
    let reply = h.app.run("drawing.view.inspect", json!({"view":front})).unwrap();
    let dims = reply["dimensions"].as_array().expect("dimensions list");
    assert_eq!(dims.len(),1,"fixture expected one dimension: {reply}");
    dims[0].clone()
}

fn t99_open_drawing(h: &Harness) -> String {
    solidcraft_engine::doc::ids::encode(h.app.session.open_drawing.unwrap())
}

fn t99_poc(h: &mut Harness) -> Value {
    page(h)["t99_warning_poc"].clone()
}

#[test]
fn t99_native_current_sheet_shows_all_current_status_pill() {
    let _guard=T99Enabled::new();
    let (mut h,_)=plate_sheet();
    h.frames(2);
    let p=t99_poc(&mut h);
    assert_eq!(p["label"],"All current","{p}");
    assert_eq!(p["count"],0,"{p}");
    assert_eq!(p["poc"],true,"{p}");
    assert!(crate::scenario::rect_of("t99:status_pill").is_some(),"status pill is really drawn");
    h.step(&json!({"click":{"handle":"t99:status_pill"}})).unwrap();
    h.frames(2);
    let p=t99_poc(&mut h);
    assert_eq!(p["expanded"],true,"{p}");
    assert!(crate::scenario::rect_of("t99:status_details").is_some(),"the open card is painted");
    println!("T99_PASS=REAL_EGUI_CI_ONLY_POC_ALL_CURRENT_AND_CLICK; NOT_PRODUCT_UI");
}

#[test]
fn t99_native_deferred_dimension_only_shows_needs_update_without_stale_views() {
    let _guard=T99Enabled::new();
    let (mut h,front)=plate_sheet();
    let id=t99_id(&mut h,&front);
    let drawing=t99_open_drawing(&h);
    h.app.run("drawing.set",json!({"updates":"deferred"})).unwrap();
    h.app.run("drawing.update",json!({})).unwrap();
    settle(&mut h);
    h.app.run("drawing.close",json!({})).unwrap();
    h.app.run("sketch.create",json!({"plane":"XY","name":"Note"})).unwrap();
    h.app.run("sketch.finish",json!({})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    h.frames(3);
    let engine=h.app.run("drawing.status",json!({})).unwrap();
    assert_eq!(engine["held"],true,"{engine}");
    assert_eq!(engine["waiting"],0,"{engine}");
    let drawn_page=page(&mut h);\n    let states=&drawn_page["views"];
    assert!(states.as_array().unwrap().iter().all(|v|v["out_of_date"]==false),"{states}");
    let p=t99_poc(&mut h);
    assert_eq!(p["count"],1,"{p}");
    assert_eq!(p["label"],"1 need attention","{p}");
    assert_eq!(p["findings"][0]["dimension"],id,"{p}");
    assert_eq!(p["findings"][0]["kind"],"Needs update","{p}");
    h.step(&json!({"click":{"handle":"t99:status_pill"}})).unwrap();
    h.frames(2);
    assert_eq!(t99_poc(&mut h)["expanded"],true);
    h.app.run("drawing.update",json!({})).unwrap();
    settle(&mut h);
    assert_eq!(t99_poc(&mut h)["label"],"All current","warning clears after explicit update");
    println!("T99_PASS=CI_ONLY_POC_DEFERRED_DIMENSION_WITH_CURRENT_VIEWS; NOT_PRODUCT_UI");
}

#[test]
fn t99_native_lost_dimension_reveals_id_reason_and_undo_recovers() {
    let _guard=T99Enabled::new();
    let (mut h,front)=plate_sheet();
    let id=t99_id(&mut h,&front);
    let drawing=t99_open_drawing(&h);
    h.app.run("drawing.close",json!({})).unwrap();
    let block=h.app.session.doc.features[0].id;
    h.app.run("timeline.delete",json!({"feature":block.to_string()})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    settle(&mut h);
    let engine=h.app.run("drawing.dimension.inspect",json!({"dimension":id})).unwrap();
    assert_eq!(engine["status"],"lost","{engine}");
    assert!(engine.get("value").is_none(),"historical reading cannot be current: {engine}");
    let p=t99_poc(&mut h);
    assert_eq!(p["count"],1,"{p}");
    assert_eq!(p["findings"][0]["dimension"],id,"{p}");
    assert_eq!(p["findings"][0]["kind"],"Lost","{p}");
    assert_eq!(p["findings"][0]["symbol"],"✕","{p}");
    assert!(!p["findings"][0]["why"].as_str().unwrap_or("").is_empty(),"{p}");
    assert_eq!(p["real_repair_button"],false,"do not pretend a repair action is implemented");
    h.step(&json!({"click":{"handle":"t99:status_pill"}})).unwrap();
    h.frames(2);
    assert!(crate::scenario::rect_of("t99:status_details").is_some());
    h.app.run("drawing.close",json!({})).unwrap();
    h.app.run("edit.undo",json!({})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    settle(&mut h);
    assert_eq!(t99_poc(&mut h)["label"],"All current","undo should remove lost warning");
    println!("T99_PASS=CI_ONLY_POC_LOST_ID_REASON_CLICK_UNDO; REAL_REPAIR_CONTROL=NO");
}
