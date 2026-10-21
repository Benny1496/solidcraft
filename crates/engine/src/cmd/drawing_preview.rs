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
    .params("body: name; previous?: a prior drawing from this command; accept?: bool. Returns provisional top-view rectangle and vertical cylinders. When geometry changes, returns stale/confirmation_required without modifying previous; accept:true returns an updated drawing with preserved label positions. No persisted sheet, full silhouette or dialog.")];

fn coords(value: &Value, n: usize) -> Option<Vec<f64>> {
    let arr = value.as_array().filter(|a| a.len() == n)?;
    arr.iter().map(|v| v.as_f64().filter(|x| x.is_finite())).collect()
}

fn mm(n: f64) -> f64 {
    (n * 1_000.0).round() / 1_000.0
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
                || !v.get("source").is_some_and(Value::is_object)
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
        },
        "dimensions": [
            {"id":"overall_x","kind":"horizontal","value_mm":dimensions[0]},
            {"id":"overall_y","kind":"vertical","value_mm":dimensions[1]}
        ],
        "label_positions": layout,
        "warnings": if !holes.is_empty() {
            vec!["Cylinder circles are provisional. Bore dimension/callout identity is not yet linked to a stable model source; do not reattach callouts automatically"]
        } else {
            Vec::new()
        }
    });

    if let Some(previous) = prev {
        let stale = previous.get("source") != drawing.get("source");
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
