/// T93/1: Direct model commands are intentionally blocked while a drawing is
/// open. But a supported model checkpoint/restore while it is open must redraw
/// both views and the saved length dimension without an explicit drawing.update.
#[test]
fn t93_open_sheet_guards_model_edits_and_restores_live_dimension() {
    let (mut s, sheet, [front, _]) = front_and_top();
    let dimension = front_edge(&mut s, front);
    let first = settled(&mut s, sheet);
    let checkpoint = run(&mut s, "edit.checkpoint", json!({}))["checkpoint"].clone();
    let original_doc = s.doc.to_json();
    let err = s
        .execute("solid.box", &json!({"length": 12, "width": 10, "height": 10}))
        .expect_err("model command must not run while drawing is open")
        .to_string();
    assert!(
        err.contains("a drawing is open: close it to change the model"),
        "{err}"
    );
    assert_eq!(s.doc.to_json(), original_doc, "rejected edit must not alter design");
    let drawing = ids::encode(s.open_drawing.unwrap());
    run(&mut s, "drawing.close", json!({}));
    let block = s.doc.features[0].id;
    run(&mut s, "timeline.edit", json!({"feature": block.to_string(), "set": {"length": 140}}));
    run(&mut s, "drawing.open", json!({"drawing": drawing}));
    let edited = settled(&mut s, sheet);
    assert_ne!(bytes_of(group_of(&first.scene, dimension)), bytes_of(group_of(&edited.scene, dimension)));
    // A restored model checkpoint is supported even with a drawing already open,
    // and must cause automatic re-render on the same open sheet.
    run(&mut s, "edit.restore_checkpoint", json!({"checkpoint": checkpoint}));
    assert!(s.open_drawing.is_some(), "restoring the checkpoint must leave the drawing open");
    let restored = settled(&mut s, sheet);
    assert!(restored.stale.is_empty() && restored.stale_dimensions.is_empty());
    assert_eq!(bytes(&restored.scene), bytes(&first.scene), "open drawing must redraw without manual update");
    assert_eq!(bytes(&restored.scene), bytes(&built(&s, sheet).scene));
    println!("T93_CASE_1=PASS; DIRECT_MODEL_EDIT_WITH_DRAWING_OPEN=INTENTIONALLY_BLOCKED; OPEN_CHECKPOINT_RESTORE_AUTO_REDRAW=PASS");
}
