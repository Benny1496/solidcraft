//! Proposed W23a E2E-01 / E2E-02 native engine regression tests for the official drawings suite.
//! Scope: executable engine-command subsets only; full UI/U2 and real file reopen remain NOT_RUN.
use std::sync::Arc;
use serde_json::{json, Value};
use solidcraft_doc::drawing::view::{View, ViewKind, ViewPlace};
use solidcraft_doc::drawing_view::{ViewDirection, ViewOverrides};
use solidcraft_doc::ids::{self, IdSource};
use solidcraft_geom::drawing_view::NamedView;
use solidcraft_geom::Vec3;
use crate::Session;

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command} {params}: {e}"))
}

fn index_of_bore_edge(s: &Session, centre: [f64; 3], radius: f64) -> usize {
    let c = Vec3::from(centre);
    let state = s.model.state();
    let mesh = state.body("Plate").expect("Plate body").mesh();
    mesh.edges.iter().position(|line| {
        line.len() >= 2 && line.iter().all(|p| (p.dist(c) - radius).abs() < 1e-5)
    }).unwrap_or_else(|| panic!("missing circular edge at {centre:?}, radius {radius}"))
}

fn inspect(s: &mut Session, id: &Value) -> Value {
    run(s, "drawing.dimension.inspect", json!({"dimension": id}))
}

fn assert_ok(reply: &Value, diameter_mm: f64) {
    assert_eq!(reply["status"], json!("ok"), "unexpected dimension state: {reply}");
    let v = reply["value"].as_f64().expect("measured dimension value");
    assert!((v - diameter_mm).abs() < 1e-8, "expected diameter {diameter_mm}, got {reply}");
    assert_eq!(reply["record"]["kind"]["radial"]["diameter"], json!(true), "{reply}");
}

// Compare actual CAD geometry against the original analytic W23a oracle volumes.
fn assert_analytic_volume(s: &mut Session, reference: f64, oracle: &str) {
    let reply = run(s, "inspect.measure", json!({}));
    assert_eq!(reply["body_count"], json!(1), "{oracle} unexpected bodies: {reply}");
    let value = reply["total"]["volume_mm3"].as_f64()
        .unwrap_or_else(|| panic!("{oracle} missing CAD volume: {reply}"));
    assert!((value-reference).abs() < reference.abs() * 0.001,
           "{oracle} CAD volume {value} differs from analytic {reference}");
}

