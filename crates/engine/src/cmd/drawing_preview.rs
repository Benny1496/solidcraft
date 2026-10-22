//! Experimental, read-only, model-linked drawing preview for a rectangular XY plate.
//! This is an explicitly bounded prototype, NOT a general B-rep projection engine.
//! In particular, the rectangle is a bounding-box envelope and cylindrical faces
//! are provisional; saved drawing sheets and a user interface are future work.

use serde_json::{Value, json};

use super::CommandSpec;
use crate::params::{bad, str_};
use crate::{Result, Session};

const CMD: &str = "drawing.preview";

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new(CMD, "Preview Drawing", preview)
        .noundo()
        .at("INSPECT", "DRAWING")
        .icon("inspect")
        .params("body: name; previous?: a prior drawing from this command; accept?: bool. Provisional XY drawing with 3D-derived cylinders, external dimensions, and user-selected annotations linked by verified exact face names. Stale edits require explicit confirmation. Missing references stay unresolved."),
    CommandSpec::new("drawing.annotation.add", "Add Verified Drawing Dimension", annotation_add)
        .noundo()
        .icon("measure")
        .params("body, previous: a current drawing, face_name: exact verified 3D cylindrical face, kind: diameter|x_from_left|y_from_bottom; position_mm?: [x,y]. Returns annotated read-only drawing; save with document.metadata.set. Rejects stale/unverified/ambiguous references."),
];

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
fn provisional_cylinder_dimensions(holes: &[[f64; 3]], min: &[f64], faces: &[Vec<String>]) -> Vec<Value> {
    holes.iter().enumerate().map(|(i,h)| {
        // A face name is eligible only if history explicitly verified it, and
        // it refers to this cylinder group alone (never the next sorted hole).
        let eligible = faces[i].iter().find(|name|
            !faces.iter().enumerate().any(|(j,other)| j != i && other.iter().any(|n| n == *name))
        ).cloned();
        json!({
            "kind": "vertical_cylinder",
            "reference_status": "unverified",
            "eligible_face_name": eligible,
            "face_name_candidates": faces[i],
            "center_mm": [h[0], h[1]],
            "diameter_mm": h[2],
            "x_from_left_mm": mm(h[0] - min[0]),
            "y_from_bottom_mm": mm(h[1] - min[1])
        })
    }).collect()
}

fn annotation_value(d: &Value, kind: &str) -> Option<f64> {
    let key = match kind {
        "diameter" => "diameter_mm",
        "x_from_left" => "x_from_left_mm",
        "y_from_bottom" => "y_from_bottom_mm",
        _ => return None,
    };
    d[key].as_f64().filter(|n| n.is_finite())
}

