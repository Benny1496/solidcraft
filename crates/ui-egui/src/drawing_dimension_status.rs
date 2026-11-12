//! Read-only M1-A projection of drawing.status and drawing.dimension.inspect.
//! No drawing command, update, repair or persistence mutation is performed here.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewItem {
    pub id: String,
    pub sheet: String,
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimensionItem {
    pub id: String,
    pub view: Option<String>,
    pub sheet: Option<String>,
    pub status: String,
    pub why: Option<String>,
    pub out_of_date: bool,
    pub inspect_error: Option<String>,
}

impl DimensionItem {
    pub fn needs_attention(&self) -> bool {
        self.out_of_date || !matches!(self.status.as_str(), "ok" | "overridden") || self.inspect_error.is_some()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DrawingAttention {
    pub views: Vec<ViewItem>,
    pub dimensions: Vec<DimensionItem>,
    pub waiting_views: usize,
    pub held: bool,
    pub warnings: Vec<String>,
}

impl DrawingAttention {
    /// The count is by unique dimension ID, not by individual flags.
    pub fn dimensions_needing_attention(&self) -> usize {
        self.dimensions.iter().filter(|d| d.needs_attention()).count()
    }
    /// An ID is returned only when inspect is needed to map it to a view/sheet.
    pub fn dimensions_to_inspect(&self) -> Vec<&str> {
        self.dimensions.iter().filter(|d| d.view.is_none()).map(|d| d.id.as_str()).collect()
    }
    /// Attach a previously fetched inspect result. Unknown/missing views remain unknown.
    pub fn with_inspections(mut self, inspections: &BTreeMap<String, Result<Value, String>>) -> Self {
        let sheets: BTreeMap<_, _> = self.views.iter().map(|v| (v.id.as_str(), v.sheet.as_str())).collect();
        for dim in &mut self.dimensions {
            match inspections.get(&dim.id) {
                Some(Ok(value)) => {
                    let view = value.get("view").and_then(Value::as_str).map(str::to_owned);
                    dim.sheet = view.as_deref().and_then(|id| sheets.get(id).copied()).map(str::to_owned);
                    dim.view = view;
                    dim.inspect_error = None;
                }
                Some(Err(error)) => dim.inspect_error = Some(error.clone()),
                None => {}
            }
        }
        self
    }
}

/// Projects one engine response; never guesses a dimension's sheet from its order.
/// Malformed entries are ignored rather than assigned fabricated identifiers.
pub fn from_status(status: &Value) -> DrawingAttention {
    let views = status
        .get("views")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| {
            Some(ViewItem {
                id: v.get("view")?.as_str()?.to_owned(),
                sheet: v.get("sheet")?.as_str()?.to_owned(),
                status: v.get("status")?.as_str()?.to_owned(),
                reason: v.get("reason").and_then(Value::as_str).map(str::to_owned),
            })
        })
        .collect();
    let mut seen = BTreeSet::new();
    let dimensions = status
        .get("dimensions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|d| {
            let id = d.get("dimension")?.as_str()?.to_owned();
            if !seen.insert(id.clone()) {
                return None;
            }
            Some(DimensionItem {
                id,
                view: None,
                sheet: None,
                status: d.get("status")?.as_str()?.to_owned(),
                why: d.get("why").and_then(Value::as_str).map(str::to_owned),
                out_of_date: d.get("out_of_date").and_then(Value::as_bool).unwrap_or(false),
                inspect_error: None,
            })
        })
        .collect();
    DrawingAttention {
        views,
        dimensions,
        waiting_views: status.get("waiting").and_then(Value::as_u64).and_then(|n| usize::try_from(n).ok()).unwrap_or(0),
        held: status.get("held").and_then(Value::as_bool).unwrap_or(false),
        warnings: status.get("param_warnings").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn lost_and_stale_count_once_and_map_by_inspected_view() {
        let status = json!({"views":[{"view":"v1","sheet":"s1","status":"current"},{"view":"v2","sheet":"s2","status":"current"}],"dimensions":[{"dimension":"d1","status":"lost","why":"missing edge","out_of_date":true},{"dimension":"d1","status":"lost","out_of_date":true},{"dimension":"d2","status":"ok"}],"waiting":0});
        let a = from_status(&status);
        assert_eq!(a.dimensions_needing_attention(), 1);
        assert_eq!(a.dimensions.len(), 2);
        let map = BTreeMap::from([("d1".into(), Ok(json!({"view":"v2"}))), ("d2".into(), Ok(json!({"view":"v1"})))]);
        let a = a.with_inspections(&map);
        assert_eq!(a.dimensions[0].sheet.as_deref(), Some("s2"));
        assert_eq!(a.dimensions[1].sheet.as_deref(), Some("s1"));
    }

    #[test]
    fn held_without_waiting_and_drawn_reason_are_preserved() {
        let a = from_status(
            &json!({"views":[{"view":"v","sheet":"s","status":"current","reason":"study no longer exists"}],"dimensions":[{"dimension":"d","status":"ok","out_of_date":true}],"waiting":0,"held":true}),
        );
        assert!(a.held);
        assert_eq!(a.waiting_views, 0);
        assert_eq!(a.views[0].reason.as_deref(), Some("study no longer exists"));
        assert_eq!(a.dimensions_needing_attention(), 1);
    }

    #[test]
    fn inspect_failure_and_unknown_view_never_guess_sheet() {
        let a = from_status(&json!({"views":[{"view":"v","sheet":"s","status":"current"}],"dimensions":[{"dimension":"d","status":"ambiguous"}]}));
        let a = a.with_inspections(&BTreeMap::from([("d".into(), Err("inspect unavailable".into()))]));
        assert_eq!(a.dimensions[0].sheet, None);
        assert!(a.dimensions[0].needs_attention());
        let a = a.with_inspections(&BTreeMap::from([("d".into(), Ok(json!({"view":"gone"})))]));
        assert_eq!(a.dimensions[0].view.as_deref(), Some("gone"));
        assert_eq!(a.dimensions[0].sheet, None);
    }
}

#[cfg(test)]
mod w23a_engine_integration_tests {
    use super::*;
    use serde_json::json;
    use solidcore_record::Vec3;
    use solidcraft_engine::Session;
    use solidcraft_geom::Vec3Geom;

