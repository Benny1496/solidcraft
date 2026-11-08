// Test #99: a temporary, explicitly marked UI prototype on Loohi's
// drawing page. It is NOT product UI and never lands in official code.

struct T100Enabled;
impl T100Enabled {
    fn new() -> Self {
        super::t100_warning_poc::enable();
        Self
    }
}
impl Drop for T100Enabled {
    fn drop(&mut self) { super::t100_warning_poc::disable(); }
}

fn t100_id(h: &mut Harness, front: &str) -> Value {
    let reply = h.app.run("drawing.view.inspect", json!({"view":front})).unwrap();
    let dims = reply["dimensions"].as_array().expect("dimensions list");
    assert_eq!(dims.len(),1,"fixture expected one dimension: {reply}");
    dims[0].clone()
}

fn t100_open_drawing(h: &Harness) -> String {
    solidcraft_engine::doc::ids::encode(h.app.session.open_drawing.unwrap())
}

fn t100_poc(h: &mut Harness) -> Value {
    page(h)["t100_warning_poc"].clone()
}

#[test]
fn t100_native_current_sheet_shows_all_current_status_pill() {
    let _guard=T100Enabled::new();
    let (mut h,_)=plate_sheet();
    h.frames(2);
    let p=t100_poc(&mut h);
    assert_eq!(p["label"],"All current","{p}");
    assert_eq!(p["count"],0,"{p}");
    assert_eq!(p["poc"],true,"{p}");
    assert!(crate::scenario::rect_of("t100:status_pill").is_some(),"status pill is really drawn");
    h.step(&json!({"click":{"handle":"t100:status_pill"}})).unwrap();
    h.frames(2);
    let p=t100_poc(&mut h);
    assert_eq!(p["expanded"],true,"{p}");
    assert!(crate::scenario::rect_of("t100:status_details").is_some(),"the open card is painted");
    h.step(&json!({"click":{"handle":"t100:status_pill"}})).unwrap();
    h.frames(2);
    assert_eq!(t100_poc(&mut h)["expanded"],false,"second click closes the status card");
    println!("T100_PASS=CARD_TOGGLE_OPEN_CLOSE");
    println!("T100_PASS=REAL_EGUI_CI_ONLY_POC_ALL_CURRENT_AND_CLICK; NOT_PRODUCT_UI");
}

#[test]
fn t100_native_deferred_dimension_only_shows_needs_update_without_stale_views() {
    let _guard=T100Enabled::new();
    let (mut h,front)=plate_sheet();
    let id=t100_id(&mut h,&front);
    let drawing=t100_open_drawing(&h);
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
    let drawn_page = page(&mut h);
    let states = &drawn_page["views"];
    assert!(states.as_array().unwrap().iter().all(|v|v["out_of_date"]==false),"{states}");
    let p=t100_poc(&mut h);
    assert_eq!(p["count"],1,"{p}");
    assert_eq!(p["label"],"1 need attention","{p}");
    assert_eq!(p["findings"][0]["dimension"],id,"{p}");
    assert_eq!(p["findings"][0]["kind"],"Needs update","{p}");
    h.step(&json!({"click":{"handle":"t100:status_pill"}})).unwrap();
    h.frames(2);
    assert_eq!(t100_poc(&mut h)["expanded"],true);
    h.app.run("drawing.update",json!({})).unwrap();
    settle(&mut h);
    assert_eq!(t100_poc(&mut h)["label"],"All current","warning clears after explicit update");
    println!("T100_PASS=CI_ONLY_POC_DEFERRED_DIMENSION_WITH_CURRENT_VIEWS; NOT_PRODUCT_UI");
}

