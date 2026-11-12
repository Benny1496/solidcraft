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
mod stage4_selene_positive_component_dimension {

    use super::*;
    use serde_json::{json, Value};
    use solidcraft_engine::{io, Session};
    use std::time::Instant;

    const SELENE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/render/selene/models/SELENE-functional.solidcraft");
    const TARGET: &str = "Pi5 BCM2712 substrate";
    const COMPONENT: &str = "Raspberry Pi 5 componentry";

    fn run(s: &mut Session, id: &str, p: Value) -> Value {
        s.execute(id, &p).unwrap_or_else(|e| panic!("{id}: {e}"))
    }

    /// Real SELENE model, no synthesized placeholder geometry; a single native
    /// occurrence is drawn to keep assembly occlusion out of the measurement.
    fn open_selene() -> (Session, Value, Vec<u64>, Instant) {
        let start = Instant::now();
        let (doc, saved) = io::read_design_file(SELENE, None::<fn(io::DesignPreview)>)
            .expect("read original 439883-byte native SELENE file");
        assert_eq!(doc.features.len(), 5_484, "unmodified functional SELENE master");
        let component = doc.components.iter().find(|c| c.name == COMPONENT)
            .expect("Pi5 componentry in actual native SELENE model").id;
        let occurrence = doc.occurrences.iter().find(|o| o.component == component && o.parent == 0)
            .expect("placed Pi5 componentry native occurrence").id;
        let mut s = Session::opened_at(doc, saved.as_ref(), Some(SELENE));
        assert_eq!(s.model.state().bodies.len(), 2_116, "complete real SELENE assembly evaluated");
        assert!(s.model.state().body(TARGET).is_some(), "actual Pi5 BCM2712 substrate body required");
        println!("SELENE_DIM_PHASE=loaded body={} occurrence={} elapsed_ms={}", TARGET, occurrence, start.elapsed().as_millis());
        run(&mut s, "drawing.create", json!({"name": "SELENE Pi5 component dimension integration"}));
        let path = vec![occurrence];
        let view = run(&mut s, "drawing.view.base", json!({
            "sheet": "Sheet 1", "view": "top", "scale": "1:2", "position_mm": [205, 170],
            "source": {"occurrences": [path.clone()]}
        }))["view"].clone();
        assert!(view.is_string(), "native subassembly view missing");
        (s, view, path, start)
    }

    /// Real top perimeter edge, measured along X. The Pi5 substrate is a box
    /// 15.5 x 15.5 x 0.55 mm, documented by the official electronics builder.
    /// Find its live mesh index rather than assuming a topology ordering.
    fn upper_x_edge(s: &Session) -> usize {
        let state = s.model.state();
        let mesh = state.body(TARGET).expect("Pi5 substrate exists").mesh();
        let top_z = mesh.edges.iter().flat_map(|line| line.iter()).map(|p| p.z)
            .fold(f64::NEG_INFINITY, f64::max);
        mesh.edges.iter().position(|line| {
            if line.len() < 2 { return false; }
            let a = &line[0];
            let b = line.last().unwrap();
            let dx = (a.x - b.x).abs();
            let dy = (a.y - b.y).abs();
            dx > 15.48 && dx < 15.52 && dy < 1e-4
                && line.iter().all(|p| (p.z - top_z).abs() < 1e-4)
        }).expect("live unsplit 15.5-mm top edge on original Pi5 substrate")
    }

    fn dimension(s: &mut Session, view: &Value, path: &[u64]) -> Value {
        let index = upper_x_edge(s);
        let reply = run(s, "drawing.dimension.add", json!({
            "view": view,
            "kind": {"length": {"edge": {"body": TARGET, "index": index, "path": path}}},
            "text": {"at": [14.0, 22.0]}
        }));
        assert_eq!(reply["status"], "ok", "real Pi5 edge must be dimensionable: {reply}");
        let id = reply["dimension"].clone();
        assert!(id.is_string(), "persistent dimension ID missing: {reply}");
        let measured = reply["value"].as_f64().expect("real measured value");
        assert!((measured - 15.5).abs() < 0.02, "Pi5 substrate expected 15.5 mm: {reply}");
        id
    }

    #[test]
    fn stage4_selene_positive_component_dimension_updates_and_undo_redo() {
        let (mut s, view, path, start) = open_selene();
        let id = dimension(&mut s, &view, &path);
        let original = run(&mut s, "drawing.dimension.inspect", json!({"dimension": id.clone()}));
        let drawing = run(&mut s, "drawing.close", json!({}))["closed"].clone();
        let before = run(&mut s, "timeline.inspect", json!({"feature": TARGET}));
        run(&mut s, "timeline.edit", json!({"feature": TARGET, "set": {"length": "16.0"}}));
        let after_edit = run(&mut s, "timeline.inspect", json!({"feature": TARGET}));
        assert_ne!(after_edit["definition"]["length"], before["definition"]["length"], "native Pi5 substrate length must change");
        run(&mut s, "drawing.open", json!({"drawing": drawing.clone()}));
        let now = run(&mut s, "drawing.dimension.inspect", json!({"dimension": id.clone()}));
        assert_eq!(now["status"], "ok", "changed real component edge must remain named: {now}");
        assert_eq!(now["record"]["kind"], original["record"]["kind"], "persistent component edge reference changed");
        let value = now["value"].as_f64().expect("updated real Pi5 dimension");
        assert!((value - 16.0).abs() < 0.02, "measured dimension must follow actual geometry: {now}");
        let status = run(&mut s, "drawing.status", json!({}));
        let attention = from_status(&status);
        let measured = attention.dimensions.iter().find(|d| d.id == id.as_str().unwrap())
            .unwrap_or_else(|| panic!("changed dimension missing from real status: {status}"));
        assert_eq!(measured.status, "ok", "out of date does not mean lost");
        assert!(measured.out_of_date && attention.waiting_views > 0, "model change should flag stale drawing: {status}");

        run(&mut s, "drawing.close", json!({}));
        run(&mut s, "edit.undo", json!({}));
        run(&mut s, "drawing.open", json!({"drawing": drawing.clone()}));
        let undone = run(&mut s, "drawing.dimension.inspect", json!({"dimension": id.clone()}));
        assert_eq!(undone["status"], "ok");
        assert!((undone["value"].as_f64().unwrap() - 15.5).abs() < 0.02, "undo must restore old geometry: {undone}");
        run(&mut s, "drawing.close", json!({}));
        run(&mut s, "edit.redo", json!({}));
        run(&mut s, "drawing.open", json!({"drawing": drawing}));
        let redo = run(&mut s, "drawing.dimension.inspect", json!({"dimension": id}));
        assert_eq!(redo["status"], "ok");
        assert!((redo["value"].as_f64().unwrap() - 16.0).abs() < 0.02, "redo must restore new Pi5 dimension: {redo}");
        println!("T129_SELENE_POSITIVE_PASS component=Pi5_BCM2712 initial=15.5 edited=16 undo=15.5 redo=16 elapsed_ms={}", start.elapsed().as_millis());
    }
}
