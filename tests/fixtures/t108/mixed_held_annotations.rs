
/// Runner-only T108 regression: the merged drawing.check, scene and export must
/// agree when the same exploded view holds back a note AND a dimension.
#[test]
fn t108_mixed_held_note_and_dimension_keep_kind_across_scene_export_and_reassembly() {
    let mut dimension = Value::Null;
    let mut note = Value::Null;
    let (mut s, view, _) = exploded(|s, view| {
        dimension = length(s, view, [50.0, 0.0, 0.0]);
        note = note_on(s, view, [50.0, 0.0, 0.0]);
    });
    let sheet = s.drawing("test").unwrap().sheets[0].id;
    let why = "its view draws an exploded view: not supported yet, so it isn't drawn";
    let checked_held = checked(&mut s, json!({}));
    let mut actual = whys(&checked_held, "out_of_date");
    actual.sort();
    let mut expected = vec![
        ("dimension".to_string(), dimension.as_str().unwrap().to_string(), why.to_string()),
        ("note".to_string(), note.as_str().unwrap().to_string(), why.to_string()),
    ];
    expected.sort();
    assert_eq!(actual, expected, "both records must retain their own kind: {checked_held}");

    let held = [dimension.clone(), note.clone()];
    for command in ["drawing.scene", "drawing.export"] {
        let reply = run(&mut s, command, json!({"sheet": ids::encode(sheet)}));
        let listed = reply["out_of_date"].as_array().expect("scene/export must list held IDs");
        for record in &held {
            assert_eq!(listed.iter().filter(|item| *item == record).count(), 1,
                "{command} must list each held record exactly once: {reply}");
        }
    }

    // Neither an update nor an export should mislabel a held annotation.
    run(&mut s, "drawing.set", json!({"updates": "deferred"}));
    run(&mut s, "drawing.update", json!({}));
    let after_update = checked(&mut s, json!({}));
    let mut after = whys(&after_update, "out_of_date");
    after.sort();
    assert_eq!(after, expected, "updating cannot unhold exploded-view annotations");

    // Once the view is assembled again, the two records must no longer be held.
    run(&mut s, "drawing.view.edit",
        json!({"view": ids::encode(view), "set": {"exploded": "assembled"}}));
    let assembled = checked(&mut s, json!({}));
    let held_again = of_kind(&assembled, "out_of_date");
    for record in &held {
        assert!(!held_again.iter().any(|(_, id)| id == record.as_str().unwrap()),
            "assembled view must release its note/dimension: {assembled}");
    }
}
