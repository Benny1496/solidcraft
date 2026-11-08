// T104 native egui acceptance extension, appended ONLY to temporary CI runner checkout.
// Reuses the tested T103 status-card prototype. No production Drawing UI changes.
fn t104_other_dimension(h: &mut Harness, front: &str) -> Value {
    h.app.run("drawing.dimension.add", json!({
        "view":front,"kind":{"length":{"edge":[50,0,0]}}
    })).unwrap()["dimension"].clone()
}
fn t104_unrelated_deferred_change(h: &mut Harness, drawing: &str) {
    h.app.run("drawing.close",json!({})).unwrap();
    h.app.run("sketch.create",json!({"plane":"XY","name":"T104 extra sketch"})).unwrap();
    h.app.run("sketch.finish",json!({})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    h.frames(3);
}
#[test]
fn t104_two_stale_dimensions_can_be_updated_with_one_explicit_click() {
    let _guard=T103Enabled::new();
    let (mut h,front)=plate_sheet();
    let first=t103_dim(&mut h,&front);
    let second=t104_other_dimension(&mut h,&front);
    let drawing=t103_drawing(&h);
    h.app.run("drawing.set",json!({"updates":"deferred"})).unwrap();
    h.app.run("drawing.update",json!({})).unwrap();
    settle(&mut h);
    t104_unrelated_deferred_change(&mut h,&drawing);
    let status=h.app.run("drawing.status",json!({})).unwrap();
    assert_eq!(status["waiting"],0,"{status}");
    assert_eq!(status["held"],true,"{status}");
    t103_open_card(&mut h);
    let before=t103_state(&mut h);
    assert_eq!(before["count"],2,"both dimension IDs must be shown: {before}");
    let rows=before["findings"].as_array().unwrap();
    assert!(rows.iter().any(|r|r["dimension"]==first),"{before}");
    assert!(rows.iter().any(|r|r["dimension"]==second),"{before}");
    assert_eq!(before["update_buttons"],2,"{before}");
    assert_eq!(before["repair_buttons"],0,"{before}");
    assert!(before["last_action"].is_null(),"never update automatically");
    h.step(&json!({"click":{"handle":"t103:update_drawing"}})).unwrap();
    settle(&mut h);
    let after=t103_state(&mut h);
    assert_eq!(after["last_action"]["success"],true,"{after}");
    assert_eq!(after["last_action"]["command"],"drawing.update");
    assert_eq!(after["label"],"All current","{after}");
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":first})).unwrap()["status"],"ok");
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":second})).unwrap()["status"],"ok");
    println!("T104_PASS=TWO_STALE_DIMENSIONS_SHOWN_AND_ONE_CLICK_UPDATES_BOTH");
}
#[test]
fn t104_lost_and_stale_never_silently_repair_a_lost_reference() {
    let _guard=T103Enabled::new();
    let (mut h,front)=plate_sheet();
    let lost=t103_dim(&mut h,&front);
    let other=t104_other_dimension(&mut h,&front);
    let drawing=t103_drawing(&h);
    t103_make_fillet(&mut h,&drawing);
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":lost})).unwrap()["status"],"lost");
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":other})).unwrap()["status"],"ok");
    h.app.run("drawing.set",json!({"updates":"deferred"})).unwrap();
    t104_unrelated_deferred_change(&mut h,&drawing);
    t103_open_card(&mut h);
    let card=t103_state(&mut h);
    let rows=card["findings"].as_array().unwrap();
    assert!(rows.iter().any(|r|r["dimension"]==lost && r["kind"]=="Lost"),"{card}");
    assert!(rows.iter().any(|r|r["dimension"]==other && r["kind"]=="Needs update"),"{card}");
    assert!(card["update_buttons"].as_u64().unwrap_or(0)>0,"{card}");
    assert!(card["last_action"].is_null(),"must not guess");
    h.step(&json!({"click":{"handle":"t103:update_drawing"}})).unwrap();
    settle(&mut h);
    let after=t103_state(&mut h);
    assert_eq!(after["last_action"]["command"],"drawing.update","{after}");
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":lost})).unwrap()["status"],"lost","Update must not invent a repair");
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":other})).unwrap()["status"],"ok");
    assert!(after["findings"].as_array().unwrap().iter().any(|r|r["dimension"]==lost && r["kind"]=="Lost"),"{after}");
    println!("T104_PASS=MIXED_STALE_UPDATE_NEVER_REPAIRS_LOST");
}
#[test]
fn t104_clicked_repair_is_kept_when_drawing_closes_and_reopens_in_session() {
    let _guard=T103Enabled::new();
    let (mut h,front)=plate_sheet();
    let dim=t103_dim(&mut h,&front);
    let drawing=t103_drawing(&h);
    t103_make_fillet(&mut h,&drawing);
    let choice=t103_front_fillet_candidate(&mut h,&dim);
    t103_open_card(&mut h);
    h.step(&json!({"click":{"handle":format!("t103:repair:0:{choice}")}})).unwrap();
    h.frames(3);
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap()["status"],"ok");
    h.app.run("drawing.close",json!({})).unwrap();
    h.app.run("drawing.open",json!({"drawing":drawing})).unwrap();
    settle(&mut h);
    let fixed=h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap();
    assert_eq!(fixed["status"],"ok","{fixed}");
    assert_eq!(fixed["dimension"],dim,"original ID must survive reopen");
    assert_eq!(fixed["value"],json!(100.0),"front replacement edge");
    let card=t103_state(&mut h);
    assert_eq!(card["label"],"All current","{card}");
    assert_eq!(card["repair_buttons"],0,"{card}");
    println!("T104_PASS=REPAIRED_DIMENSION_SAME_ID_AFTER_CLOSE_REOPEN");
}
#[test]
fn t104_candidate_click_passes_the_exact_inspect_candidate() {
    let _guard=T103Enabled::new();
    let (mut h,front)=plate_sheet();
    let dim=t103_dim(&mut h,&front);
    let drawing=t103_drawing(&h);
    t103_make_fillet(&mut h,&drawing);
    let choice=t103_front_fillet_candidate(&mut h,&dim);
    let detail=h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap();
    let candidate=detail["candidates"][choice].clone();
    t103_open_card(&mut h);
    assert!(t103_state(&mut h)["last_action"].is_null(),"no automatic candidate choice");
    h.step(&json!({"click":{"handle":format!("t103:repair:0:{choice}")}})).unwrap();
    h.frames(3);
    let result=t103_state(&mut h)["last_action"].clone();
    assert_eq!(result["success"],true,"{result}");
    assert_eq!(result["command"],"drawing.dimension.repair");
    assert_eq!(result["params"]["dimension"],dim);
    assert_eq!(result["params"]["slot"],"edge");
    assert_eq!(result["params"]["pick"],candidate,"candidate unchanged from engine");
    assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap()["status"],"ok");
    println!("T104_PASS=REPAIR_USES_EXACT_SELECTED_ENGINE_CANDIDATE");
}
#[test]
fn t104_three_consecutive_repair_undo_redo_cycles_have_no_ghost_warning() {
    let _guard=T103Enabled::new();
    let (mut h,front)=plate_sheet();
    let dim=t103_dim(&mut h,&front);
    let drawing=t103_drawing(&h);
    t103_make_fillet(&mut h,&drawing);
    let choice=t103_front_fillet_candidate(&mut h,&dim);
    t103_open_card(&mut h);
    h.step(&json!({"click":{"handle":format!("t103:repair:0:{choice}")}})).unwrap();
    h.frames(3);
    for _ in 0..3 {
        h.app.run("edit.undo",json!({})).unwrap();
        h.frames(2);
        assert_eq!(h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap()["status"],"lost");
        let warning=t103_state(&mut h);
        assert!(warning["findings"].as_array().unwrap().iter().any(|r|r["dimension"]==dim && r["kind"]=="Lost"),"{warning}");
        h.app.run("edit.redo",json!({})).unwrap();
        h.frames(2);
        let fixed=h.app.run("drawing.dimension.inspect",json!({"dimension":dim})).unwrap();
        assert_eq!(fixed["status"],"ok","{fixed}");
        assert_eq!(fixed["dimension"],dim);
        let card=t103_state(&mut h);
        assert_eq!(card["label"],"All current","{card}");
    }
    println!("T104_PASS=REPEATED_REPAIR_UNDO_REDO_NO_GHOST_WARNINGS");
}
#[test]
fn t104_status_matrix_preserves_id_reason_and_never_invents_current_value() {
    for (row,kind,why) in [
        (json!({"dimension":"A","status":"lost","out_of_date":true,"why":"source edge gone","last_good":120.0}),"Lost","source edge gone"),
        (json!({"dimension":"B","status":"ambiguous","out_of_date":true,"why":"several edges"}),"Ambiguous","several edges"),
        (json!({"dimension":"C","status":"ok","out_of_date":true,"why":"deferred"}),"Needs update","deferred"),
        (json!({"dimension":"D","status":"ok","out_of_date":true}),"Needs update","Drawing is not up to date")
    ] {
        let warning=super::t103_warning_poc::message(&row).expect("warning expected");
        assert_eq!(warning["kind"],kind,"{warning}");
        assert_eq!(warning["why"],why,"{warning}");
        assert_eq!(warning["dimension"],row["dimension"]);
        assert!(warning.get("value").is_none(),"do not show last_good as current");
    }
    for healthy in [
        json!({"dimension":"E","status":"ok","out_of_date":false,"last_good":120.0}),
        json!({"dimension":"F","status":"overridden","out_of_date":false})
    ] {
        assert!(super::t103_warning_poc::message(&healthy).is_none(),"{healthy}");
    }
    println!("T104_PASS=WARNING_PRIORITY_ID_REASON_NO_FALSE_LIVE_VALUE");
}