    fn command(session: &mut Session, name: &str, params: Value) -> Value {
        session.execute(name, &params).unwrap_or_else(|error| panic!("{name}: {error}"))
    }

    /// Exercise the real W23a model, engine status and the actual UI projection.
    #[test]
    fn w23a_deleted_bore_is_reported_by_real_engine_and_ui_adapter() {
        let script: Value = serde_json::from_str(include_str!("../../../examples/acceptance/drawing_m2.json")).unwrap();
        let mut session = Session::default();
        let replies = session.run_script(&script).expect("W23a script must execute");
        let view = replies.last().expect("W23a creates a view")["view"].clone();
        let centre = Vec3::from([20.0, 20.0, 20.0]);
        let mesh = session.model.state().body("Plate").expect("Plate body").mesh();
        let index = mesh.edges.iter().position(|edge| {
            !edge.is_empty() && edge.iter().all(|point| (point.dist(centre) - 5.0).abs() < 1e-6)
        }).expect("Bore1 circular edge");
        let dimension = command(&mut session, "drawing.dimension.add", json!({
            "view": view,
            "kind": {"radial": {"edge": {"body": "Plate", "index": index}}},
            "text": {"at": [12.0, 24.0]}
        }))["dimension"].clone();
        assert_eq!(command(&mut session, "drawing.dimension.inspect", json!({"dimension": dimension}))["status"], "ok");

        let drawing = command(&mut session, "drawing.close", json!({}))["closed"].clone();
        assert!(drawing.is_string(), "drawing.close must return encoded drawing ID");
        command(&mut session, "timeline.delete", json!({"feature": "Bore1"}));
        command(&mut session, "drawing.open", json!({"drawing": drawing}));
        let engine_status = command(&mut session, "drawing.status", json!({}));
        let attention = from_status(&engine_status);
        let id = dimension.as_str().expect("dimension ID");
        let lost = attention.dimensions.iter().find(|item| item.id == id)
            .unwrap_or_else(|| panic!("W23a dimension absent from drawing.status: {engine_status}"));
        assert_eq!(lost.status, "lost", "{engine_status}");
        assert!(lost.needs_attention());
        assert!(attention.dimensions_needing_attention() >= 1);
        let inspected = command(&mut session, "drawing.dimension.inspect", json!({"dimension": dimension}));
        assert_eq!(inspected["status"], "lost");
        let enriched = attention.with_inspections(&BTreeMap::from([(id.to_owned(), Ok(inspected))]));
        assert!(enriched.dimensions.iter().find(|item| item.id == id).unwrap().view.is_some());
    }
}

#[cfg(test)]
mod stage3_issue616_fixture_gate {
    use serde_json::Value;