struct Plate {
    session: Session,
    drawing: String,
    bore1: Value,
    bore2: Value,
}
impl Plate {
    fn open(&mut self) {
        run(&mut self.session, "drawing.open", json!({"drawing": self.drawing}));
    }
    fn close(&mut self) {
        run(&mut self.session, "drawing.close", json!({}));
    }
    fn change_bore1_to_12(&mut self) {
        self.close();
        let changed = run(&mut self.session, "parameters.change",
                          json!({"name": "bore1_d", "expression": "12 mm"}));
        assert!(changed["errors"].as_array().is_some_and(|a| a.is_empty()), "{changed}");
        assert_analytic_volume(&mut self.session, 156167.25696262045, "oracle 86");
        self.open();
    }
}
fn plate_two_bores() -> Plate {
    let mut s = Session::default();
    // Replay the *real* official Oracle #84 sketch/extrude/solid.hole recipe.
    // The commands JSON comes from the actual pinned SolidCraft CLI recipe converter.
    // Committed conversion of the official Oracle84 recipe; no external runner or CI-report file.
    let source = include_str!("../../../tests/fixtures/w23a-drawing-updates/oracle84-commands.json");
    let script: Value = serde_json::from_str(source).expect("oracle84 JSON");
    let steps = script["commands"].as_array().expect("commands");
    assert!(steps.len() > 3, "the real oracle must have several commands");
    let hole_count = steps.iter().filter(|step| step["command"] == "solid.hole").count();
    let extrusions = steps.iter().filter(|step| step["command"] == "solid.extrude").count();
    assert_eq!(hole_count, 2, "two real hole features expected");
    assert!(extrusions >= 1, "true extruded sketch required");
    for (index, step) in steps.iter().enumerate() {
        let cmd = step["command"].as_str().expect("command name");
        let params = step.get("params").cloned().unwrap_or_else(|| json!({}));
        let answer = s.execute(cmd, &params)
            .unwrap_or_else(|e| panic!("oracle84 command #{index}: {cmd} {params}: {e}"));
        if let Some(error) = answer.get("error") { panic!("oracle84 command #{index} ({cmd}) error: {error}"); }
    }
    assert_analytic_volume(&mut s, 156858.4073464102, "oracle 84 baseline");

    // Actually make a drawing and save a real top-view record through existing W19 API.
    run(&mut s, "drawing.create", json!({"name":"W23a test42"}));
    let mut d = (**s.drawing("test42").expect("open drawing")).clone();
    let drawing = ids::encode(d.id);
    let view = d.allocate_id(&mut IdSource::seeded(28280)).expect("view ID");
    let (sheet, source) = (d.sheets[0].id, d.sources[0].id);
    d.add_views([Arc::new(View {
        id:view, sheet,
        kind: ViewKind::Base { source, view: ViewDirection::Named(NamedView::Top) },
        place: ViewPlace::At([205.0, 150.0]),
        settings: ViewOverrides { scale: Some("1:1".parse().unwrap()), ..ViewOverrides::default() },
        reference_margin_mm:0.0,
    })]);
    d.validate().expect("valid drawing and top view");
    s.doc_mut().put_drawing(Arc::new(d));

    // Both dimensions are added by the actual public W19d/W20a drawing commands.
    let one = index_of_bore_edge(&s, [20.0, 20.0, 20.0], 5.0);
    let two = index_of_bore_edge(&s, [60.0, 50.0, 20.0], 5.0);
    assert_ne!(one, two, "two distinct holes");
    let bore1 = run(&mut s, "drawing.dimension.add",
        json!({"view": ids::encode(view), "kind": {"radial": {
              "edge": {"body":"Plate","index":one}, "diameter":true}}}))["dimension"].clone();
    let bore2 = run(&mut s, "drawing.dimension.add",
        json!({"view": ids::encode(view), "kind": {"radial": {
              "edge": {"body":"Plate","index":two}, "diameter":true}}}))["dimension"].clone();
    assert_ne!(bore1, bore2);
    assert_ok(&inspect(&mut s, &bore1), 10.0);
    assert_ok(&inspect(&mut s, &bore2), 10.0);
    Plate {session:s, drawing, bore1, bore2}
}


#[test]
fn w23a_official_bore1_10_to_12_keeps_dimension_id_and_independent_bore2() {
    let mut f = plate_two_bores();
    let id = f.bore1.clone();
    f.change_bore1_to_12();
    let a = inspect(&mut f.session, &f.bore1);
    assert_eq!(a["dimension"], id, "dimension ID changed");
    assert_ok(&a, 12.0);
    let other = inspect(&mut f.session, &f.bore2);
    assert_ok(&other, 10.0);
    println!("PASS: W23a E2E-01 early W19d/W20a inspect subset; diameter id={} now 12mm; Bore2 still 10mm",id);
}


#[test]
fn w23a_official_deleted_bore1_is_lost_never_bore2_then_undo() {
    let mut f = plate_two_bores();
    let id = f.bore1.clone();
    f.close();
    run(&mut f.session, "timeline.delete", json!({"feature":"Bore1"}));
    assert_analytic_volume(&mut f.session, 158429.2036732051, "oracle 87 after delete");
    f.open();
    let lost = inspect(&mut f.session, &f.bore1);
    assert_eq!(lost["dimension"], id, "dimension ID changed");
    assert_eq!(lost["status"], json!("lost"), "deleted Bore1 must be lost: {lost}");
    assert!(lost.get("value").is_none(), "lost Bore1 unexpectedly measures {lost}");
    let other = inspect(&mut f.session, &f.bore2);
    assert_ok(&other, 10.0);
    f.close();
    run(&mut f.session, "edit.undo", json!({}));
    f.open();
    let restored = inspect(&mut f.session, &f.bore1);
    assert_eq!(restored["dimension"], id);
    assert_ok(&restored, 10.0);
    println!("PASS: W23a E2E-02 early W19d/W20a inspect subset; Bore1 lost, Bore2 unchanged, undo restores");
}


