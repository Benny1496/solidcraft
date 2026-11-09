// T105: disk round-trip and multi-sheet dimension regression.
// Injected ONLY into the temporary GitHub Actions checkout.
#[test]
fn t105_repaired_edge_and_healthy_dimension_survive_disk_roundtrip_in_both_formats() {
    for filename in [format!("t105-repaired.{}", solidcraft_io::DESIGN_EXT), "t105-repaired.json".to_owned()] {
        let (mut s, view) = front_view();
        let drawing = s.drawing("test").unwrap().id;
        let edge = add(&mut s, view, json!({"length":{"edge":[40,0,30]}}))["dimension"].clone();
        let stable = add(&mut s, view, json!({"length":{"edge":[40,0,0]}}))["dimension"].clone();
        run(&mut s, "drawing.close", json!({}));
        lose_the_edge(&mut s);
        run(&mut s, "drawing.open", json!({"drawing":ids::encode(drawing)}));
        let lost = run(&mut s, INSPECT, json!({"dimension":edge}));
        assert_eq!(lost["status"],"lost","{filename}: {lost}");
        let candidate = json!({"edge":{"body":"Box","index":edge_index(&s,[0.0,0.0,25.0],[80.0,0.0,25.0])}});
        assert!(lost["candidates"].as_array().is_some_and(|cs|cs.contains(&candidate)),"{lost}");
        let fixed = run(&mut s, REPAIR, json!({"dimension":edge,"slot":"edge","pick":candidate}));
        reads(&fixed,80.0,"80.00");
        let stable_before = run(&mut s, INSPECT, json!({"dimension":stable}));
        reads(&stable_before,80.0,"80.00");
        let mut fresh = reopened(&mut s,drawing,&filename);
        let from_disk = run(&mut fresh, INSPECT, json!({"dimension":edge}));
        reads(&from_disk,80.0,"80.00");
        assert_eq!(from_disk["dimension"],edge,"repaired id must survive disk load");
        // The saved file materializes last_good on reopen, while the immediate
        // repair reply may not include it yet. Compare every authored field,
        // and assert the cached measurement independently rather than treating
        // an additional persisted last_good as a lost repair.
        assert_eq!(from_disk["record"]["last_good"], json!(80.0),
                   "{filename}: repaired dimension must retain its last valid reading");
        let mut disk_record = from_disk["record"].clone();
        let mut repair_record = fixed["record"].clone();
        disk_record.as_object_mut().unwrap().remove("last_good");
        repair_record.as_object_mut().unwrap().remove("last_good");
        assert_eq!(disk_record, repair_record, "{filename}: repaired authored fields must survive disk load");
        let stable_after=run(&mut fresh, INSPECT, json!({"dimension":stable}));
        reads(&stable_after,80.0,"80.00");
        assert_eq!(stable_after["dimension"],stable);
        assert_eq!(stable_after["record"]["last_good"], json!(80.0),
                   "{filename}: healthy dimension must retain its last valid reading");
        let mut stable_disk_record = stable_after["record"].clone();
        let mut stable_original_record = stable_before["record"].clone();
        stable_disk_record.as_object_mut().unwrap().remove("last_good");
        stable_original_record.as_object_mut().unwrap().remove("last_good");
        assert_eq!(stable_disk_record, stable_original_record,
                   "{filename}: other dimension's authored fields must survive disk load unchanged");
        assert!(!fresh.is_dirty(),"reads after disk load are read only");
    }
    println!("T105_PASS=REPAIR_PERSISTS_ON_DISK_BOTH_NATIVE_AND_JSON_OTHER_DIMENSION_UNTOUCHED");
}