    #[test]
    fn official_lid_ribs_document_has_expected_features_and_rib_references() {
        let source: Value = serde_json::from_str(include_str!("../../../tests/fixtures/issue_616_lid_ribs_stage3.solidcraft"))
            .expect("official issue-616 .solidcraft JSON document");
        assert_eq!(source["format"], "solidcraft/15");
        assert_eq!(source["name"], "issue-616-lid-ribs");
        let features = source["features"].as_array().expect("document features");
        assert_eq!(features.len(), 11, "must retain the full original lid model");
        assert_eq!(features[0]["body_names"][0], "Lid");
        for name in ["Rib1", "Rib2", "Rib3", "Rib4"] {
            let rib = features.iter().find(|f| f["name"] == name)
                .unwrap_or_else(|| panic!("missing original feature {name}"));
            assert_eq!(rib["type"], "box");
            assert_eq!(rib["operation"], "join");
            assert_eq!(rib["suppressed"], false);
        }
    }
}

#[cfg(test)]
mod stage3_issue616_engine_gate {
    use serde_json::json;
    use solidcraft_engine::{doc::Document, Session};

    #[test]
    fn stage3_real_engine_opens_lid_ribs_and_creates_drawing_view() {
        let source = include_str!("../../../tests/fixtures/issue_616_lid_ribs_stage3.solidcraft");
        let doc = Document::from_json(source).expect("parse official issue-616 design");
        let mut session = Session::new(doc);
        let model_state = session.model.state();
        let lid = model_state.body("Lid").expect("real engine must evaluate Lid");
        assert!(!lid.mesh().positions.is_empty(), "lid must have real mesh geometry");
        session.execute("drawing.create", &json!({})).expect("create drawing for original lid");
        let view = session.execute("drawing.view.base", &json!({
            "sheet": "Sheet 1", "view": "top", "scale": "1:2", "position_mm": [150, 180]
        })).expect("generate drawing view from real lid geometry");
        assert!(view["view"].is_string(), "drawing view id missing: {view}");
        let status = session.execute("drawing.status", &json!({})).expect("inspect drawing status");
        assert!(status["views"].as_array().is_some(), "drawing status views missing: {status}");
    }
}

#[cfg(test)]
mod stage4_selene_native_engine_gate {
    use super::*;
    use serde_json::json;
    use solidcraft_engine::{io, Session};
    use std::time::Instant;

    const SELENE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/render/selene/models/SELENE-functional.solidcraft"
    );

    fn command(session: &mut Session, name: &str, arguments: serde_json::Value) -> serde_json::Value {
        session.execute(name, &arguments).unwrap_or_else(|error| panic!("{name}: {error}"))
    }