#[test]
fn w23a_official_original_oracle84_change_undo_redo_preserves_measure_identity() {
    let mut f = plate_two_bores();
    let id = f.bore1.clone();
    f.change_bore1_to_12();
    assert_ok(&inspect(&mut f.session, &id), 12.0);
    f.close();
    run(&mut f.session, "edit.undo", json!({}));
    assert_analytic_volume(&mut f.session, 156858.4073464102, "oracle 84 undo");
    f.open();
    assert_ok(&inspect(&mut f.session, &id), 10.0);
    assert_ok(&inspect(&mut f.session, &f.bore2), 10.0);
    f.close();
    run(&mut f.session, "edit.redo", json!({}));
    assert_analytic_volume(&mut f.session, 156167.25696262045, "oracle 86 redo");
    f.open();
    let final_dimension = inspect(&mut f.session, &id);
    assert_eq!(final_dimension["dimension"], id);
    assert_ok(&final_dimension, 12.0);
    assert_ok(&inspect(&mut f.session, &f.bore2), 10.0);
    println!("PASS: real W23a oracle84 sketch/hole dimension identity through parameter change, undo, redo");
}


fn set_authored_text_at_paper(
    s: &mut Session, dimension: &Value, paper: [f64;2], view_centre: [f64;2]
) -> [f64;2] {
    // W19d text.at is relative to the centre of its view, NOT paper coordinates.
    let relative = [paper[0] - view_centre[0], paper[1] - view_centre[1]];
    let answer = run(s, "drawing.dimension.edit", json!({
        "dimension": dimension,
        "set": {"text": {"at": relative}}
    }));
    assert!(answer["changed"].as_array().is_some_and(|changes|
        changes.iter().any(|field| field == "text")), "text edit not applied: {answer}");
    relative
}

fn assert_text_relative(s: &mut Session, dimension: &Value, relative: [f64;2]) {
    let reply = inspect(s, dimension);
    assert_eq!(reply["dimension"], *dimension, "dimension id changed: {reply}");
    let at = reply["record"]["text"]["at"].as_array().unwrap_or_else(||
        panic!("text position missing from authored dimension: {reply}"));
    assert_eq!(at.len(), 2);
    for (i, goal) in relative.iter().enumerate() {
        let actual = at[i].as_f64().expect("numeric text position");
        assert!((actual-goal).abs()<1e-9, "text offset changed {relative:?}: {reply}");
    }
}


#[test]
fn w23a_official_user_authored_sheet_text_positions_survive_diameter_edit_and_undo_redo() {
    let mut f = plate_two_bores();
    let first=f.bore1.clone();
    let second=f.bore2.clone();
    // View place in fixture is [205,150] millimetres on paper.
    let view_centre=[205.0,150.0];
    let first_relative=set_authored_text_at_paper(&mut f.session,&first,[95.0,55.0],view_centre);
    let second_relative=set_authored_text_at_paper(&mut f.session,&second,[145.0,55.0],view_centre);
    assert_eq!(first_relative,[-110.0,-95.0]);
    assert_eq!(second_relative,[-60.0,-95.0]);
    assert_text_relative(&mut f.session,&first,first_relative);
    assert_text_relative(&mut f.session,&second,second_relative);
    f.change_bore1_to_12();
    assert_ok(&inspect(&mut f.session,&first),12.0);
    assert_ok(&inspect(&mut f.session,&second),10.0);
    assert_text_relative(&mut f.session,&first,first_relative);
    assert_text_relative(&mut f.session,&second,second_relative);
    f.close();
    run(&mut f.session,"edit.undo",json!({}));
    f.open();
    assert_ok(&inspect(&mut f.session,&first),10.0);
    assert_text_relative(&mut f.session,&first,first_relative);
    assert_text_relative(&mut f.session,&second,second_relative);
    f.close();
    run(&mut f.session,"edit.redo",json!({}));
    f.open();
    assert_ok(&inspect(&mut f.session,&first),12.0);
    assert_text_relative(&mut f.session,&first,first_relative);
    assert_text_relative(&mut f.session,&second,second_relative);
    println!("PASS: authored dimension text.at stays at the same sheet position after model edit/undo/redo (record contract, not visual rendering)");
}


