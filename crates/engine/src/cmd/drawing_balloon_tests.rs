use serde_json::json;
use solidcraft_doc::drawing::view::{View, ViewKind, ViewPlace};
use solidcraft_doc::drawing_view::{ViewDirection, ViewOverrides};
use solidcraft_geom::drawing_view::NamedView;

use super::*;
use crate::schema::tests::conforms;
use crate::tests::Folder;

fn run(s: &mut Session, id: &str, p: Value) -> Value {
    let reply = s.execute(id, &p).unwrap_or_else(|e| panic!("{id}: {e}"));
    let schema = output_schema_for::<BalloonResult>();
    assert!(conforms(schema.as_value(), schema.as_value(), &reply) || !matches!(id, ADD | INSPECT), "{id}: {reply}");
    reply
}

fn refused(s: &mut Session, cmd: &str, p: Value) -> String {
    s.execute(cmd, &p).err().map(|e| e.to_string()).unwrap_or_else(|| panic!("accepted: {p}"))
}

fn saved(s: &Session) -> String {
    s.drawing("test").unwrap().to_json().unwrap()
}

/// A component, Bracket, holding a box 80 x 40 x 30 mm along X, Y and Z from the origin, called
/// Box, placed once where it was made; a 10 mm cube of the design's own, Base, at x = 200; and a
/// drawing of them, open, with a front view at 1:1. The session, the view's id and the Bracket's
/// component and occurrence.
fn placed() -> (Session, u64, [u64; 2]) {
    let mut s = Session::default();
    let made = run(&mut s, "component.create", json!({"name": "Bracket"}));
    let bracket = [made["component"].as_u64().unwrap(), made["occurrence"].as_u64().unwrap()];
    run(&mut s, "solid.box", json!({"length": 80, "width": 40, "height": 30, "body_name": "Box"}));
    run(&mut s, "component.activate", json!({"component": "root"}));
    run(&mut s, "solid.box", json!({"length": 10, "width": 10, "height": 10, "corner": [200, 0, 0], "operation": "new", "body_name": "Base"}));
    run(&mut s, "drawing.create", json!({}));
    let mut d = (**s.drawing("test").unwrap()).clone();
    let id = d.allocate_id(&mut ids::IdSource::seeded(3874)).unwrap();
    let (sheet, source) = (d.sheets[0].id, d.sources[0].id);
    let kind = ViewKind::Base { source, view: ViewDirection::Named(NamedView::Front) };
    let settings = ViewOverrides { scale: Some("1:1".parse().unwrap()), ..ViewOverrides::default() };
    d.add_views([Arc::new(View { id, sheet, kind, place: ViewPlace::At([200.0, 150.0]), settings, reference_shown_percent: 0.0 })]);
    d.validate().unwrap();
    s.doc_mut().put_drawing(Arc::new(d));
    (s, id, bracket)
}

/// A balloon on the Box's bottom front edge, through the Bracket, at (60, 40) on paper.
fn add(s: &mut Session, view: u64, occurrence: u64) -> Value {
    run(s, ADD, json!({"view": ids::encode(view), "edge": {"point": [40, 0, 0], "path": [occurrence]}, "at": [60, 40]}))
}