    /// Load and evaluate the actual original 439,883-byte binary SELENE model, then
    /// create and inspect a native Drawing view. No mock or reduced geometry.
    #[test]
    fn stage4_full_selene_native_model_loads_and_produces_drawing_status() {
        let started = Instant::now();
        let bytes = std::fs::read(SELENE).expect("read official functional SELENE artifact");
        assert_eq!(bytes.len(), 439_883, "unexpected SELENE model size");
        assert!(io::is_design_container(&bytes), "SELENE must be a native binary container");
        println!("SELENE_STAGE4_PHASE=verified_native_container bytes={} elapsed_ms={}", bytes.len(), started.elapsed().as_millis());

        // The desktop / CLI loading path preserves any saved evaluation cache and
        // resolves component links relative to the actual model location.
        let (document, saved) = io::read_design_file(SELENE, None::<fn(io::DesignPreview)>)
            .expect("official native SELENE format must be readable by current engine");
        let features = document.features.len();
        println!("SELENE_STAGE4_PHASE=parsed features={} saved_model={} elapsed_ms={}", features, saved.is_some(), started.elapsed().as_millis());
        assert_eq!(features, 5_484, "SELENE fixture no longer matches the documented master");

        let mut session = Session::opened_at(document, saved.as_ref(), Some(SELENE));
        let body_count = session.model.state().bodies.len();
        println!("SELENE_STAGE4_PHASE=evaluated bodies={} last_recomputed={} elapsed_ms={}", body_count, session.model.last_recomputed, started.elapsed().as_millis());
        assert_eq!(body_count, 2_116, "SELENE geometry/body inventory differs from the published master");

        // A real Drawing view is a stronger gate than merely parsing the ZIP or
        // listing the document's features, and exercises the status adapter.
        let drawing = command(&mut session, "drawing.create", json!({"name": "SELENE Stage 4 gate"}));
        assert!(drawing["sheets"].as_array().is_some(), "drawing.create must return a native drawing: {drawing}");
        println!("SELENE_STAGE4_PHASE=drawing_created elapsed_ms={}", started.elapsed().as_millis());
        let view = command(&mut session, "drawing.view.base", json!({
            "sheet": "Sheet 1", "view": "front", "scale": "1:20", "position_mm": [295, 205]
        }));
        assert!(view["view"].is_string(), "real SELENE drawing view ID missing: {view}");
        println!("SELENE_STAGE4_PHASE=drawing_view_created elapsed_ms={}", started.elapsed().as_millis());

        let status = command(&mut session, "drawing.status", json!({}));
        let views = status["views"].as_array().expect("drawing.status must return actual views");
        assert!(!views.is_empty(), "SELENE view absent from drawing.status: {status}");
        let attention = from_status(&status);
        assert!(!attention.views.is_empty(), "read-only Drawing UI adapter lost the real SELENE view");
        println!(
            "T128_SELENE_ENGINE_GATE_PASS features={} bodies={} drawing_views={} ui_views={} elapsed_ms={}",
            features, body_count, views.len(), attention.views.len(), started.elapsed().as_millis()
        );
    }
}

#[cfg(test)]
mod stage4_selene_multisheet_native_persistence {
    use super::*;
    use serde_json::{json, Value};
    use solidcraft_engine::{io, Session};
    use std::time::Instant;

    const SELENE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/render/selene/models/SELENE-functional.solidcraft");
    const TARGET: &str = "Pi5 BCM2712 substrate";
    const NEIGHBOR: &str = "Pi5 BCM2712 heat spreader";

    fn run(s: &mut Session, id: &str, args: Value) -> Value {
        s.execute(id, &args).unwrap_or_else(|e| panic!("{id}: {e}"))
    }

    /// Mesh edge selection is based on the geometry in the evaluated production
    /// model, not on an arbitrary index or a hand-built test substitute.
    fn top_x_edge(s: &Session, body: &str, expected: f64) -> usize {
        let state = s.model.state();
        let mesh = state.body(body).unwrap_or_else(|| panic!("real SELENE body absent: {body}")).mesh();
        let z = mesh.edges.iter().flat_map(|edge| edge.iter()).map(|p| p.z).fold(f64::NEG_INFINITY, f64::max);
        mesh.edges.iter().position(|line| {
            if line.len() < 2 { return false; }
            let first = &line[0];
            let last = line.last().unwrap();
            let dx = (first.x - last.x).abs();
            dx > expected - 0.02 && dx < expected + 0.02
                && (first.y - last.y).abs() < 1e-4
                && line.iter().all(|p| (p.z - z).abs() < 1e-4)
        }).unwrap_or_else(|| panic!("missing {expected}mm top edge on real SELENE body {body}"))
    }

    fn add_dimension(s: &mut Session, view: &Value, path: &[u64], body: &str, expected: f64) -> Value {
        let index = top_x_edge(s, body, expected);
        let read = run(s, "drawing.dimension.add", json!({
            "view": view,
            "kind": {"length": {"edge": {"body": body, "index": index, "path": path}}},
            "text": {"at": [17.0, 26.0]}
        }));
        assert_eq!(read["status"], "ok", "first real dimension must be valid: {read}");
        assert!((read["value"].as_f64().expect("real measured length") - expected).abs() < 0.02,
            "unexpected native SELENE geometry on {body}: {read}");
        assert!(read["dimension"].is_string(), "persistent dimension ID missing: {read}");
        read["dimension"].clone()
    }