#[test]
fn t100_native_lost_dimension_reveals_id_reason_and_undo_recovers() {
    let _guard=T100Enabled::new();
    let (mut h,front)=plate_sheet();
    let id=t100_id(&mut h,&front);
    let drawing=t100_open_drawing(&h);
    h.app.run("drawing.close",json!({})).unwrap();
    let block=h.app.session.doc.features[0].id;
    h.app.run("timeline.delete",json!({"feature":block.to_string()})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    settle(&mut h);
    let engine=h.app.run("drawing.dimension.inspect",json!({"dimension":id})).unwrap();
    assert_eq!(engine["status"],"lost","{engine}");
    assert!(engine.get("value").is_none(),"historical reading cannot be current: {engine}");
    let p=t100_poc(&mut h);
    assert_eq!(p["count"],1,"{p}");
    assert_eq!(p["findings"][0]["dimension"],id,"{p}");
    assert_eq!(p["findings"][0]["kind"],"Lost","{p}");
    assert_eq!(p["findings"][0]["symbol"],"✕","{p}");
    assert!(!p["findings"][0]["why"].as_str().unwrap_or("").is_empty(),"{p}");
    assert_eq!(p["real_repair_button"],false,"do not pretend a repair action is implemented");
    h.step(&json!({"click":{"handle":"t100:status_pill"}})).unwrap();
    h.frames(2);
    assert!(crate::scenario::rect_of("t100:status_details").is_some());
    h.app.run("drawing.close",json!({})).unwrap();
    h.app.run("edit.undo",json!({})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    settle(&mut h);
    assert_eq!(t100_poc(&mut h)["label"],"All current","undo should remove lost warning");
    println!("T100_PASS=CI_ONLY_POC_LOST_ID_REASON_CLICK_UNDO; REAL_REPAIR_CONTROL=NO");
}


/// D26 mapping: critical states take precedence over stale flags; the test-only
/// warning summary never mistakes an ordinary healthy dimension for an issue.
#[test]
fn t100_d26_priority_maps_lost_ambiguous_stale_and_healthy() {
    let rows = [
        (json!({"dimension":"dim-l","status":"lost","out_of_date":true,"why":"edge removed"}), "Lost", "✕"),
        (json!({"dimension":"dim-a","status":"ambiguous","out_of_date":true,"why":"multiple matches"}), "Ambiguous", "⚠"),
        (json!({"dimension":"dim-s","status":"ok","out_of_date":true}), "Needs update", "⟳"),
    ];
    for (row, name, symbol) in rows {
        let m=super::t100_warning_poc::message(&row).expect("should appear in warning card");
        assert_eq!(m["kind"],name,"{m}");
        assert_eq!(m["symbol"],symbol,"{m}");
        assert_eq!(m["dimension"],row["dimension"]);
        assert!(!m["why"].as_str().unwrap_or("").is_empty());
    }
    for row in [json!({"dimension":"dim-v","status":"ok","out_of_date":false}),
                json!({"dimension":"dim-o","status":"overridden","out_of_date":false})] {
        assert!(super::t100_warning_poc::message(&row).is_none(),"no spurious warning: {row}");
    }
    println!("T100_PASS=D26_STATUS_PRIORITY_SYMBOLS_AND_NO_FALSE_ALARM; NOT_PRODUCT_UI");
}

/// Missing explanation text must not create an empty warning or copy a previous
/// measurement as an up-to-date value.
#[test]
fn t100_d07_fallback_reasons_remain_explicit_without_old_measurements() {
    let lost=json!({"dimension":"dim-x","status":"lost","why":"","last_good":80.0});
    let p=super::t100_warning_poc::message(&lost).expect("lost must be surfaced");
    assert_eq!(p["kind"],"Lost");
    assert_eq!(p["why"],"Reference needs attention");
    assert!(p.get("value").is_none(),"last_good is not the live value");
    assert!(p.get("last_good").is_none(),"do not suggest historical value is current");
    let outdated=json!({"dimension":"dim-y","status":"ok","out_of_date":true,"last_good":100.0});
    let stale=super::t100_warning_poc::message(&outdated).expect("stale must be surfaced");
    assert_eq!(stale["why"],"Drawing is not up to date");
    assert!(stale.get("value").is_none());
    println!("T100_PASS=D07_EXPLANATION_FALLBACKS_NEVER_EMIT_OLD_VALUE_AS_LIVE; NOT_PRODUCT_UI");
}

#[test]
fn t100_native_disabled_prototype_does_not_modify_sheet_inspection() {
    super::t100_warning_poc::disable();
    let (mut h, _) = plate_sheet();
    h.frames(2);
    assert!(page(&mut h).get("t100_warning_poc").is_some(), "test-only inspection key must exist");
    assert!(t100_poc(&mut h).is_null(), "disabled proof-of-concept must not be active");
    assert!(crate::scenario::rect_of("sheet:paper").is_some());
    assert!(crate::scenario::rect_of("t100:status_pill").is_none());
    println!("T100_PASS=DISABLED_OVERLAY_DOES_NOT_AFFECT_REGULAR_DRAWING");
}