fn ends_at(reply: &Value, p: [f64; 3]) {
    let point: Vec<f64> = reply["point"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect();
    assert!(point.iter().zip(p).all(|(a, b)| (a - b).abs() < 1e-9), "{reply}");
}

/// A balloon numbers the part its edge is placed through, saves the edge by name (any point on it
/// picks the same record), and its arrow ends on the edge's middle, whatever point was picked. An
/// add is one undo step.
#[test]
fn a_balloon_points_at_a_placed_part_and_is_saved_by_name() {
    let (mut s, view, [_, bracket]) = placed();
    let before = saved(&s);
    let reply = add(&mut s, view, bracket);
    assert_eq!((reply["status"].as_str(), reply["exact"].as_bool()), (Some("ok"), Some(true)), "{reply}");
    assert_eq!(reply["occurrence"], json!([bracket]));
    assert_eq!(reply["view"], json!(ids::encode(view)));
    assert_eq!(reply["record"]["edge"]["path"], json!([bracket]));
    assert_eq!(reply["record"]["at"], json!([60.0, 40.0]));
    ends_at(&reply, [40.0, 0.0, 0.0]);
    let elsewhere = run(&mut s, ADD, json!({"view": ids::encode(view), "edge": {"point": [7.5, 0, 0], "path": [bracket]}, "at": [0, 0]}));
    assert_eq!(elsewhere["record"]["edge"], reply["record"]["edge"], "saved by name, not by the point");
    ends_at(&elsewhere, [40.0, 0.0, 0.0]);
    run(&mut s, "edit.undo", json!({}));
    let one = saved(&s);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(saved(&s), before);
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(saved(&s), one);
}

/// What a balloon can't point at is refused with its reason, and the drawing is left as it was: a
/// view the drawing doesn't have, a body of the design itself (no part), an occurrence that isn't
/// there, a point on no edge, and a place off paper.
#[test]
fn a_balloon_is_refused_what_it_cant_point_at() {
    let (mut s, view, [_, bracket]) = placed();
    let before = saved(&s);
    let v = json!(ids::encode(view));
    let nowhere = ids::encode(99);
    let cases = [
        (json!({"view": nowhere, "edge": {"point": [40, 0, 0], "path": [bracket]}, "at": [0, 0]}), format!("the drawing has no view {nowhere}")),
        (json!({"view": v, "edge": [205, 0, 0], "at": [0, 0]}), "it points at a placed part: its edge's body is placed through an occurrence".into()),
        (json!({"view": v, "edge": {"point": [40, 0, 0], "path": [999]}, "at": [0, 0]}), "no edge at [40, 0, 0]".into()),
        (json!({"view": v, "edge": {"point": [40, 20, 0], "path": [bracket]}, "at": [0, 0]}), "no edge at [40, 20, 0]".into()),
        (json!({"view": v, "edge": {"point": [40, 0, 0], "path": [bracket]}, "at": [5000.001, 0]}), "its place is at most 5000 mm on paper".into()),
    ];
    for (p, why) in cases {
        let e = refused(&mut s, ADD, p.clone());
        assert!(e.contains(&why), "{p}: {e}");
    }
    assert_eq!(saved(&s), before);
}

/// An edit moves a balloon and names what changed, as one undo step; the same place again changes
/// nothing and adds no step. What it points at isn't one of its settings.
#[test]
fn an_edit_moves_a_balloon_and_the_same_place_again_changes_nothing() {
    let (mut s, view, [_, bracket]) = placed();
    let id = add(&mut s, view, bracket)["balloon"].clone();
    let before = saved(&s);
    let moved = run(&mut s, EDIT, json!({"balloon": id, "set": {"at": [-12.5, 80]}}));
    assert_eq!(moved, json!({"balloon": id, "changed": ["at"]}));
    assert_eq!(run(&mut s, INSPECT, json!({"balloon": id}))["record"]["at"], json!([-12.5, 80.0]));
    let after = saved(&s);
    for same in [json!({"at": [-12.5, 80]}), json!({})] {
        assert_eq!(run(&mut s, EDIT, json!({"balloon": id, "set": same}))["changed"], json!([]));
    }
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(saved(&s), before, "one step for the move, none for the same place");
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(saved(&s), after);
    let e = refused(&mut s, EDIT, json!({"balloon": id, "set": {"edge": [40, 0, 0]}}));
    assert!(e.contains("edge"), "{e}");
    let e = refused(&mut s, EDIT, json!({"balloon": ids::encode(99), "set": {"at": [0, 0]}}));
    assert!(e.contains("the drawing has no balloon"), "{e}");
    assert_eq!(saved(&s), after);
}

/// Inspect says where a balloon stands with the model now: a notch across its edge splits it, so it
/// lists both pieces and has no middle; undone, it is whole again; an occurrence that is gone is
/// lost, said so, and never moved to another part. The model is changed with the drawing closed.
#[test]
fn inspect_says_where_a_balloon_stands_with_the_model() {
    let (mut s, view, [component, bracket]) = placed();
    let drawing = json!({"drawing": ids::encode(s.drawing("test").unwrap().id)});
    let id = add(&mut s, view, bracket)["balloon"].clone();
    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "component.activate", json!({"component": component}));
    run(&mut s, "solid.box", json!({"length": 10, "width": 5, "height": 5, "corner": [35, -1, -1], "operation": "cut", "targets": ["Box"]}));
    run(&mut s, "component.activate", json!({"component": "root"}));
    run(&mut s, "drawing.open", drawing.clone());
    let split = run(&mut s, INSPECT, json!({"balloon": id}));
    assert_eq!(split["status"], "ambiguous", "{split}");
    assert_eq!(split["candidates"].as_array().map(Vec::len), Some(2), "{split}");
    assert!(split.get("point").is_none(), "no one middle: {split}");
    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "drawing.open", drawing);
    let whole = run(&mut s, INSPECT, json!({"balloon": id}));
    assert_eq!(whole["status"], "ok", "{whole}");
    ends_at(&whole, [40.0, 0.0, 0.0]);
    let mut d = (**s.drawing("test").unwrap()).clone();
    let b = d.balloons.first_mut().map(Arc::make_mut).unwrap();
    b.edge.path = vec![999];
    s.doc_mut().put_drawing(Arc::new(d));
    let gone = run(&mut s, INSPECT, json!({"balloon": id}));
    assert_eq!((gone["status"].as_str(), gone["occurrence"].clone()), (Some("lost"), json!([999])), "{gone}");
    assert!(gone["why"].as_str().is_some_and(|why| !why.is_empty()) && gone.get("point").is_none(), "{gone}");
    let e = refused(&mut s, INSPECT, json!({"balloon": ids::encode(99)}));
    assert!(e.contains("the drawing has no balloon"), "{e}");
}