    #[test]
    fn stage4_selene_multisheet_native_save_reopen_keeps_valid_and_suppressed_dimensions() {
        let started = Instant::now();
        let (doc, model) = io::read_design_file(SELENE, None::<fn(io::DesignPreview)>)
            .expect("official functional SELENE native design");
        assert_eq!(doc.features.len(), 5_484);
        let component = doc.components.iter().find(|x| x.name == "Raspberry Pi 5 componentry")
            .expect("the original Pi5 native component").id;
        let occurrence = doc.occurrences.iter().find(|x| x.component == component && x.parent == 0)
            .expect("Pi5 occurrence under root").id;
        let path = vec![occurrence];
        let mut session = Session::opened_at(doc, model.as_ref(), Some(SELENE));
        assert_eq!(session.model.state().bodies.len(), 2_116);
        println!("T131_PHASE=loaded_original elapsed_ms={}", started.elapsed().as_millis());

        // Two different native SELENE bodies, dimensioned on two persisted sheets.
        run(&mut session, "drawing.create", json!({"name": "SELENE multi-sheet persistence regression"}));
        let v1 = run(&mut session, "drawing.view.base", json!({
            "sheet": "Sheet 1", "view": "top", "scale": "1:2", "position_mm": [205, 170],
            "source": {"occurrences": [path.clone()]}
        }))["view"].clone();
        let primary = add_dimension(&mut session, &v1, &path, TARGET, 15.5);
        run(&mut session, "drawing.sheet.add", json!({"name": "Healthy Pi5"}));
        let v2 = run(&mut session, "drawing.view.base", json!({
            "sheet": "Healthy Pi5", "view": "top", "scale": "1:2", "position_mm": [205, 170],
            "source": {"occurrences": [path.clone()]}
        }))["view"].clone();
        let healthy = add_dimension(&mut session, &v2, &path, NEIGHBOR, 13.6);
        let primary_ref = run(&mut session, "drawing.dimension.inspect", json!({"dimension": primary.clone()}))["record"]["kind"].clone();
        let healthy_ref = run(&mut session, "drawing.dimension.inspect", json!({"dimension": healthy.clone()}))["record"]["kind"].clone();
        let draw_id = run(&mut session, "drawing.close", json!({}))["closed"].clone();

        // Suppress only the first body's source feature. A second dimension on a
        // different sheet must remain measurable despite this intentionally bad reference.
        run(&mut session, "timeline.suppress", json!({"feature": TARGET, "suppressed": true}));
        run(&mut session, "drawing.open", json!({"drawing": draw_id.clone()}));
        let bad_before = run(&mut session, "drawing.dimension.inspect", json!({"dimension": primary.clone()}));
        let good_before = run(&mut session, "drawing.dimension.inspect", json!({"dimension": healthy.clone()}));
        assert_eq!(bad_before["status"], "suppressed", "suppression not detected: {bad_before}");
        assert_eq!(good_before["status"], "ok", "unaffected neighbor must stay valid: {good_before}");
        assert!((bad_before["last_good"].as_f64().expect("previously valid measurement") - 15.5).abs() < 0.02);
        assert!((good_before["value"].as_f64().unwrap() - 13.6).abs() < 0.02);
        let status_before = from_status(&run(&mut session, "drawing.status", json!({})));
        assert_eq!(status_before.dimensions.len(), 2, "both real sheet dimensions must be listed");
        let bad_item = status_before.dimensions.iter().find(|d| d.id == primary.as_str().unwrap()).unwrap();
        let good_item = status_before.dimensions.iter().find(|d| d.id == healthy.as_str().unwrap()).unwrap();
        assert_eq!(bad_item.status, "suppressed");
        assert_eq!(good_item.status, "ok");
        assert!(bad_item.sheet.is_some() && good_item.sheet.is_some() && bad_item.sheet != good_item.sheet,
            "two dimension IDs should remain on distinct sheets");

        // Serialize the current session's *authoritative design* as a real native
        // .sc container. Session::design() explicitly materializes last_good values
        // that have been read in this session. Use a scratch file, never the master.
        let saved_bytes = io::write_design_container(&session.design())
            .expect("save altered SELENE drawing as native SolidCraft container");
        assert!(io::is_design_container(&saved_bytes), "save must produce a real binary design");
        let saved_path = std::env::temp_dir().join(format!("solidcraft-t131-selene-{}.sc", std::process::id()));
        std::fs::write(&saved_path, &saved_bytes).expect("write CI-only native SELENE copy");
        println!("T131_PHASE=native_saved bytes={} elapsed_ms={}", saved_bytes.len(), started.elapsed().as_millis());

        let file = saved_path.to_str().expect("UTF-8 temp directory");
        let (loaded, cache) = io::read_design_file(file, None::<fn(io::DesignPreview)>)
            .expect("reopen temporary SELENE-native design from disk");
        assert_eq!(loaded.features.len(), 5_484, "saving must retain the full SELENE feature timeline");
        assert_eq!(loaded.drawings.len(), 1, "the saved drawing must survive");
        assert_eq!(loaded.drawings[0].sheets.len(), 2, "both sheets must survive native persistence");
        let mut reopened = Session::opened_at(loaded, cache.as_ref(), Some(file));
        assert!(reopened.model.state().body(NEIGHBOR).is_some(), "healthy neighbor must still be in reloaded model");
        run(&mut reopened, "drawing.open", json!({"drawing": draw_id.clone()}));
        let bad_after = run(&mut reopened, "drawing.dimension.inspect", json!({"dimension": primary.clone()}));
        let good_after = run(&mut reopened, "drawing.dimension.inspect", json!({"dimension": healthy.clone()}));
        assert_eq!(bad_after["status"], "suppressed", "reopen must retain warning: {bad_after}");
        assert!(bad_after.get("value").is_none(), "suppressed measure cannot invent a current value: {bad_after}");
        assert!((bad_after["last_good"].as_f64().expect("saved last-good must survive") - 15.5).abs() < 0.02);
        assert_eq!(good_after["status"], "ok", "neighbor still valid after reopening: {good_after}");
        assert!((good_after["value"].as_f64().unwrap() - 13.6).abs() < 0.02);
        assert_eq!(bad_after["record"]["kind"], primary_ref, "original Pi5 reference must persist unchanged");
        assert_eq!(good_after["record"]["kind"], healthy_ref, "healthy Pi5 reference must persist unchanged");
        let status_after = from_status(&run(&mut reopened, "drawing.status", json!({})));
        assert_eq!(status_after.dimensions.len(), 2);
        let broken = status_after.dimensions.iter().find(|x| x.id == primary.as_str().unwrap()).unwrap();
        let intact = status_after.dimensions.iter().find(|x| x.id == healthy.as_str().unwrap()).unwrap();
        assert_eq!(broken.status, "suppressed");
        assert_eq!(intact.status, "ok");
        assert_ne!(broken.sheet, intact.sheet, "sheet mapping must persist across file save/open");
        println!("T131_PHASE=reopened_native verified_two_sheets=true last_good=15.5 healthy=13.6 elapsed_ms={}", started.elapsed().as_millis());

        // Explicit repair after reopening, not Undo: undo history is intentionally
        // per-session and must not be assumed to persist in the .sc design file.
        run(&mut reopened, "drawing.close", json!({}));
        run(&mut reopened, "timeline.suppress", json!({"feature": TARGET, "suppressed": false}));
        run(&mut reopened, "drawing.open", json!({"drawing": draw_id}));
        let repaired = run(&mut reopened, "drawing.dimension.inspect", json!({"dimension": primary.clone()}));
        let untouched = run(&mut reopened, "drawing.dimension.inspect", json!({"dimension": healthy.clone()}));
        assert_eq!(repaired["status"], "ok", "explicit unsuppress should restore the named Pi5 edge: {repaired}");
        assert_eq!(repaired["record"]["kind"], primary_ref);
        assert!((repaired["value"].as_f64().unwrap() - 15.5).abs() < 0.02);
        assert_eq!(untouched["status"], "ok");
        assert!((untouched["value"].as_f64().unwrap() - 13.6).abs() < 0.02);

        std::fs::remove_file(&saved_path).expect("remove CI-only native copy");
        println!("T131_SELENE_PERSISTENCE_PASS two_sheets=2 native_reopen=true suppressed_last_good=15.5 healthy=13.6 explicit_repair=ok elapsed_ms={}", started.elapsed().as_millis());
    }
}