fn add_second_front_view(f: &mut Plate) -> u64 {
    // The real A3 drawing from plate_two_bores() already owns a Top view.
    // Add a separate Front view, so we can verify independent freshness/step transitions.
    let mut d = (**f.session.drawing("test").expect("drawing open")).clone();
    let id = d.allocate_id(&mut IdSource::seeded(343434)).expect("second view id");
    let sheet = d.sheets[0].id;
    let source = d.sources[0].id;
    d.add_views([Arc::new(View {
        id, sheet,
        kind: ViewKind::Base { source, view: ViewDirection::Named(NamedView::Front) },
        place: ViewPlace::At([105.0, 100.0]),
        settings: ViewOverrides {
            scale: Some("1:2".parse().unwrap()), ..ViewOverrides::default()
        },
        reference_margin_mm: 0.0,
    })]);
    d.validate().expect("valid A3 drawing with two independent views");
    f.session.doc_mut().put_drawing(Arc::new(d));
    id
}


fn public_drawing_scene(s: &mut Session, sheet: u64, count: usize) -> Value {
    // This is the user-visible drawing.scene command, rather than the internal provider API.
    let answer = run(s, "drawing.scene", json!({"sheet": ids::encode(sheet)}));
    assert_eq!(answer["sheet"], json!(ids::encode(sheet)));
    let views = answer["views"].as_array().expect("public scene must contain view states");
    assert_eq!(views.len(), count, "missing view states: {answer}");
    assert!(views.iter().all(|v| v["status"] == "drawn"), "{answer}");
    assert!(answer["scene"]["groups"].as_array().is_some_and(|g| !g.is_empty()), "empty public scene: {answer}");
    answer
}

fn public_scene_matches_engine_provider(s: &mut Session, sheet: u64, count: usize) -> String {
    let public = public_drawing_scene(s, sheet, count);
    let _initial = crate::drawing_scene::extract(s, sheet).expect("extract");
    while crate::drawing_scene::step(s) {}
    let settled = crate::drawing_scene::extract(s, sheet).expect("settled");
    assert!(settled.stale.is_empty(), "provider stale after steps");
    assert_eq!(settled.views.len(), count);
    let source = serde_json::to_value(settled.scene.as_ref()).unwrap();
    assert_eq!(public["scene"],source,"public drawing.scene command and engine provider disagree");
    serde_json::to_string(&source).unwrap()
}


#[test]
fn w23a_official_public_drawing_scene_matches_two_views_after_bore_parameter_edit() {
    let mut f=plate_two_bores();
    let _front=add_second_front_view(&mut f);
    let sheet=f.session.drawing("test").unwrap().sheets[0].id;
    let base=public_scene_matches_engine_provider(&mut f.session,sheet,2);
    let a=f.bore1.clone();
    f.change_bore1_to_12();
    let modified=public_scene_matches_engine_provider(&mut f.session,sheet,2);
    assert_ne!(base,modified,"original model Ø10→Ø12 must change public 2D drawing output");
    let first=inspect(&mut f.session,&a);
    assert_eq!(first["dimension"],a);
    assert_ok(&first,12.0);
    assert_ok(&inspect(&mut f.session,&f.bore2),10.0);
    println!("PASS: public drawing.scene shows changed two-view 2D lines for original oracle84, and matches incremental engine after Ø10→Ø12");
}


#[test]
fn w23a_official_public_two_view_delete_undo_redo_stays_lost_without_rebinding() {
    let mut f = plate_two_bores();
    let _front = add_second_front_view(&mut f);
    let sheet = f.session.drawing("test").unwrap().sheets[0].id;
    let first = f.bore1.clone();
    let second = f.bore2.clone();
    let original = public_scene_matches_engine_provider(&mut f.session, sheet, 2);

    f.close();
    run(&mut f.session, "timeline.delete", json!({"feature":"Bore1"}));
    assert_analytic_volume(&mut f.session, 158429.2036732051, "oracle 87 delete Bore1");
    f.open();
    let deleted = public_scene_matches_engine_provider(&mut f.session, sheet, 2);
    assert_ne!(original, deleted, "deleting Bore1 must remove lines from public scene");
    let lost = inspect(&mut f.session, &first);
    assert_eq!(lost["dimension"], first, "deleted dimension ID changed");
    assert_eq!(lost["status"], "lost", "Bore1 must be lost, not assigned to Bore2");
    assert!(lost.get("value").is_none(), "a lost dimension must not present a live value");
    assert_ok(&inspect(&mut f.session, &second), 10.0);

    f.close();
    run(&mut f.session, "edit.undo", json!({}));
    assert_analytic_volume(&mut f.session, 156858.4073464102, "oracle 84 undo Bore1 delete");
    f.open();
    let restored = public_scene_matches_engine_provider(&mut f.session, sheet, 2);
    assert_eq!(original, restored, "undo must restore public 2D lines exactly");
    assert_ok(&inspect(&mut f.session, &first), 10.0);
    assert_ok(&inspect(&mut f.session, &second), 10.0);

    f.close();
    run(&mut f.session, "edit.redo", json!({}));
    assert_analytic_volume(&mut f.session, 158429.2036732051, "oracle 87 redo Bore1 delete");
    f.open();
    let again = public_scene_matches_engine_provider(&mut f.session, sheet, 2);
    assert_eq!(deleted, again, "redo must restore the deleted-hole 2D scene exactly");
    let again_lost = inspect(&mut f.session, &first);
    assert_eq!(again_lost["dimension"], first);
    assert_eq!(again_lost["status"], "lost");
    assert_ok(&inspect(&mut f.session, &second), 10.0);
    println!("PASS: public two-view drawing.scene after Bore1 delete/undo/redo matches exact 2D states; Bore1 stays lost without rebinding to Bore2");
}