/// A balloon put on a piece of a cut edge is found through the piece's numbered name, so not exact.
/// Once the cut is suppressed, the two faces the name names meet in one whole edge again, found
/// exactly, and the arrow moves to its middle.
#[test]
fn a_balloon_on_a_piece_is_not_exact_until_the_edge_is_whole_again() {
    let (mut s, view, [component, bracket]) = placed();
    let drawing = json!({"drawing": ids::encode(s.drawing("test").unwrap().id)});
    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "component.activate", json!({"component": component}));
    let cut = run(
        &mut s,
        "solid.box",
        json!({"length": 10, "width": 5, "height": 5, "corner": [35, -1, -1], "operation": "cut", "targets": ["Box"]}),
    )["feature"]
        .clone();
    run(&mut s, "component.activate", json!({"component": "root"}));
    run(&mut s, "drawing.open", drawing.clone());
    let piece = run(&mut s, ADD, json!({"view": ids::encode(view), "edge": {"point": [10, 0, 0], "path": [bracket]}, "at": [60, 40]}));
    assert_eq!((piece["status"].as_str(), piece["exact"].as_bool()), (Some("ok"), Some(false)), "{piece}");
    ends_at(&piece, [17.5, 0.0, 0.0]);
    run(&mut s, "drawing.close", json!({}));
    run(&mut s, "component.activate", json!({"component": component}));
    run(&mut s, "timeline.suppress", json!({"feature": cut, "suppressed": true}));
    run(&mut s, "component.activate", json!({"component": "root"}));
    run(&mut s, "drawing.open", drawing);
    let whole = run(&mut s, INSPECT, json!({"balloon": piece["balloon"]}));
    assert_eq!((whole["status"].as_str(), whole["exact"].as_bool()), (Some("ok"), Some(true)), "{whole}");
    ends_at(&whole, [40.0, 0.0, 0.0]);
}

