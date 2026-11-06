/// T91: capture real paper SVG and PNG before and after a 3D feature edit, with a dimension.
/// This is a native engine drawing export, not a browser screenshot.
#[test]
fn t91_a_3d_edit_redraws_the_2d_sheet_and_keeps_its_dimension_in_png_and_svg() {
    let destination = std::env::var("SOLIDCRAFT_T91_EVIDENCE_DIR").expect("T91 evidence output directory");
    let folder = std::path::PathBuf::from(destination);
    std::fs::create_dir_all(&folder).unwrap();

    let (mut s, sheet, [front, _top]) = front_and_top();
    // A bottom-edge measurement remains valid when the box grows in height.
    let added = run(
        &mut s,
        "drawing.dimension.add",
        json!({"view": ids::encode(front), "kind": {"linear": {"a": [0, 0, 0], "b": [100, 0, 0]}}}),
    );
    let dim = added["dimension"].clone();
    assert!(dim.is_string(), "{added}");

    let snapshot = |session: &mut Session, stage: &str| -> [Vec<u8>; 2] {
        let mut outputs = Vec::new();
        for format in ["svg", "png"] {
            let file = folder.join(format!("{stage}.{format}"));
            let mut params = json!({
                "sheet": ids::encode(sheet),
                "format": format,
                "path": file.to_str().unwrap()
            });
            if format == "png" {
                params["dpi"] = json!(90);
            }
            let result = run(session, CMD, params);
            let views = result["views"].as_array().expect("exported views list");
            assert_eq!(views.len(), 2, "{result}");
            assert!(views.iter().all(|v| v["status"] == "drawn"), "{result}");
            let file_bytes = std::fs::read(&file).unwrap();
            assert_eq!(result["bytes"].as_u64(), Some(file_bytes.len() as u64));
            assert!(file_bytes.len() > 500, "Unexpected empty {stage} {format}");
            if format == "svg" {
                assert!(file_bytes.windows(4).any(|w| w == b"<svg"));
            } else {
                assert!(file_bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
            }
            outputs.push(file_bytes);
        }
        let inspected = run(session, "drawing.dimension.inspect", json!({"dimension": dim}));
        assert_eq!(inspected["status"], "ok", "{stage}: {inspected}");
        assert!(inspected["warnings"].is_null(), "{stage}: {inspected}");
        outputs.try_into().unwrap()
    };

    let before = snapshot(&mut s, "before");
    let drawing = ids::encode(s.open_drawing.unwrap());
    run(&mut s, "drawing.close", json!({}));
    let block = s.doc.features[0].id;
    run(
        &mut s,
        "timeline.edit",
        json!({"feature": block.to_string(), "set": {"height": 40}}),
    );
    run(&mut s, "drawing.open", json!({"drawing": drawing}));
    let after = snapshot(&mut s, "after");

    assert_ne!(before[0], after[0], "SVG must change when the model changes");
    assert_ne!(before[1], after[1], "PNG must change when the model changes");
    println!(
        "T91_VISUAL_EVIDENCE=PASS; BEFORE_SVG={}; AFTER_SVG={}; BEFORE_PNG={}; AFTER_PNG={}; DIMENSION=OK",
        before[0].len(), after[0].len(), before[1].len(), after[1].len()
    );
}