fn assert_lost_with_session_last_good(reply: &Value, id: &Value, historical: f64) {
    assert_eq!(reply["dimension"], *id, "a lost record must preserve its identity: {reply}");
    assert_eq!(reply["status"], json!("lost"), "lost record must not silently bind elsewhere: {reply}");
    assert!(reply.get("value").is_none(), "a lost reference must have no live measured value: {reply}");
    let observed=reply["last_good"].as_f64().unwrap_or_else(||
        panic!("missing last good session value from a previously measured dimension: {reply}"));
    assert!((observed-historical).abs()<1e-8, "last_good incorrectly changed: expected {historical}, got {observed}");
}


#[test]
fn w23a_official_last_good_12_mm_stays_with_lost_dimension_through_delete_undo_redo() {
    let mut f=plate_two_bores();
    let _front=add_second_front_view(&mut f);
    let sheet=f.session.drawing("test").unwrap().sheets[0].id;
    let id=f.bore1.clone();
    let other=f.bore2.clone();
    let at=set_authored_text_at_paper(&mut f.session,&id,[95.0,55.0],[205.0,150.0]);
    assert_ok(&inspect(&mut f.session,&id),10.0);
    f.change_bore1_to_12();
    let valid=inspect(&mut f.session,&id);
    assert_eq!(valid["dimension"],id);
    assert_ok(&valid,12.0);
    assert_text_relative(&mut f.session,&id,at);
    let scene_valid=public_scene_matches_engine_provider(&mut f.session,sheet,2);

    f.close();
    run(&mut f.session,"timeline.delete",json!({"feature":"Bore1"}));
    assert_analytic_volume(&mut f.session,160000.0 - std::f64::consts::PI * 5.0 * 5.0 * 20.0,"one remaining Bore2");
    f.open();
    let scene_lost=public_scene_matches_engine_provider(&mut f.session,sheet,2);
    assert_ne!(scene_valid,scene_lost);
    let lost=inspect(&mut f.session,&id);
    assert_lost_with_session_last_good(&lost,&id,12.0);
    assert_text_relative(&mut f.session,&id,at);
    assert_ok(&inspect(&mut f.session,&other),10.0);
    // Do not assert on persisted last-good record: CamWight is changing that separately.
    // This test proves the *session read* only; file save/reopen is a distinct acceptance.

    f.close();
    run(&mut f.session,"edit.undo",json!({}));
    f.open();
    assert_eq!(scene_valid,public_scene_matches_engine_provider(&mut f.session,sheet,2));
    assert_ok(&inspect(&mut f.session,&id),12.0);
    assert_text_relative(&mut f.session,&id,at);

    f.close();
    run(&mut f.session,"edit.redo",json!({}));
    f.open();
    assert_eq!(scene_lost,public_scene_matches_engine_provider(&mut f.session,sheet,2));
    assert_lost_with_session_last_good(&inspect(&mut f.session,&id),&id,12.0);
    assert_ok(&inspect(&mut f.session,&other),10.0);
    assert_text_relative(&mut f.session,&id,at);
    println!("PASS: actual original Oracle84 Bore1 Ø12 read then deleted retains last_good=12 in session across undo/redo, not a persisted file; public 2D scenes and text record remain coherent");
}