/// Every place a value can go in `v`, as JSON pointers: `v` itself, then each member and item.
fn places(v: &Value, at: String, out: &mut Vec<String>) {
    out.push(at.clone());
    match v {
        Value::Object(o) => o.iter().for_each(|(k, x)| places(x, format!("{at}/{k}"), out)),
        Value::Array(l) => l.iter().enumerate().for_each(|(i, x)| places(x, format!("{at}/{i}"), out)),
        _ => {}
    }
}

/// A hostile value in each place of each command's parameters, at every depth (the edge's pick and
/// path, the place, the edit's settings): never a panic, and the drawing keeps its rules.
#[test]
fn hostile_balloon_params_never_panic() {
    let (mut s, view, [_, bracket]) = placed();
    let id = add(&mut s, view, bracket)["balloon"].clone();
    let v = json!(ids::encode(view));
    let calls = [
        (ADD, json!({"view": v, "edge": {"point": [40, 0, 0], "path": [bracket]}, "at": [1, 2]})),
        (ADD, json!({"view": v, "edge": {"body": "Box", "index": 0, "path": [bracket]}, "at": [1, 2]})),
        (EDIT, json!({"balloon": id, "set": {"at": [3, 4]}})),
        (INSPECT, json!({"balloon": id})),
    ];
    let mut values = crate::tests::hostile_values();
    values.extend([json!({"body": "Box", "index": u64::MAX}), Value::Array(vec![json!(1); 1000]), json!([1e308, 0]), json!("x".repeat(600))]);
    let mut internal = Vec::new();
    for (cmd, template) in calls {
        let mut at = Vec::new();
        places(&template, String::new(), &mut at);
        for (place, value) in at.iter().flat_map(|place| values.iter().map(move |value| (place, value))) {
            let mut p = template.clone();
            if let Some(slot) = p.pointer_mut(place) {
                *slot = value.clone();
            }
            let mut copy = s.scratch();
            if let Err(crate::EngineError::Internal(_, msg)) = copy.execute(cmd, &p) {
                internal.push(format!("{cmd} {p}: {msg}"));
            }
            copy.doc.drawings.iter().for_each(|d| d.validate().unwrap());
        }
    }
    assert!(internal.is_empty(), "balloon commands panicked:\n{}", internal.join("\n"));
}

/// A view lists its balloons. A copied view's balloons are copies with ids of their own, pointing
/// at the same part from the same place, and one undo step takes the copy and them out. A sheet
/// with them on it is refused, naming them, until its contents go with it.
#[test]
fn balloons_go_with_their_view_when_it_is_copied_and_with_its_sheet() {
    let (mut s, view, [_, bracket]) = placed();
    let original = add(&mut s, view, bracket);
    let listed = |s: &mut Session, view: &Value| run(s, "drawing.view.inspect", json!({"view": view}))["balloons"].clone();
    assert_eq!(listed(&mut s, &json!(ids::encode(view))), json!([original["balloon"]]));
    let before = saved(&s);
    let sheet = run(&mut s, "drawing.sheet.add", json!({"name": "Sheet 2"}))["sheet"].clone();
    let reply = run(&mut s, "drawing.view.copy_to_sheet", json!({"view": ids::encode(view), "sheet": sheet, "position_mm": [100, 100]}));
    let copy = reply["view"].clone();
    let copied = listed(&mut s, &copy);
    let [balloon] = copied.as_array().map(Vec::as_slice).unwrap_or_default() else { panic!("one copied balloon: {copied}") };
    assert_ne!(balloon, &original["balloon"]);
    let read = run(&mut s, INSPECT, json!({"balloon": balloon}));
    assert_eq!((read["status"].as_str(), &read["view"]), (Some("ok"), &copy), "{read}");
    assert_eq!((&read["record"]["edge"], &read["record"]["at"]), (&original["record"]["edge"], &original["record"]["at"]));
    assert_eq!(listed(&mut s, &json!(ids::encode(view))), json!([original["balloon"]]), "the original keeps its own");
    let e = refused(&mut s, "drawing.sheet.delete", json!({"sheet": sheet}));
    let named = format!("1 view (`{}`) and 1 balloon (`{}`)", copy.as_str().unwrap(), balloon.as_str().unwrap());
    assert!(e.contains(&named), "{e}");
    run(&mut s, "drawing.sheet.delete", json!({"sheet": sheet, "contents": "delete"}));
    let e = refused(&mut s, INSPECT, json!({"balloon": balloon}));
    assert!(e.contains("the drawing has no balloon"), "{e}");
    for _ in 0..3 {
        run(&mut s, "edit.undo", json!({}));
    }
    assert_eq!(saved(&s), before);
}

