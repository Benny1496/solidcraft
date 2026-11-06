/// On the W23 assembly sheet, a balloon on a block inside the hidden second holder is refused.
/// The refusal keeps the drawing unchanged. A visible holder still accepts a balloon,
/// and the hidden holder accepts one once its view shows it again.
#[test]
fn the_assembly_sheet_refuses_a_balloon_on_its_hidden_second_holder() {
    let (mut s, [_, _, first, second, block]) = assembly_sheet();
    run(&mut s, "drawing.create", json!({}));
    let top = run(
        &mut s,
        "drawing.view.base",
        json!({
            "sheet": "Sheet 1",
            "view": "top",
            "scale": "1:2",
            "position_mm": [150, 150],
            "hide": [[second]]
        }),
    )["view"]
        .clone();
    let view = top.as_str().unwrap().to_owned();

    let on = |holder: u64, point: [f64; 3], at: [f64; 2]| {
        json!({
            "view": top,
            "edge": {"point": point, "path": [holder, block]},
            "at": at
        })
    };

    let before = saved(&s);
    let error = refused(&mut s, ADD, on(second, [50.0, 50.0, 40.0], [40.0, 60.0]));
    let reason = format!("view {view} doesn't draw that part: its hide leaves out occurrence path [{second}]");
    assert!(error.contains(&reason), "{error}");
    assert_eq!(saved(&s), before, "a refused balloon must not change the drawing");

    let first_balloon = run(&mut s, ADD, on(first, [50.0, 10.0, 40.0], [0.0, 60.0]));
    assert_eq!(first_balloon["status"], "ok", "{first_balloon}");
    assert_eq!(first_balloon["occurrence"], json!([first, block]), "{first_balloon}");

    run(&mut s, "drawing.view.edit", json!({"view": top, "set": {"hide": null}}));

    let second_balloon = run(&mut s, ADD, on(second, [50.0, 50.0, 40.0], [40.0, 60.0]));
    assert_eq!(second_balloon["status"], "ok", "{second_balloon}");
    assert_eq!(second_balloon["occurrence"], json!([second, block]), "{second_balloon}");
    assert_ne!(first_balloon["balloon"], second_balloon["balloon"], "different holders keep distinct balloon ids");
}