// W23a E2E-04 executable *file persistence subset* on real two-bore Oracle84.
// The drawing engine still does not render live dimension annotations on paper;
// this tests real design-file I/O, model reference identities and authored records.
fn w23a_save_reopen_file(format_name: &str) {
    let mut f = plate_two_bores();
    let _front = add_second_front_view(&mut f);
    let sheet = f.session.drawing("test").unwrap().sheets[0].id;
    let drawing = f.session.drawing("test").unwrap().id;
    let bore1 = f.bore1.clone();
    let bore2 = f.bore2.clone();
    let authored = set_authored_text_at_paper(&mut f.session, &bore1, [95.0, 55.0], [205.0, 150.0]);
    assert_ok(&inspect(&mut f.session, &bore1), 10.0);
    assert_ok(&inspect(&mut f.session, &bore2), 10.0);

    // A real model command updates the measured Ø10 to Ø12, and an actual
    // dimension inspect stores the last successfully measured value in-session.
    f.change_bore1_to_12();
    assert_ok(&inspect(&mut f.session, &bore1), 12.0);
    assert_text_relative(&mut f.session, &bore1, authored);

    // A real deletion makes the reference lost; check it before file saving.
    f.close();
    run(&mut f.session, "timeline.delete", json!({"feature": "Bore1"}));
    f.open();
    let lost_before_save = inspect(&mut f.session, &bore1);
    assert_lost_with_session_last_good(&lost_before_save, &bore1, 12.0);
    assert_ok(&inspect(&mut f.session, &bore2), 10.0);
    let old_scene = public_scene_matches_engine_provider(&mut f.session, sheet, 2);
    let old_status = lost_before_save["status"].clone();
    assert_eq!(old_status, json!("lost"));

    // Actually write a file through SolidCraft's public file.save_as command;
    // a different brand-new Session must open the bytes from disk via doc.open.
    let path_dir = std::env::temp_dir().join(format!(
        "solidcraft-w23a-save-reopen-{}-{format_name}", std::process::id()
    ));
    std::fs::create_dir_all(&path_dir).expect("create isolated W23a test dir");
    let path = path_dir.join(format!("workshop-plate.{format_name}"));
    run(&mut f.session, "file.save_as", json!({"path": path}));
    assert!(path.exists(), "real design file missing after file.save_as: {path:?}");
    let mut reopened = Session::default();
    run(&mut reopened, "doc.open", json!({"path": path}));
    run(&mut reopened, "drawing.open", json!({"drawing": ids::encode(drawing)}));

    // The lost reference, original dimension ID, kept last measurement, and
    // manually authored label position must all survive the full file roundtrip.
    let persisted = inspect(&mut reopened, &bore1);
    assert_lost_with_session_last_good(&persisted, &bore1, 12.0);
    assert_eq!(persisted["record"]["last_good"], json!(12.0),
        "latest W19d file format failed to persist last_good: {persisted}");
    assert!(persisted.get("text").is_none(), "a lost dimension has no live text: {persisted}");
    assert_text_relative(&mut reopened, &bore1, authored);
    let surviving = inspect(&mut reopened, &bore2);
    assert_eq!(surviving["dimension"], bore2, "Bore2 ID changed across file save/reopen");
    assert_ok(&surviving, 10.0);
    let new_scene = public_scene_matches_engine_provider(&mut reopened, sheet, 2);
    assert_eq!(old_scene, new_scene, "2D view geometry changed across real file save/reopen");
    assert!(!reopened.is_dirty(), "read-only inspect must not dirty the re-opened design");
    std::fs::remove_dir_all(&path_dir).expect("clean real W23a test file");
    println!("PASS W23a E2E04 file subset: real {format_name} file save/reopen keeps Bore1 lost and last_good=12, Bore2 Ø10, dimension IDs, authored positions and two 2D scenes");
}

#[test]
fn w23a_official_e2e04_real_native_file_save_reopen_keeps_last_good_and_reference() {
    w23a_save_reopen_file(solidcraft_io::DESIGN_EXT);
}

#[test]
fn w23a_official_e2e04_real_json_file_save_reopen_keeps_last_good_and_reference() {
    w23a_save_reopen_file("json");
}
