//! Experimental, read-only, model-linked drawing preview for a rectangular XY plate.
//! This is an explicitly bounded prototype, NOT a general B-rep projection engine.
//! In particular, the rectangle is a bounding-box envelope and cylindrical faces
//! are provisional; saved drawing sheets and a user interface are future work.

use serde_json::{Value, json};

use super::CommandSpec;
use crate::params::{bad, str_};
use crate::{Result, Session};

const CMD: &str = "drawing.preview";

pub static COMMANDS: &[CommandSpec] = &[CommandSpec::new(CMD, "Preview Drawing", preview)
    .noundo()
    .at("INSPECT", "DRAWING")
    .icon("inspect")
    .params("body: name; previous?: a prior drawing from this command; accept?: bool. Returns provisional top-view rectangle and vertical cylinders. When geometry changes, returns stale/confirmation_required without modifying previous; accept:true returns an updated drawing with preserved label positions. Also reports read-only provisional cylinder diameters and XY offsets from bounding edges. No persistent hole identifiers or native drawing sheet.")];

fn coords(value: &Value, n: usize) -> Option<Vec<f64>> {
    let arr = value.as_array().filter(|a| a.len() == n)?;
    arr.iter().map(|v| v.as_f64().filter(|x| x.is_finite())).collect()
}

fn mm(n: f64) -> f64 {
    (n * 1_000.0).round() / 1_000.0
}

/// Parse the measured source into numeric values. Browser JSON.stringify turns
/// Rust's 150.0 into 150; comparing serde_json::Value would then report a
/// false difference because JSON integer and float number variants differ.
fn source_numbers(v: &Value) -> Option<(Vec<f64>, Vec<f64>, Vec<[f64; 3]>)> {
    let lower = coords(v.get("lower_xy_mm")?, 2)?;
    let size = coords(v.get("size_mm")?, 3)?;
    let circles = v.get("cylinders_xy_diameter_mm")?.as_array()?;
    if circles.len() > 64 || size.iter().any(|x| *x <= 0.0) {
        return None;
    }
    let holes = circles
        .iter()
        .map(|circle| {
            let v = coords(circle, 3)?;
            (v[2] > 0.0).then_some([v[0], v[1], v[2]])
        })
        .collect::<Option<Vec<_>>>()?;
    Some((lower, size, holes))
}