/// The W23 assembly sheet's test assembly (#11773), from its command script, and the occurrences
/// the script names: the plate, the first screw, the two holders and the block placed in each.
fn assembly_sheet() -> (Session, [u64; 5]) {
    let script: Value = serde_json::from_str(include_str!("../../../../tests/fixtures/w23-assembly-sheet/assembly.json")).unwrap();
    let mut s = Session::default();
    let made = s.run_script(&script).unwrap();
    let named = |name: &str| {
        let step = script["commands"].as_array().unwrap().iter().position(|c| c["as"] == name).unwrap();
        made[step]["occurrence"].as_u64().unwrap()
    };
    let occurrences = ["plate", "screw", "first_holder", "second_holder", "block"].map(named);
    (s, occurrences)
}

/// Balloons on the assembly sheet's top view and the drawing's revision rows, through model edits
/// made with the drawing closed, their undo and redo, and save and reopen: a balloon follows its
/// occurrence; the second holder's balloon reads lost when that holder is deleted, and is never
/// moved to the first holder or removed; the drawing's saved record and its revision rows never
/// change with the model.
#[test]
fn balloons_on_the_assembly_sheet_follow_the_model_and_its_revision_rows_stay() {
    let (mut s, [plate, screw, first, second, block]) = assembly_sheet();
    run(&mut s, "drawing.create", json!({}));
    let top = run(&mut s, "drawing.view.base", json!({"sheet": "Sheet 1", "view": "top", "scale": "1:2", "position_mm": [150, 150]}))["view"].clone();
    let drawing = ids::encode(s.drawing("test").unwrap().id);

    // Four balloons: the plate's top front edge, the first screw's top rim, and the block by its
    // top front edge through each holder.
    let put = |s: &mut Session, point: [f64; 3], path: Value, at: [f64; 2]| {
        let reply = run(s, ADD, json!({"view": top, "edge": {"point": point, "path": path}, "at": at}));
        assert_eq!((&reply["status"], &reply["occurrence"]), (&json!("ok"), &path), "{reply}");
        reply
    };
    let on_plate = put(&mut s, [60.0, 0.0, 10.0], json!([plate]), [-60.0, -50.0]);
    let on_screw = put(&mut s, [8.0, 5.0, 30.0], json!([screw]), [-70.0, 40.0]);
    let on_first = put(&mut s, [50.0, 10.0, 40.0], json!([first, block]), [0.0, 60.0]);
    let on_second = put(&mut s, [50.0, 50.0, 40.0], json!([second, block]), [40.0, 60.0]);
    ends_at(&on_plate, [60.0, 0.0, 10.0]);
    ends_at(&on_first, [50.0, 10.0, 40.0]);
    ends_at(&on_second, [50.0, 50.0, 40.0]);
    assert_ne!(on_first["record"]["edge"], on_second["record"]["edge"], "one block, two placements, two balloons");
    let balloons = [&on_plate, &on_screw, &on_first, &on_second].map(|b| b["balloon"].clone());

    // Revision rows 1, 2 and 3; row 2 deleted: 1 and 3 stay, and the next is still 4.
    let mut rows = Vec::new();
    for (description, date) in [("First issue", "2026-11-03"), ("Withdrawn", "2026-11-04"), ("Holder moved", "2026-11-05")] {
        rows.push(run(&mut s, "drawing.revision.add", json!({"set": {"description": description, "designer": "BS", "date": date}}))["row"].clone());
    }
    run(&mut s, "drawing.revision.delete", json!({"row": rows[1]}));
    let revisions = run(&mut s, "drawing.inspect", json!({}))["revisions"].clone();
    let numbers: Vec<&str> = revisions["rows"].as_array().unwrap().iter().map(|r| r["revision"].as_str().unwrap()).collect();
    assert_eq!((numbers, revisions["next"].as_str()), (vec!["1", "3"], Some("4")), "{revisions}");

    // The model is edited with the drawing closed, then the drawing is opened again.
    let record = saved(&s);
    let on_the_model = |s: &mut Session, id: &str, p: Value| {
        run(s, "drawing.close", json!({}));
        run(s, id, p);
        run(s, "drawing.open", json!({"drawing": drawing}));
    };
    let read = |s: &mut Session| balloons.clone().map(|b| run(s, INSPECT, json!({"balloon": b})));
    let statuses = |reads: &[Value; 4]| reads.clone().map(|r| r["status"].as_str().unwrap().to_string());
    let ok = ["ok", "ok", "ok", "ok"].map(String::from);

    // The second holder moved 30 mm along Y: its balloon's arrow follows, the others stay.
    on_the_model(&mut s, "occurrence.move", json!({"occurrence": second, "space": "world", "translate": [0, 30, 0]}));
    let moved = read(&mut s);
    assert_eq!(statuses(&moved), ok);
    ends_at(&moved[0], [60.0, 0.0, 10.0]);
    ends_at(&moved[2], [50.0, 10.0, 40.0]);
    ends_at(&moved[3], [50.0, 80.0, 40.0]);
    assert_eq!(saved(&s), record, "a model edit leaves the drawing's record alone");
    on_the_model(&mut s, "edit.undo", json!({}));
    ends_at(&read(&mut s)[3], [50.0, 50.0, 40.0]);

    // The second holder deleted: its balloon is lost, says why, keeps its occurrence and is still
    // there; the others stay ok.
    on_the_model(&mut s, "selection.delete", json!({"occurrences": [second]}));
    let gone = read(&mut s);
    assert_eq!(statuses(&gone), ["ok", "ok", "ok", "lost"].map(String::from), "{gone:?}");
    assert!(gone[3]["why"].is_string(), "{}", gone[3]);
    assert_eq!(gone[3]["occurrence"], json!([second, block]), "never moved to the first holder");
    assert_eq!(gone[3]["record"], on_second["record"], "never removed or changed");
    ends_at(&gone[2], [50.0, 10.0, 40.0]);
    assert_eq!(saved(&s), record);

    // Saved with the holder deleted and opened again: the same record, statuses and revisions.
    let folder = Folder::new("assembly-sheet-balloons");
    let path = folder.path("assembly.solidcraft");
    run(&mut s, "file.save", json!({"path": path}));
    let mut opened = Session::default();
    run(&mut opened, "doc.open", json!({"path": path}));
    run(&mut opened, "drawing.open", json!({"drawing": drawing}));
    assert_eq!(saved(&opened), record, "opened again");
    let again = read(&mut opened);
    assert_eq!(statuses(&again), statuses(&gone));
    for (was, is) in gone.iter().zip(&again) {
        assert_eq!((&is["record"], &is["occurrence"]), (&was["record"], &was["occurrence"]), "{is}");
    }
    assert_eq!(run(&mut opened, "drawing.inspect", json!({}))["revisions"], revisions, "revision rows opened again");

    // Undone: the second holder and its balloon are back; redone, lost again; undone once more.
    on_the_model(&mut s, "edit.undo", json!({}));
    let back = read(&mut s);
    assert_eq!(statuses(&back), ok, "{back:?}");
    ends_at(&back[3], [50.0, 50.0, 40.0]);
    on_the_model(&mut s, "edit.redo", json!({}));
    assert_eq!(statuses(&read(&mut s))[3], "lost");
    on_the_model(&mut s, "edit.undo", json!({}));
    assert_eq!(statuses(&read(&mut s)), ok);

    // Through every model edit, the drawing's record and its revision rows are as they were.
    assert_eq!(saved(&s), record);
    assert_eq!(run(&mut s, "drawing.inspect", json!({}))["revisions"], revisions);
}