/// Resolve only against uniquely named, history-verified cylindrical faces.
/// Unresolvable old dimensions survive with their identity and manual layout,
/// but without an authoritative new value.
fn resolve_annotations(previous: Option<&Value>, cylinders: &[Value]) -> Result<Vec<Value>> {
    let Some(old) = previous else { return Ok(Vec::new()) };
    let entries = old.as_array().filter(|a| a.len() <= 64)
        .ok_or_else(|| bad(CMD, "annotations must contain at most 64 entries"))?;
    let mut ids = std::collections::HashSet::new();
    let mut pair = std::collections::HashSet::new();
    let mut out = Vec::new();
    for a in entries {
        let id = a["id"].as_str().filter(|s| s.starts_with("linked_cyl_") && s.len() <= 80
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            .ok_or_else(|| bad(CMD, "invalid annotation id"))?;
        let face = a["face_name"].as_str().filter(|s| !s.is_empty() && s.len() <= 1024)
            .ok_or_else(|| bad(CMD, "invalid annotation face"))?;
        let kind = a["kind"].as_str().filter(|k|
            matches!(*k, "diameter" | "x_from_left" | "y_from_bottom"))
            .ok_or_else(|| bad(CMD, "invalid annotation kind"))?;
        if !ids.insert(id.to_string()) || !pair.insert((face.to_string(),kind.to_string())) {
            return Err(bad(CMD, "duplicate drawing annotation"));
        }
        let matches: Vec<&Value> = cylinders.iter().filter(|d|
            d["eligible_face_name"].as_str() == Some(face)).collect();
        let resolved = matches.len() == 1;
        let original = a["value_mm"].as_f64().filter(|v| v.is_finite())
            .or_else(|| a["last_value_mm"].as_f64().filter(|v| v.is_finite()));
        let value = if resolved { annotation_value(matches[0],kind) } else { None };
        let center = if resolved { matches[0]["center_mm"].clone() }
            else { a.get("center_mm").cloned().unwrap_or(Value::Null) };
        out.push(json!({
            "id":id, "face_name":face, "kind":kind,
            "reference_status":if value.is_some() {"resolved"} else {"missing_reference"},
            "center_mm":center,
            "value_mm":value,
            "last_value_mm":value.or(original)
        }));
    }
    Ok(out)
}

/// A human/user-chosen dimension never silently attaches to a cylinder by
/// list index or XY proximity. The face name must be uniquely certified.
fn annotation_add(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "drawing.annotation.add";
    let name = str_(p,"body").ok_or_else(|| bad(cmd,"body is required"))?;
    let old = p.get("previous").ok_or_else(|| bad(cmd,"previous drawing required"))?;
    let face = str_(p,"face_name").filter(|x| !x.is_empty() && x.len() <= 1024)
        .ok_or_else(|| bad(cmd,"face_name required"))?;
    let kind = str_(p,"kind").filter(|k|
        matches!(*k,"diameter"|"x_from_left"|"y_from_bottom"))
        .ok_or_else(|| bad(cmd,"kind must be diameter, x_from_left, or y_from_bottom"))?;
    let current = preview(s,&json!({"body":name,"previous":old}))?;
    if current["status"] != "current" {
        return Err(bad(cmd,"drawing is stale: confirm original 3D change before adding dimensions"));
    }
    let matches: Vec<&Value> = current["drawing"]["geometry"]["cylinder_dimensions"]
        .as_array().into_iter().flatten().filter(|d|
            d["eligible_face_name"].as_str() == Some(face)).collect();
    if matches.len() != 1 {
        return Err(bad(cmd,"cylindrical face is not unique and history-verified"));
    }
    let saved = &current["drawing"];
    if saved["annotations"].as_array().is_some_and(|list| list.iter().any(|a|
        a["face_name"].as_str() == Some(face) && a["kind"].as_str() == Some(kind))) {
        return Err(bad(cmd,"dimension is already annotated"));
    }
    let mut annotated = saved.clone();
    let list = annotated["annotations"].as_array_mut()
        .ok_or_else(|| bad(cmd,"invalid drawing annotations"))?;
    if list.len() >= 64 { return Err(bad(cmd,"too many dimensions")); }
    let mut counter = 1;
    let id = loop {
        let candidate=format!("linked_cyl_{counter}");
        if !list.iter().any(|a| a["id"].as_str()==Some(candidate.as_str())) { break candidate; }
        counter+=1;
        if counter>128 { return Err(bad(cmd,"annotation ids exhausted")); }
    };
    let d=matches[0];
    let location = match p.get("position_mm") {
        Some(v) => coords(v,2).filter(|a| a.iter().all(|x| x.abs() <= 1_000_000.0))
            .ok_or_else(|| bad(cmd,"invalid annotation position"))?,
        None => {
            let center = coords(&d["center_mm"],2)
                .ok_or_else(|| bad(cmd,"invalid cylinder centre"))?;
            vec![center[0]+12.0,center[1]+12.0]
        },
    };
    list.push(json!({
        "id":id,"face_name":face,"kind":kind,"value_mm":annotation_value(d,kind),
        "center_mm":d["center_mm"],"reference_status":"resolved"
    }));
    annotated["label_positions"][id.as_str()]=json!(location);
    let rechecked=preview(s,&json!({"body":name,"previous":annotated}))?;
    if rechecked["status"] != "current" {
        return Err(bad(cmd,"model changed before annotation could be saved"));
    }
    Ok(json!({"status":"annotated","drawing":rechecked["drawing"],"annotation_id":id}))
}

// A limited, geometrically scaled SVG view for the same verified XY envelope.
// Width/height use physical millimetres. Geometry is not a true occlusion/HLR
// projection; the drawing contract states the supported subset explicitly.
fn svg_preview(min: &[f64], max: &[f64], cylinders: &[[f64; 3]], labels: &Value, annotations: &[Value]) -> String {
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
    // A saved user-selected annotation has an actual placed leader and text
    // in the exported SVG; only its measured number is trusted as drawing
    // content (never interpolate an untrusted face name into XML).
    for a in annotations {
        let Some(id)=a["id"].as_str() else { continue };
        let Some(from)=coords(&a["center_mm"],2) else { continue };
        let Some(at)=labels.get(id).and_then(|v| coords(v,2)) else { continue };
        let (x0,y0)=map(&from);
        let (x1,y1)=map(&at);
        let resolved=a["reference_status"] == "resolved";
        let color=if resolved {"#12283c"} else {"#b44c24"};
        let historic=a["value_mm"].as_f64().or_else(||a["last_value_mm"].as_f64())
            .filter(|v|v.is_finite()).unwrap_or(0.0);
        let prefix=if resolved {""} else {"? "};
        let label=match a["kind"].as_str() {
            Some("diameter")=>format!("{prefix}Ø{historic:.2}"),
            Some("x_from_left")=>format!("{prefix}X {historic:.2}"),
            Some("y_from_bottom")=>format!("{prefix}Y {historic:.2}"),
            _=>continue,
        };
        svg.push_str(&format!(
            "<path stroke=\"{color}\" stroke-width=\"0.17\" d=\"M {x0} {y0} L {x1} {y1}\"/><text x=\"{x1}\" y=\"{y1}\" font-family=\"sans-serif\" font-size=\"3.5\" fill=\"{color}\" text-anchor=\"middle\">{label}</text>"
        ));
    }
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
    let mut holes: Vec<([f64; 3], Vec<String>)> = Vec::new();
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
        let exact_name = face["name"].as_str().filter(|_| face["exact"] == true)
            .filter(|n| !n.is_empty() && n.len() <= 1024).map(str::to_string);
        if let Some(group) = holes.iter_mut().find(|h|
            (h.0[0] - x).abs() < 0.01 && (h.0[1] - y).abs() < 0.01
            && (h.0[2] - diameter).abs() < 0.01)
        {
            if let Some(n) = exact_name {
                if !group.1.contains(&n) { group.1.push(n); }
            }
        } else {
            if holes.len() >= 64 {
                return Err(bad(CMD, "more than 64 cylinders"));
            }
            holes.push(([x, y, diameter], exact_name.into_iter().collect()));
        }
    }
    holes.sort_by(|a, b| a.0[1].total_cmp(&b.0[1])
        .then(a.0[0].total_cmp(&b.0[0]))
        .then(a.0[2].total_cmp(&b.0[2])));
    let eligible_faces: Vec<Vec<String>> = holes.iter().map(|h| h.1.clone()).collect();
    let holes: Vec<[f64; 3]> = holes.into_iter().map(|h| h.0).collect();
    let measured_cylinders = provisional_cylinder_dimensions(&holes,&min,&eligible_faces);

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
    let annotated = resolve_annotations(prev.and_then(|d| d.get("annotations")), &measured_cylinders)?;
    let mut label_defaults = current_labels;
    for annotation in &annotated {
        if let (Some(id), Some(c)) = (
            annotation["id"].as_str(),
            coords(&annotation["center_mm"],2)
        ) {
            label_defaults[id] = json!([c[0]+12.0,c[1]+12.0]);
        }
    }
    let layout = label_positions(prev.and_then(|d| d.get("label_positions")), &label_defaults)?;
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
            "cylinder_dimensions": measured_cylinders,
        },
        "dimensions": [
            {"id":"overall_x","kind":"horizontal","value_mm":dimensions[0]},
            {"id":"overall_y","kind":"vertical","value_mm":dimensions[1]}
        ],
        "svg": svg_preview(&min, &max, &holes, &layout, &annotated),
        "annotations": annotated,
        "label_positions": layout,
        "warnings": if !holes.is_empty() {
            vec!["Cylinders are measured but not necessarily drilled holes; only user-selected dimensions with unique, history-verified face names are safely linked. The outer XY rectangle is not a true manufacturing projection"]
        } else {
            Vec::new()
        }
    });

    if let Some(previous) = prev {
        let missing = drawing["annotations"].as_array().is_some_and(|a| a.iter().any(|d|
            d["reference_status"] == "missing_reference" && previous["annotations"]
                .as_array().is_some_and(|old| old.iter().any(|p|
                    p["id"] == d["id"] && p["reference_status"] == "resolved"))));
        let stale = !source_matches(&previous["source"], &drawing["source"]) || missing;
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
    use serde_json::{Value, json};

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

    #[test]
    fn diameter_and_xy_offsets_come_from_measured_3d_cylinders() {
        let mut session=Session::default();
        session.execute("solid.box",&json!({
            "length":110,"width":70,"height":8,
            "corner":[-30,-20,0],"body_name":"OffsetPlate"
        })).unwrap();
        session.execute("solid.hole",&json!({
            "name":"Drill6","position":[-18,-8,8],"diameter":6
        })).unwrap();
        session.execute("solid.hole",&json!({
            "name":"Drill10","position":[50,40,8],"diameter":10
        })).unwrap();
        let drawing=preview(&mut session,&json!({"body":"OffsetPlate"})).unwrap();
        let dims=drawing["drawing"]["geometry"]["cylinder_dimensions"].as_array().unwrap();
        assert_eq!(dims.len(),2);
        assert_eq!(drawing["drawing"]["dimensions"][0]["value_mm"].as_f64(),Some(110.0));
        assert_eq!(drawing["drawing"]["dimensions"][1]["value_mm"].as_f64(),Some(70.0));
        let first=dims.iter().find(|d| d["diameter_mm"].as_f64()==Some(6.0)).unwrap();
        assert_eq!(first["x_from_left_mm"].as_f64(),Some(12.0));
        assert_eq!(first["y_from_bottom_mm"].as_f64(),Some(12.0));
        assert_eq!(first["reference_status"],"unverified");
        let second=dims.iter().find(|d| d["diameter_mm"].as_f64()==Some(10.0)).unwrap();
        assert_eq!(second["x_from_left_mm"].as_f64(),Some(80.0));
        assert_eq!(second["y_from_bottom_mm"].as_f64(),Some(60.0));
        assert_eq!(second["reference_status"],"unverified");
    }

    #[test]
    fn cylinder_measurements_update_only_in_proposed_drawing_until_confirmation() {
        let mut session=Session::default();
        session.execute("solid.box",&json!({
            "length":95,"width":50,"height":8,"body_name":"Plate"
        })).unwrap();
        session.execute("solid.hole",&json!({
            "name":"Drill","position":[14,20,8],"diameter":8
        })).unwrap();
        let first=preview(&mut session,&json!({"body":"Plate"})).unwrap();
        let old=first["drawing"].clone();
        assert_eq!(old["geometry"]["cylinder_dimensions"].as_array().unwrap().len(),1);
        session.execute("solid.hole",&json!({
            "name":"SecondDrill","position":[72,32,8],"diameter":10
        })).unwrap();
        let pending=preview(&mut session,&json!({
            "body":"Plate","previous":old
        })).unwrap();
        assert_eq!(pending["status"],"stale");
        assert_eq!(pending["drawing"]["geometry"]["cylinder_dimensions"].as_array().unwrap().len(),1);
        assert_eq!(pending["proposed"]["geometry"]["cylinder_dimensions"].as_array().unwrap().len(),2);
        let accepted=preview(&mut session,&json!({
            "body":"Plate","previous":old,"accept":true
        })).unwrap();
        assert_eq!(accepted["status"],"updated");
        assert_eq!(accepted["drawing"]["geometry"]["cylinder_dimensions"].as_array().unwrap().len(),2);
        assert_eq!(preview(&mut session,&json!({
            "body":"Plate","previous":accepted["drawing"]
        })).unwrap()["status"],"current");
    }

    #[test]
    fn exact_face_annotations_recompute_after_change_and_preserve_user_identity() {
        let first = json!([
            {"eligible_face_name":"F42:side", "center_mm":[10,20],
             "diameter_mm":8, "x_from_left_mm":10, "y_from_bottom_mm":20}
        ]);
        let previous = json!([
            {"id":"linked_cyl_1","face_name":"F42:side","kind":"diameter",
             "reference_status":"resolved","value_mm":8,"center_mm":[10,20]},
            {"id":"linked_cyl_2","face_name":"F42:side","kind":"x_from_left",
             "reference_status":"resolved","value_mm":10,"center_mm":[10,20]}
        ]);
        let original = super::resolve_annotations(Some(&previous),first.as_array().unwrap()).unwrap();
        assert_eq!(original[0]["value_mm"].as_f64(), Some(8.0));
        assert_eq!(original[1]["value_mm"].as_f64(),Some(10.0));
        let edited = json!([
            {"eligible_face_name":"F42:side","center_mm":[14,20],
             "diameter_mm":12,"x_from_left_mm":14,"y_from_bottom_mm":20},
            {"eligible_face_name":"F100:side","center_mm":[80,50],
             "diameter_mm":6,"x_from_left_mm":80,"y_from_bottom_mm":50}
        ]);
        let again=super::resolve_annotations(Some(&previous),edited.as_array().unwrap()).unwrap();
        assert_eq!(again[0]["id"],"linked_cyl_1");
        assert_eq!(again[0]["face_name"],"F42:side");
        assert_eq!(again[0]["reference_status"],"resolved");
        assert_eq!(again[0]["value_mm"].as_f64(),Some(12.0));
        assert_eq!(again[1]["value_mm"].as_f64(),Some(14.0));
    }

    #[test]
    fn removed_or_ambiguous_3d_faces_are_never_silently_relinked() {
        let old = json!([{"id":"linked_cyl_1","face_name":"F7:bore",
            "kind":"diameter","reference_status":"resolved","value_mm":8,"center_mm":[15,15]}]);
        let unrelated = json!([{"eligible_face_name":"F99:bore",
            "center_mm":[15,15],"diameter_mm":8,"x_from_left_mm":15,"y_from_bottom_mm":15}]);
        let missing=super::resolve_annotations(Some(&old),unrelated.as_array().unwrap()).unwrap();
        assert_eq!(missing[0]["reference_status"],"missing_reference");
        assert!(missing[0]["value_mm"].is_null(),"stale value is not certified as current");
        assert_eq!(missing[0]["last_value_mm"].as_f64(),Some(8.0));
        let twice=json!([
            {"eligible_face_name":"F7:bore","center_mm":[15,15],
             "diameter_mm":8,"x_from_left_mm":15,"y_from_bottom_mm":15},
            {"eligible_face_name":"F7:bore","center_mm":[70,15],
             "diameter_mm":8,"x_from_left_mm":70,"y_from_bottom_mm":15}
        ]);
        let ambiguous=super::resolve_annotations(Some(&old),twice.as_array().unwrap()).unwrap();
        assert_eq!(ambiguous[0]["reference_status"],"missing_reference");
    }

    #[test]
    fn malformed_duplicate_and_oversized_annotations_are_rejected() {
        let good=json!([{"id":"linked_cyl_1","face_name":"F5:side",
            "kind":"diameter","value_mm":8,"center_mm":[10,10]}]);
        let refs=json!([{"eligible_face_name":"F5:side","center_mm":[10,10],
            "diameter_mm":8,"x_from_left_mm":10,"y_from_bottom_mm":10}]);
        for broken in [
            json!([good[0],good[0]]),
            json!([{"id":"bad<id>","face_name":"F5:side","kind":"diameter"}]),
            json!([{"id":"linked_cyl_1","face_name":"F5:side","kind":"unsupported"}]),
            json!([{"id":"linked_cyl_1","face_name":"","kind":"diameter"}]),
            json!([{"id":"linked_cyl_1","face_name":"F5:side","kind":"diameter"},
                   {"id":"linked_cyl_2","face_name":"F5:side","kind":"diameter"}])
        ] {
            assert!(super::resolve_annotations(Some(&broken),refs.as_array().unwrap()).is_err(),
                "malformed dimension must be rejected {broken}");
        }
        let oversized = Value::Array((0..65).map(|i| json!({
            "id":format!("linked_cyl_{}",i),"face_name":format!("F{i}"),
            "kind":"diameter"
        })).collect());
        assert!(super::resolve_annotations(Some(&oversized),refs.as_array().unwrap()).is_err());
    }

    #[test]
    fn user_selected_3d_hole_face_creates_real_named_dimension_and_follows_parameter_change() {
        let mut session = Session::default();
        session.execute("parameters.add",&json!({"name":"DrillDiameter","expression":"8 mm"})).unwrap();
        session.execute("solid.box",&json!({"length":120,"width":60,"height":8,
            "name":"BaseStock","body_name":"Plate"})).unwrap();
        session.execute("solid.hole",&json!({"name":"ReferenceDrill",
            "position":[18,22,8],"diameter":"DrillDiameter"})).unwrap();
        let original=preview(&mut session,&json!({"body":"Plate"})).unwrap()["drawing"].clone();
        let face=original["geometry"]["cylinder_dimensions"].as_array().unwrap().iter()
            .filter_map(|d| d["eligible_face_name"].as_str()).next()
            .expect("the drill cylinder must expose a unique history-verified face").to_string();
        let added=super::annotation_add(&mut session,&json!({
            "body":"Plate","previous":original,"face_name":face,
            "kind":"diameter","position_mm":[43,39]
        })).unwrap();
        assert_eq!(added["status"],"annotated");
        let old=added["drawing"].clone();
        assert_eq!(old["annotations"].as_array().unwrap().len(),1);
        assert_eq!(old["annotations"][0]["reference_status"],"resolved");
        assert_eq!(old["annotations"][0]["value_mm"].as_f64(),Some(8.0));
        let id=added["annotation_id"].as_str().unwrap();
        assert_eq!(super::coords(&old["label_positions"][id],2),Some(vec![43.0,39.0]));
        session.execute("parameters.change",&json!({
            "name":"DrillDiameter","expression":"10 mm"
        })).unwrap();
        let pending=preview(&mut session,&json!({"body":"Plate","previous":old})).unwrap();
        assert_eq!(pending["status"],"stale");
        assert_eq!(pending["drawing"]["annotations"][0]["value_mm"].as_f64(),Some(8.0));
        assert_eq!(pending["proposed"]["annotations"][0]["reference_status"],"resolved");
        assert_eq!(pending["proposed"]["annotations"][0]["value_mm"].as_f64(),Some(10.0));
        let accepted=preview(&mut session,&json!({
            "body":"Plate","previous":old,"accept":true
        })).unwrap();
        assert_eq!(accepted["status"],"updated");
        assert_eq!(accepted["drawing"]["annotations"][0]["value_mm"].as_f64(),Some(10.0));
        assert_eq!(accepted["drawing"]["label_positions"][id].as_array().unwrap().len(),2);
        assert!(super::annotation_add(&mut session,&json!({
            "body":"Plate","previous":accepted["drawing"],
            "face_name":face,"kind":"diameter"
        })).is_err(),"a duplicate of the same dimension must be rejected");
    }

    #[test]
    fn lost_annotation_historical_dimension_survives_repeated_preview_and_accept() {
        let old=json!([{"id":"linked_cyl_1","face_name":"F15:inner",
            "kind":"diameter","center_mm":[18,16],"reference_status":"resolved",
            "value_mm":8,"last_value_mm":8}]);
        let empty:Vec<Value>=Vec::new();
        let first=super::resolve_annotations(Some(&old),&empty).unwrap();
        assert_eq!(first[0]["reference_status"],"missing_reference");
        assert_eq!(first[0]["last_value_mm"].as_f64(),Some(8.0));
        let again=super::resolve_annotations(Some(&json!(first)),&empty).unwrap();
        assert_eq!(again[0]["last_value_mm"].as_f64(),Some(8.0));
        assert!(again[0]["value_mm"].is_null());
    }

    #[test]
    fn selected_face_annotation_survives_actual_solidcraft_save_and_reopen() {
        let path=std::env::temp_dir().join(format!(
            "solidcraft-linked-drawing-test-{}.solidcraft",std::process::id()));
        let mut session=Session::default();
        session.execute("parameters.add",&json!({
            "name":"DrillD","expression":"8 mm"})).unwrap();
        session.execute("solid.box",&json!({
            "length":110,"width":55,"height":8,"body_name":"Plate"
        })).unwrap();
        session.execute("solid.hole",&json!({
            "name":"Bore","position":[18,21,8],"diameter":"DrillD"
        })).unwrap();
        let first=preview(&mut session,&json!({"body":"Plate"})).unwrap();
        let candidate=first["drawing"]["geometry"]["cylinder_dimensions"]
            .as_array().unwrap().iter().find_map(|d|
                d["eligible_face_name"].as_str())
            .expect("must have history-verified source cylinder");
        let accepted=super::annotation_add(&mut session,&json!({
            "body":"Plate","previous":first["drawing"],
            "face_name":candidate,"kind":"diameter","position_mm":[43,43]
        })).unwrap();
        session.execute("document.metadata.set",&json!({
            "key":"org.solidcraft.experimental_drawing","value":accepted["drawing"]
        })).unwrap();
        session.execute("file.save_as",&json!({
            "path":path,"format":"container"
        })).unwrap();
        let mut opened=Session::default();
        opened.execute("doc.open",&json!({"path":path})).unwrap();
        let saved=opened.doc.metadata.get("org.solidcraft.experimental_drawing")
            .expect("linked drawing persists in real .solidcraft archive").clone();
        assert_eq!(saved["annotations"][0]["reference_status"],"resolved");
        assert_eq!(saved["annotations"][0]["value_mm"].as_f64(),Some(8.0));
        let same=preview(&mut opened,&json!({
            "body":"Plate","previous":saved
        })).unwrap();
        assert_eq!(same["status"],"current");
        assert_eq!(same["drawing"]["annotations"][0]["reference_status"],"resolved");
        opened.execute("parameters.change",&json!({
            "name":"DrillD","expression":"10 mm"
        })).unwrap();
        let pending=preview(&mut opened,&json!({
            "body":"Plate","previous":saved
        })).unwrap();
        assert_eq!(pending["status"],"stale");
        assert_eq!(pending["proposed"]["annotations"][0]["reference_status"],"resolved");
        assert_eq!(pending["proposed"]["annotations"][0]["value_mm"].as_f64(),Some(10.0));
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn only_uniquely_verified_faces_can_be_selected_on_the_2d_circle() {
        let cylinders=[
            [12.0, 15.0, 6.0],
            [35.0, 15.0, 8.0],
            [60.0, 15.0, 10.0],
            [85.0, 15.0, 10.0],
        ];
        // The preview collects only history-exact names into this table.
        // Nonexact source faces must not enter it. Two groups with the same
        // "verified" name are still ambiguous and remain unselectable.
        let certified=vec![
            vec!["F11:side".to_string()],
            vec![],
            vec!["Fshared:side".to_string()],
            vec!["Fshared:side".to_string()],
        ];
        let measured=super::provisional_cylinder_dimensions(
            &cylinders,&[0.0,0.0,0.0],&certified);
        assert_eq!(measured.len(),4);
        assert_eq!(measured[0]["eligible_face_name"],"F11:side");
        assert!(measured[1]["eligible_face_name"].is_null());
        assert!(measured[2]["eligible_face_name"].is_null());
        assert!(measured[3]["eligible_face_name"].is_null());
        assert!(measured.iter().all(|d|d["reference_status"] == "unverified"),
            "a geometrically projected cylinder is not necessarily a drilled hole");
    }

    #[test]
    fn reversing_cylinder_order_does_not_move_an_annotation_to_another_bore() {
        let previous=json!([
            {"id":"linked_cyl_1","face_name":"F1:bore","kind":"diameter",
             "value_mm":6,"center_mm":[20,20]},
            {"id":"linked_cyl_2","face_name":"F2:bore","kind":"diameter",
             "value_mm":10,"center_mm":[85,45]},
            {"id":"linked_cyl_3","face_name":"F1:bore","kind":"x_from_left",
             "value_mm":20,"center_mm":[20,20]}
        ]);
        // Simulate a topology rebuild where the order, diameters and
        // positions change. Never resolve by sorted index or closest radius.
        let current=json!([
            {"eligible_face_name":"F2:bore","center_mm":[17,19],
             "diameter_mm":12,"x_from_left_mm":17,"y_from_bottom_mm":19},
            {"eligible_face_name":"F1:bore","center_mm":[83,44],
             "diameter_mm":8,"x_from_left_mm":83,"y_from_bottom_mm":44}
        ]);
        let updated=super::resolve_annotations(
            Some(&previous),current.as_array().unwrap()).unwrap();
        assert_eq!(updated[0]["id"],"linked_cyl_1");
        assert_eq!(updated[0]["face_name"],"F1:bore");
        assert_eq!(updated[0]["value_mm"].as_f64(),Some(8.0));
        assert_eq!(updated[1]["face_name"],"F2:bore");
        assert_eq!(updated[1]["value_mm"].as_f64(),Some(12.0));
        assert_eq!(updated[2]["face_name"],"F1:bore");
        assert_eq!(updated[2]["value_mm"].as_f64(),Some(83.0));
        assert_eq!(updated[2]["reference_status"],"resolved");
    }

}