fn source_matches(old: &Value, current: &Value) -> bool {
    match (source_numbers(old), source_numbers(current)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Read-only measurements derived from actual B-rep cylinder surfaces and
/// the body's XY bounding rectangle. Entries have no persistent hole id:
/// without certified feature provenance reordering must never move a callout.
fn provisional_cylinder_dimensions(holes: &[[f64; 3]], min: &[f64]) -> Vec<Value> {
    holes.iter().map(|h| json!({
        "kind": "vertical_cylinder",
        "reference_status": "unverified",
        "center_mm": [h[0], h[1]],
        "diameter_mm": h[2],
        "x_from_left_mm": mm(h[0] - min[0]),
        "y_from_bottom_mm": mm(h[1] - min[1])
    })).collect()
}

// A limited, geometrically scaled SVG view for the same verified XY envelope.
// Width/height use physical millimetres. Geometry is not a true occlusion/HLR
// projection; the drawing contract states the supported subset explicitly.
fn svg_preview(min: &[f64], max: &[f64], cylinders: &[[f64; 3]], labels: &Value) -> String {
    let w = mm(max[0] - min[0]);
    let h = mm(max[1] - min[1]);
    let view_w = w + 48.0;
    let view_h = h + 48.0;
    let start_x = 20.0;
    let start_y = 20.0;
    let map = |pt: &[f64]| (start_x + pt[0] - min[0], start_y + max[1] - pt[1]);
    let length = labels.get("overall_x").and_then(|v| coords(v, 2)).unwrap_or_else(|| vec![mm((min[0] + max[0]) / 2.0), mm(min[1] - 12.0)]);
    let width = labels.get("overall_y").and_then(|v| coords(v, 2)).unwrap_or_else(|| vec![mm(min[0] - 12.0), mm((min[1] + max[1]) / 2.0)]);
    let (lx, ly) = map(&length);
    let (wx, wy) = map(&width);
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {view_w} {view_h}\" width=\"{view_w}mm\" height=\"{view_h}mm\"><title>SolidCraft experimental XY top-view drawing</title><g stroke=\"#12283c\" stroke-width=\"0.3\" fill=\"none\"><rect x=\"{start_x}\" y=\"{start_y}\" width=\"{w}\" height=\"{h}\"/>"
    );
    for c in cylinders {
        let (cx, cy) = map(&c[0..2]);
        svg.push_str(&format!("<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{}\"/>", c[2] / 2.0));
    }
    let bottom = start_y + h;
    let right = start_x + w;
    let dim_y = ly + 2.0;
    let dim_x = wx - 2.0;
    svg.push_str(&format!(
        "<path stroke=\"#65748b\" stroke-width=\"0.15\" d=\"M {start_x} {bottom} V {dim_y} M {right} {bottom} V {dim_y} M {start_x} {start_y} H {dim_x} M {start_x} {bottom} H {dim_x}\"/>"
    ));
    svg.push_str("</g>");
    svg.push_str(&format!(
        "<g fill=\"#12283c\" font-family=\"sans-serif\" font-size=\"4\" text-anchor=\"middle\"><text x=\"{lx}\" y=\"{ly}\">{w:.3} mm</text><text x=\"{wx}\" y=\"{wy}\">{h:.3} mm</text></g></svg>"
    ));
    svg
}

fn label_positions(old: Option<&Value>, current: &Value) -> Result<Value> {
    let mut positions = current.clone();
    if let Some(old) = old {
        let labels = old.as_object().ok_or_else(|| bad(CMD, "previous label_positions must be an object"))?;
        if labels.len() > 128 {
            return Err(bad(CMD, "too many label positions"));
        }
        let out = positions.as_object_mut().ok_or_else(|| bad(CMD, "invalid generated label positions"))?;
        for (key, v) in labels {
            if out.contains_key(key) {
                let xy = coords(v, 2).filter(|a| a.iter().all(|x| x.abs() <= 1_000_000.0))
                    .ok_or_else(|| bad(CMD, "invalid label position"))?;
                out.insert(key.clone(), json!(xy));
            }
        }
    }
    Ok(positions)
}

/// Generate a transient XY top view from measured model geometry.
/// Refuse geometries outside the supported prototype rather than inventing
/// precise outlines or confidently reattaching a previously selected hole.
fn preview(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_(p, "body").filter(|n| !n.is_empty() && n.len() < 256)
        .ok_or_else(|| bad(CMD, "body name required"))?;
    let accept = match p.get("accept") {
        None => false,
        Some(Value::Bool(x)) => *x,
        _ => return Err(bad(CMD, "accept must be a boolean")),
    };
    let old = p.get("previous");
    if accept && old.is_none() {
        return Err(bad(CMD, "accept requires a previous drawing"));
    }

    let doc = s.execute("document.inspect", &json!({"measure": true}))?;
    let bodies = doc.get("bodies").and_then(Value::as_array)
        .ok_or_else(|| bad(CMD, "model has no bodies list"))?;
    let body = bodies.iter().find(|b| b.get("name").and_then(Value::as_str) == Some(name))
        .ok_or_else(|| bad(CMD, "body not found"))?;
    let bbox = body.get("bbox").ok_or_else(|| bad(CMD, "body has no bounding box"))?;
    let min = coords(&bbox["min"], 3).ok_or_else(|| bad(CMD, "invalid lower bound"))?;
    let max = coords(&bbox["max"], 3).ok_or_else(|| bad(CMD, "invalid upper bound"))?;
    let dimensions: Vec<f64> = (0..3).map(|i| mm(max[i] - min[i])).collect();
    if dimensions.iter().any(|d| !d.is_finite() || *d <= 0.0) {
        return Err(bad(CMD, "body must have three positive dimensions"));
    }

    let topology = s.execute("model.faces", &json!({"body": name}))?;
    let faces = topology.get("faces").and_then(Value::as_array)
        .ok_or_else(|| bad(CMD, "body has no inspectable faces"))?;
    if faces.len() > 2048 {
        return Err(bad(CMD, "too many faces for drawing preview"));
    }
    let mut holes: Vec<[f64; 3]> = Vec::new();
    for face in faces {
        let Some(cylinder) = face.get("cylinder") else { continue };
        let origin = coords(&cylinder["origin"], 3).ok_or_else(|| bad(CMD, "invalid cylinder origin"))?;
        let axis = coords(&cylinder["axis"], 3).ok_or_else(|| bad(CMD, "invalid cylinder axis"))?;
        let radius = cylinder["radius"].as_f64().filter(|r| r.is_finite() && *r > 0.0)
            .ok_or_else(|| bad(CMD, "invalid cylinder radius"))?;
        let length = axis.iter().map(|x| x * x).sum::<f64>().sqrt();
        if !length.is_finite() || length <= 0.0 {
            return Err(bad(CMD, "invalid cylinder axis length"));
        }
        if axis[2].abs() / length < 0.999 {
            return Err(bad(CMD, "only vertical cylinders are supported in this prototype"));
        }
        let x = mm(origin[0]);
        let y = mm(origin[1]);
        let diameter = mm(2.0 * radius);
        if x - radius < min[0] - 0.02 || x + radius > max[0] + 0.02
            || y - radius < min[1] - 0.02 || y + radius > max[1] + 0.02
        {
            return Err(bad(CMD, "cylinder is not contained by the XY envelope"));
        }
        if !holes.iter().any(|h| (h[0] - x).abs() < 0.01 && (h[1] - y).abs() < 0.01
            && (h[2] - diameter).abs() < 0.01)
        {
            if holes.len() >= 64 {
                return Err(bad(CMD, "more than 64 cylinders"));
            }
            holes.push([x, y, diameter]);
        }
    }
    holes.sort_by(|a, b| a[1].total_cmp(&b[1]).then(a[0].total_cmp(&b[0]))
        .then(a[2].total_cmp(&b[2])));

    // This is deliberately geometry-based rather than only the user's command
    // parameters: the measured 3D body is the authority for freshness.
    let source = json!({
        "lower_xy_mm": [mm(min[0]), mm(min[1])],
        "size_mm": dimensions,
        "cylinders_xy_diameter_mm": holes,
    });
    let mut default_labels = serde_json::Map::new();
    let cx = mm((min[0] + max[0]) / 2.0);
    let cy = mm((min[1] + max[1]) / 2.0);
    default_labels.insert("overall_x".into(), json!([cx, mm(min[1] - 12.0)]));
    default_labels.insert("overall_y".into(), json!([mm(min[0] - 12.0), cy]));
    // Bore positions are geometry-derived and may be reordered by an edit.
    // Do not store positional bore callout ids: that could silently link a
    // manually placed dimension to the wrong hole after a topology change.
    let current_labels = Value::Object(default_labels);

    let prev = match old {
        None => None,
        Some(v) => {
            if v.get("format").and_then(Value::as_str) != Some("solidcraft-drawing-preview/1")
                || v.get("body").and_then(Value::as_str) != Some(name)
                || !v.get("source").is_some_and(|source| source_numbers(source).is_some())
            {
                return Err(bad(CMD, "previous drawing has wrong format, body or source"));
            }
            Some(v)
        }
    };
    let layout = label_positions(prev.and_then(|d| d.get("label_positions")), &current_labels)?;
    let drawing = json!({
        "format": "solidcraft-drawing-preview/1",
        "body": name,
        "projection": "xy_top",
        "accuracy": "bbox_envelope_and_measured_cylinders_only",
        "source": source,
        "geometry": {
            "outline": { "type": "rectangle_envelope", "min_xy_mm": [mm(min[0]),mm(min[1])],
                         "max_xy_mm": [mm(max[0]),mm(max[1])] },
            "cylinders": holes.iter().map(|h| json!({"center_mm":[h[0],h[1]],"diameter_mm":h[2]})).collect::<Vec<_>>(),
            "cylinder_dimensions": provisional_cylinder_dimensions(&holes, &min),
        },
        "dimensions": [
            {"id":"overall_x","kind":"horizontal","value_mm":dimensions[0]},
            {"id":"overall_y","kind":"vertical","value_mm":dimensions[1]}
        ],
        "svg": svg_preview(&min, &max, &holes, &layout),
        "label_positions": layout,
        "warnings": if !holes.is_empty() {
            vec!["Measured cylinder diameters and XY offsets are provisional, not confirmed hole callouts. No persistent hole/feature reference exists; never reattach drawing dimensions by list order"]
        } else {
            Vec::new()
        }
    });

    if let Some(previous) = prev {
        let stale = !source_matches(&previous["source"], &drawing["source"]);
        if stale && !accept {
            return Ok(json!({
                "status":"stale",
                "requires_confirmation":true,
                "message":"The original 3D model has changed. Update this drawing?",
                "drawing":previous,
                "proposed":drawing
            }));
        }
        return Ok(json!({
            "status":if stale {"updated"} else {"current"},
            "requires_confirmation":false,
            "drawing":drawing
        }));
    }
    Ok(json!({"status":"created","requires_confirmation":false,"drawing":drawing}))
}

#[cfg(test)]
mod tests {
    use super::{preview, source_matches};
    use crate::Session;
    use serde_json::json;

    #[test]
    fn javascript_numeric_json_roundtrip_does_not_stale_a_drawing() {
        let original = json!({
            "lower_xy_mm": [0.0, 0.0],
            "size_mm": [150.0, 80.0, 10.0],
            "cylinders_xy_diameter_mm": [[15.0, 15.0, 8.0], [135.0, 15.0, 8.0]]
        });
        // Matches JSON.stringify in the browser: integral floats lose their
        // decimal dot and serde_json parses them as integer variants.
        let browser = json!({
            "lower_xy_mm": [0, 0],
            "size_mm": [150, 80, 10],
            "cylinders_xy_diameter_mm": [[15, 15, 8], [135, 15, 8]]
        });
        assert_ne!(original, browser);
        assert!(source_matches(&original, &browser));
        assert!(!source_matches(&original, &json!({
            "lower_xy_mm": [0, 0],
            "size_mm": [180, 80, 10],
            "cylinders_xy_diameter_mm": [[15, 15, 8], [165, 15, 8]]
        })));
    }

    #[test]
    fn drawing_stays_current_until_model_changes_then_requires_confirmation() {
        let mut session = Session::default();
        session.execute("solid.box", &json!({
            "length": 150, "width": 80, "height": 10,
            "body_name": "DrawingPlate", "name": "Stock"
        })).unwrap();
        let first = preview(&mut session, &json!({"body":"DrawingPlate"})).unwrap();
        assert_eq!(first["status"], "created");
        assert_eq!(first["drawing"]["dimensions"][0]["value_mm"], 150.0);
        let old = first["drawing"].clone();
        // Replace numeric JSON variants like the browser's JSON.stringify.
        let previous = json!({
            "format": old["format"],
            "body": old["body"],
            "source": {
                "lower_xy_mm": [0,0],
                "size_mm": [150,80,10],
                "cylinders_xy_diameter_mm": []
            },
            "label_positions": {"overall_x":[99,-22],"overall_y":[-23,40]}
        });
        let unchanged = preview(&mut session, &json!({
            "body":"DrawingPlate", "previous":previous
        })).unwrap();
        assert_eq!(unchanged["status"], "current");
        session.execute("timeline.edit", &json!({
            "feature":"Stock", "set":{"length":"180"}
        })).unwrap();
        let stale = preview(&mut session, &json!({
            "body":"DrawingPlate", "previous":previous
        })).unwrap();
        assert_eq!(stale["status"], "stale");
        assert_eq!(stale["requires_confirmation"], true);
        assert_eq!(stale["drawing"]["source"]["size_mm"][0], 150);
        assert_eq!(stale["proposed"]["dimensions"][0]["value_mm"], 180.0);
        let accepted = preview(&mut session, &json!({
            "body":"DrawingPlate","previous":previous,"accept":true
        })).unwrap();
        assert_eq!(accepted["status"], "updated");
        assert_eq!(accepted["drawing"]["dimensions"][0]["value_mm"], 180.0);
        // Coordinates are parsed as f64 when the previous user layout is
        // normalised. Compare the measured values, not the JSON int/float tags.
        let label = &accepted["drawing"]["label_positions"]["overall_x"];
        assert_eq!(label[0].as_f64(), Some(99.0));
        assert_eq!(label[1].as_f64(), Some(-22.0));
    }
    #[test]
    fn malformed_previous_drawing_and_out_of_bounds_labels_are_refused() {
        let mut session = Session::default();
        session.execute("solid.box", &json!({"length":100,"width":40,"height":8,
            "body_name":"SafePlate"})).unwrap();
        let previous = preview(&mut session, &json!({"body":"SafePlate"})).unwrap()["drawing"].clone();
        for params in [
            json!({"body":"SafePlate","accept":true}),
            json!({"body":"SafePlate","accept":"yes"}),
            json!({"body":"SafePlate","previous":{"format":"wrong","body":"SafePlate",
                "source":previous["source"]}}),
            json!({"body":"SafePlate","previous":{
                "format":"solidcraft-drawing-preview/1", "body":"Other",
                "source":previous["source"]}}),
            json!({"body":"SafePlate","previous":{
                "format":"solidcraft-drawing-preview/1", "body":"SafePlate",
                "source":{"lower_xy_mm":[0,0],"size_mm":[100,"invalid",8],
                          "cylinders_xy_diameter_mm":[]}}}),
            json!({"body":"SafePlate","previous":{
                "format":"solidcraft-drawing-preview/1","body":"SafePlate",
                "source":previous["source"],
                "label_positions":{"overall_x":[1e20,-12]}}})
        ] {
            assert!(preview(&mut session, &params).is_err(),
                "invalid snapshots must be refused: {params}");
        }
        let after = preview(&mut session, &json!({
            "body":"SafePlate","previous":previous
        })).unwrap();
        assert_eq!(after["status"], "current");
    }

    #[test]
    fn drawing_source_change_checks_geometry_not_label_coordinates() {
        let mut session = Session::default();
        session.execute("solid.box", &json!({"length":65,"width":25,"height":5,
            "body_name":"LayoutPlate"})).unwrap();
        let created = preview(&mut session, &json!({"body":"LayoutPlate"})).unwrap();
        let mut old = created["drawing"].clone();
        old["label_positions"]["overall_x"] = json!([49,-27]);
        old["label_positions"]["overall_y"] = json!([-18,12]);
        let result = preview(&mut session, &json!({
            "body":"LayoutPlate","previous":old
        })).unwrap();
        assert_eq!(result["status"], "current",
            "moving dimension text is not a change to the 3D body");
        assert_eq!(result["drawing"]["label_positions"]["overall_x"],json!([49.0,-27.0]));
        assert_eq!(result["drawing"]["label_positions"]["overall_y"],json!([-18.0,12.0]));
        assert_eq!(result["drawing"]["dimensions"][0]["value_mm"],65.0);
        assert_eq!(result["drawing"]["dimensions"][1]["value_mm"],25.0);
        assert!(result["drawing"]["svg"].as_str().unwrap().contains("65.000 mm"));
    }

    #[test]
    fn broken_numeric_geometry_signatures_never_report_current() {
        let baseline = json!({"lower_xy_mm":[0,0],"size_mm":[80,40,5],
            "cylinders_xy_diameter_mm":[[8,8,4],[72,32,4]]});
        let malformed = [
            json!({"lower_xy_mm":[0,0],"size_mm":[80,40,5]}),
            json!({"lower_xy_mm":[0,0],"size_mm":[0,40,5],
                "cylinders_xy_diameter_mm":[[8,8,4],[72,32,4]]}),
            json!({"lower_xy_mm":[0,0],"size_mm":[80,40,5],
                "cylinders_xy_diameter_mm":[[8,8,4],[72,32,0]]}),
            json!({"lower_xy_mm":[0,0],"size_mm":[80,40,5],
                "cylinders_xy_diameter_mm":[["x",8,4],[72,32,4]]})
        ];
        for bad in malformed {
            assert!(!source_matches(&baseline,&bad), "bad geometry source cannot be current: {bad}");
        }
        let moved_hole = json!({"lower_xy_mm":[0,0],"size_mm":[80,40,5],
            "cylinders_xy_diameter_mm":[[9,8,4],[72,32,4]]});
        assert!(!source_matches(&baseline,&moved_hole));
        let moved_origin = json!({"lower_xy_mm":[1,0],"size_mm":[80,40,5],
            "cylinders_xy_diameter_mm":[[8,8,4],[72,32,4]]});
        assert!(!source_matches(&baseline,&moved_origin));
    }

    #[test]
    fn saved_drawing_survives_solidcraft_file_roundtrip_and_detects_later_edit() {
        let path = std::env::temp_dir().join(format!(
            "solidcraft-drawing-snapshot-{}.solidcraft", std::process::id()
        ));
        let mut s = Session::default();
        s.execute("solid.box", &json!({"length":90,"width":45,"height":8,
            "body_name":"StoredPlate","name":"Stock"})).unwrap();
        let mut drawing = preview(&mut s, &json!({"body":"StoredPlate"})).unwrap()["drawing"].clone();
        drawing["label_positions"]["overall_x"] = json!([45,-21]);
        s.execute("document.metadata.set", &json!({
            "key":"org.solidcraft.experimental_drawing", "value":drawing
        })).unwrap();
        s.execute("file.save_as", &json!({"path":path,"format":"container"})).unwrap();
        let mut opened = Session::default();
        opened.execute("doc.open", &json!({"path":path})).unwrap();
        let preserved = opened.doc.metadata.get("org.solidcraft.experimental_drawing")
            .expect("the saved .solidcraft file must contain the drawing snapshot").clone();
        assert_eq!(preserved["dimensions"][0]["value_mm"], 90.0);
        assert_eq!(preserved["label_positions"]["overall_x"], json!([45,-21]));
        let still_current = preview(&mut opened, &json!({
            "body":"StoredPlate","previous":preserved
        })).unwrap();
        assert_eq!(still_current["status"], "current");
        opened.execute("timeline.edit", &json!({
            "feature":"Stock","set":{"length":"110"}
        })).unwrap();
        let stale = preview(&mut opened, &json!({
            "body":"StoredPlate","previous":preserved
        })).unwrap();
        assert_eq!(stale["status"], "stale");
        assert_eq!(stale["drawing"]["dimensions"][0]["value_mm"], 90.0);
        assert_eq!(stale["proposed"]["dimensions"][0]["value_mm"], 110.0);
        let _ = std::fs::remove_file(path);
    }

}
