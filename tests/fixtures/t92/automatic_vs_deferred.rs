/// T92 (B): Verify the exact difference between automatic paper refresh and
/// deliberately held/deferred 2D drawings after the same 3D length change.
///
/// Four/five real SVG snapshots are written as independent CI evidence.
/// They are engine paper exports, not interactive browser screenshots.
#[test]
fn t92_automatic_vs_deferred_3d_to_2d_dimension_updates() {
    let root = std::path::PathBuf::from(
        std::env::var("SOLIDCRAFT_T92_EVIDENCE_DIR").expect("Set SOLIDCRAFT_T92_EVIDENCE_DIR"),
    );
    std::fs::create_dir_all(&root).unwrap();
    for deferred in [false, true] {
        let mode = if deferred { "deferred" } else { "automatic" };
        let (mut s, sheet, [front, _]) = front_and_top();
        let dim = front_edge(&mut s, front);
        if deferred {
            run(&mut s, "drawing.set", json!({"updates": "deferred"}));
            run(&mut s, "drawing.update", json!({}));
        }
        let before = settled(&mut s, sheet);
        assert!(before.stale.is_empty(), "{mode}: before view stale");
        let dimension_before = bytes_of(group_of(&before.scene, dim));
        let save_svg = |session: &mut Session, stage: &str| -> String {
            let output = run(
                session,
                "drawing.export",
                json!({"sheet": ids::encode(sheet), "format": "svg"}),
            );
            let svg = output["svg"].as_str().expect("inline SVG export").to_owned();
            assert!(svg.contains("<svg") && svg.len() > 500, "{mode}/{stage}: invalid SVG");
            let filename = root.join(format!("{mode}-{stage}.svg"));
            std::fs::write(filename, &svg).unwrap();
            svg
        };
        let svg_before = save_svg(&mut s, "before");
        let drawing = ids::encode(s.open_drawing.unwrap());
        run(&mut s, "drawing.close", json!({}));
        let block = s.doc.features[0].id;
        run(&mut s, "timeline.edit", json!({"feature": block.to_string(), "set": {"length": 140}}));
        run(&mut s, "drawing.open", json!({"drawing": drawing}));
        if deferred {
            let held = settled(&mut s, sheet);
            assert_eq!(held.stale.len(), 2, "held 2D views must be reported stale");
            assert_eq!(bytes(&held.scene), bytes(&before.scene), "held paper stays unchanged");
            assert_eq!(listed(&mut s, sheet).len(), 3, "two views and one dimension out of date");
            let held_status = run(&mut s, "drawing.status", json!({}));
            assert_eq!(held_status["held"], true, "manual update required: {held_status}");
            let svg_held = save_svg(&mut s, "held-before-manual-update");
            assert_eq!(svg_held, svg_before, "held paper must not silently refresh");
            run(&mut s, "drawing.update", json!({}));
        }
        // Automatic mode never calls drawing.update. Settling means its provider
        // completes background/step drawing work, as the UI would.
        let after = settled(&mut s, sheet);
        assert!(after.stale.is_empty(), "{mode}: updated sheet has stale view(s)");
        assert_eq!(bytes(&after.scene), bytes(&built(&s, sheet).scene), "{mode}: outdated rendered scene");
        let dimension_after = bytes_of(group_of(&after.scene, dim));
        assert_ne!(dimension_after, dimension_before, "{mode}: 100 to 140 mm dimension did not redraw");
        let svg_after = save_svg(&mut s, "after");
        assert_ne!(svg_after, svg_before, "{mode}: model edit did not change SVG");
        assert!(listed(&mut s, sheet).is_empty(), "{mode}: stale annotations remain");
        println!("T92_MODE={mode}; DIMENSION_REDRAW=PASS; PAPER_SVG_CHANGED=PASS");
    }
    println!("FINAL=T92_AUTOMATIC_AND_DEFERRED_DRAWING_POLICY_PASS");
}