#[test]
fn t105_lost_and_healthy_dimensions_remain_distinct_after_disk_load_and_update() {
    for filename in [format!("t105-lost.{}", solidcraft_io::DESIGN_EXT), "t105-lost.json".to_owned()] {
        let (mut s, view) = front_view();
        let drawing = s.drawing("test").unwrap().id;
        let lost_id=add(&mut s,view,json!({"length":{"edge":[40,0,30]}}))["dimension"].clone();
        let healthy_id=add(&mut s,view,json!({"length":{"edge":[40,0,0]}}))["dimension"].clone();
        run(&mut s,"drawing.close",json!({}));
        lose_the_edge(&mut s);
        let mut fresh=reopened(&mut s,drawing,&filename);
        let lost=run(&mut fresh,INSPECT,json!({"dimension":lost_id}));
        assert_eq!(lost["status"],"lost","{filename}: {lost}");
        assert!(lost.get("value").is_none(),"cannot pretend last_good is a current value");
        assert_eq!(lost["last_good"],json!(80.0),"historical read should survive");
        reads(&run(&mut fresh,INSPECT,json!({"dimension":healthy_id})),80.0,"80.00");
        run(&mut fresh,"drawing.update",json!({}));
        let after=run(&mut fresh,INSPECT,json!({"dimension":lost_id}));
        assert_eq!(after["status"],"lost","a general update may not guess a new reference");
        assert_eq!(after["dimension"],lost_id);
        reads(&run(&mut fresh,INSPECT,json!({"dimension":healthy_id})),80.0,"80.00");
        let candidates = after["candidates"].as_array().expect("lost repair leads");
        assert!(!candidates.is_empty(),"lost edge should still offer replacement leads");
    }
    println!("T105_PASS=LOST_AND_HEALTHY_DISK_ROUNDTRIP_NO_SILENT_REPAIR");
}

#[test]
fn t105_copy_dimension_to_second_sheet_survives_real_disk_load_with_distinct_ids() {
    for filename in [format!("t105-two-sheets.{}",solidcraft_io::DESIGN_EXT),"t105-two-sheets.json".to_owned()] {
        let (mut s,view)=front_view();
        let drawing=s.drawing("test").unwrap().id;
        let original=add(&mut s,view,json!({"length":{"edge":[40,0,30]}}))["dimension"].clone();
        run(&mut s,"drawing.sheet.add",json!({"name":"Sheet 2"}));
        run(&mut s,"drawing.view.copy_to_sheet",json!({
            "view":ids::encode(view),"sheet":"Sheet 2","position_mm":[140,140]
        }));
        let (new_id, sheet_count)={
            let d=s.drawing("test").unwrap();
            let source_id=ids::decode(original.as_str().unwrap()).unwrap();
            let others:Vec<_>=d.dimensions.iter().filter(|dim|dim.id!=source_id).collect();
            assert_eq!(others.len(),1,"copied view must have precisely one copied dimension");
            (others[0].id,d.sheets.len())
        };
        assert_eq!(sheet_count,2);
        let copy=ids::encode(new_id);
        assert_ne!(original,copy,"copied dimension has its own ID");
        reads(&run(&mut s,INSPECT,json!({"dimension":original})),80.0,"80.00");
        reads(&run(&mut s,INSPECT,json!({"dimension":copy})),80.0,"80.00");
        run(&mut s,"drawing.close",json!({}));
        lose_the_edge(&mut s);
        let mut fresh=reopened(&mut s,drawing,&filename);
        assert_eq!(fresh.drawing("test").unwrap().sheets.len(),2);
        let a=run(&mut fresh,INSPECT,json!({"dimension":original}));
        let b=run(&mut fresh,INSPECT,json!({"dimension":copy}));
        assert_eq!(a["status"],"lost","original missing reference {a}");
        assert_eq!(b["status"],"lost","copy must not silently bind an unrelated edge: {b}");
        assert_eq!(a["dimension"],original);
        assert_eq!(b["dimension"],copy);
        assert_ne!(a["view"],b["view"],"dimensions must belong to separate sheet views");
    }
    println!("T105_PASS=TWO_SHEETS_COPIED_DIMENSION_DISTINCT_IDS_LOST_PERSISTED_ON_DISK");
}
