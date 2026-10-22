//! Experimental visible 2D drawing window for a single XY manufacturing preview.
//! The snapshot is stored as versioned informational metadata in this prototype.
//! A native Drawing document and precise silhouette/provenance are future work.

use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, pos2, vec2};
use serde_json::{Value, json};

use crate::{SolveApp, theme::Tokens};

/// Temporary private metadata key. The full Drawing format must replace this
/// prototype rather than treating arbitrary metadata as authoritative CAD geometry.
const KEY: &str = "org.solidcraft.experimental_drawing";

#[derive(Default)]
pub struct State {
    pub open: bool,
    pub body: String,
    snapshot: Option<Value>,
    draft: Option<Value>,
    proposed: Option<Value>,
    error: Option<String>,
    checked_revision: Option<u64>,
    dismissed_revision: Option<u64>,
    layout_changed: bool,
    /// UI-only selection of a certified 3D cylindrical face; never persisted as
    /// an annotation until the user explicitly picks a dimension type.
    selected_face: Option<String>,
    distance_anchor: Option<String>,
}

fn xy(v: &Value) -> Option<[f32; 2]> {
    let a = v.as_array().filter(|a| a.len() == 2)?;
    let x = a.first()?.as_f64()?;
    let y = a.get(1)?.as_f64()?;
    (x.is_finite() && y.is_finite() && x.abs() <= 1_000_000.0 && y.abs() <= 1_000_000.0)
        .then_some([x as f32, y as f32])
}

fn point(value: &Value, key: &str) -> Option<[f32; 2]> {
    xy(value.get("label_positions")?.get(key)?)
}

fn get_body_names(app: &SolveApp) -> Vec<String> {
    app.session.model.state().bodies.iter().map(|body| body.name.clone()).collect()
}

/// Open from the ribbon / command palette, always rechecking source freshness.
pub fn open(app: &mut SolveApp) {
    let previous = app.session.doc.metadata.get(KEY).cloned();
    let names = get_body_names(app);
    let body = previous.as_ref()
        .and_then(|v| v.get("body").and_then(Value::as_str))
        .filter(|n| names.iter().any(|x| x == n))
        .map(str::to_string)
        .or_else(|| names.first().cloned())
        .unwrap_or_default();
    // A saved sheet can refer to a body that has since been removed. Start
    // with a fresh preview for a surviving body instead of repeatedly passing
    // an incompatible stale snapshot back to drawing.preview.
    let previous = previous.filter(|v| v["body"].as_str() == Some(body.as_str()));
    let mut state = State {
        open: true,
        body,
        snapshot: previous,
        ..State::default()
    };
    refresh(app, &mut state);
    app.drawing_ui = state;
}

/// Never mutate a drawing while checking source freshness.
fn refresh(app: &mut SolveApp, state: &mut State) {
    state.checked_revision = Some(app.session.revision);
    state.error = None;
    state.proposed = None;
    state.draft = None;
    if state.body.is_empty() {
        state.error = Some("Kein 3D-Körper vorhanden.".into());
        return;
    }
    let mut params = json!({"body": state.body});
    if let Some(previous) = &state.snapshot {
        params["previous"] = previous.clone();
    }
    match app.session.execute("drawing.preview", &params) {
        Ok(reply) => match reply["status"].as_str() {
            Some("created") => state.draft = reply.get("drawing").cloned(),
            Some("current") => {
                state.draft = reply.get("drawing").cloned();
            }
            Some("stale") => state.proposed = reply.get("proposed").cloned(),
            _ => state.error = Some("Unbekannter Zeichnungsstatus.".into()),
        },
        Err(err) => state.error = Some(err.to_string()),
    }
}

fn persist(app: &mut SolveApp, state: &mut State, drawing: Value) {
    match app.run("document.metadata.set", json!({"key": KEY, "value":drawing})) {
        Ok(_) => {
            state.snapshot = Some(drawing);
            state.dismissed_revision = None;
            state.layout_changed = false;
            refresh(app, state);
        }
        Err(err) => state.error = Some(err),
    }
}

fn accept(app: &mut SolveApp, state: &mut State) {
    let Some(previous) = state.snapshot.as_ref() else { return };
    let params = json!({"body":state.body, "previous":previous, "accept":true});
    match app.session.execute("drawing.preview", &params) {
        Ok(reply) if reply["status"] == "updated" || reply["status"] == "current" => {
            if let Some(drawing) = reply.get("drawing") {
                persist(app, state, drawing.clone());
            }
        }
        Ok(_) => state.error = Some("Aktualisierung lieferte keinen gültigen Zeichnungsstand.".into()),
        Err(err) => state.error = Some(err.to_string()),
    }
}

fn save_layout(app: &mut SolveApp, state: &mut State) {
    let Some(previous) = state.snapshot.as_ref() else { return };
    if state.proposed.is_some() {
        state.error = Some("Die veraltete Zeichnung zuerst aktualisieren.".into());
        return;
    }
    let params = json!({"body":state.body, "previous":previous});
    match app.session.execute("drawing.preview", &params) {
        Ok(reply) if reply["status"] == "current" => {
            if let Some(drawing) = reply.get("drawing") {
                persist(app, state, drawing.clone());
            }
        }
        Ok(reply) if reply["status"] == "stale" => {
            state.proposed = reply.get("proposed").cloned();
            state.error = Some("Das 3D-Modell hat sich geändert. Bitte Aktualisierung bestätigen.".into());
        }
        Ok(_) => state.error = Some("Die Beschriftungspositionen konnten nicht gespeichert werden.".into()),
        Err(err) => state.error = Some(err.to_string()),
    }
}

fn draw_label(
    ui: &mut egui::Ui, drawing: &mut Value, id: &str, text: String,
    at: Pos2, scale: f32, editable: bool, text_color: Color32,
) -> bool {
    let hit = Rect::from_center_size(at, vec2(102.0, 25.0));
    let response = ui.interact(hit, ui.id().with(("drawing_label", id)), if editable { Sense::drag() } else { Sense::hover() });
    crate::scenario::publish_target(&format!("drawing:label:{id}"), hit);
    if editable && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }
    // egui's response.drag_delta() is cumulative from drag start; adding that
    // to the current position every frame would exaggerate the movement.
    // Apply only the latest pointer movement while the label is being dragged.
    let d = if response.dragged() { ui.input(|i| i.pointer.delta()) } else { egui::Vec2::ZERO };
    if editable && (d.x != 0.0 || d.y != 0.0)
        && let Some([x, y]) = point(drawing, id)
    {
        drawing["label_positions"][id] = json!([f64::from(x + d.x / scale), f64::from(y - d.y / scale)]);
    }
    ui.painter().text(at, Align2::CENTER_CENTER, text, FontId::proportional(16.0), text_color);
    response.drag_stopped()
}

fn render_plate(ui: &mut egui::Ui, drawing: &mut Value, editable: bool, selected_face: &mut Option<String>) -> bool {
    let t = Tokens::get();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width().max(280.0).min(670.0), 355.0), Sense::hover());
    ui.painter().rect_filled(rect, 6.0, t.panel);
    ui.painter().rect_stroke(rect, 6.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
    let Some(outline) = drawing.get("geometry").and_then(|g| g.get("outline")) else { return false };
    let Some(min) = xy(&outline["min_xy_mm"]) else { return false };
    let Some(max) = xy(&outline["max_xy_mm"]) else { return false };
    let w = max[0] - min[0];
    let h = max[1] - min[1];
    if !(w > 0.0 && h > 0.0) { return false }
    let scale = ((rect.width() - 165.0).max(10.0) / w)
        .min((rect.height() - 120.0).max(10.0) / h).max(0.001);
    let center = rect.center();
    let map = |p: [f32; 2]| {
        pos2(center.x + (p[0] - (min[0] + max[0]) * 0.5) * scale,
             center.y - (p[1] - (min[1] + max[1]) * 0.5) * scale)
    };
    let left_top = map([min[0], max[1]]);
    let right_bottom = map([max[0], min[1]]);
    ui.painter().rect_stroke(Rect::from_min_max(left_top, right_bottom), 0.0, Stroke::new(1.0, t.text_dim), StrokeKind::Inside);
    // Show sampled top-plane edge candidates in a distinct colour. These are
    // model-derived lines and curves, not a certified visible silhouette.
    if let Some(paths)=drawing["geometry"]["top_edge_candidates"].as_array() {
        for path in paths {
            let Some(samples)=path["points_xy_mm"].as_array() else {continue};
            let pts: Vec<Pos2>=samples.iter().filter_map(xy).map(map).collect();
            if pts.len()>=2 && pts.len()<=1024 {
                ui.painter().add(egui::Shape::line(pts,Stroke::new(1.9,t.accent)));
            }
        }
    }

    if let Some(circles) = drawing["geometry"]["cylinders"].as_array() {
        for (i,c) in circles.iter().enumerate() {
            if let (Some(at), Some(d)) = (xy(&c["center_mm"]), c["diameter_mm"].as_f64()) {
                if d.is_finite() && d > 0.0 && d < 1e6 {
                    let p = map(at);
                    let exact_face = drawing["geometry"]["cylinder_dimensions"][i]
                        ["eligible_face_name"].as_str();
                    let selected = exact_face.is_some_and(|face| selected_face.as_deref() == Some(face));
                    let ring = if selected { t.accent } else { t.text };
                    ui.painter().circle_stroke(p, d as f32 * scale * 0.5,
                        Stroke::new(if selected { 3.0 } else { 1.5 },ring));
                    // Hit test in screen space; a small bore still has a usable
                    // finger/mouse target. Unverified faces remain read-only.
                    let radius = (d as f32 * scale * 0.5).clamp(11.0, 32.0);
                    let hit = Rect::from_center_size(p,vec2(radius*2.0,radius*2.0));
                    crate::scenario::publish_target(&format!("drawing:cylinder:{i}"),hit);
                    if editable && let Some(face) = exact_face {
                        let response=ui.interact(hit,ui.id().with(("drawing_cylinder",i)),Sense::click());
                        if response.hovered() {ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);}
                        if response.clicked() {
                            *selected_face = Some(face.to_string());
                        }
                    }
                    let cross = 5.0;
                    ui.painter().line_segment([p - vec2(cross,0.0), p + vec2(cross,0.0)], Stroke::new(0.7,t.text_dim));
                    ui.painter().line_segment([p - vec2(0.0,cross), p + vec2(0.0,cross)], Stroke::new(0.7,t.text_dim));
                    // Provisional, read-only diameter note based on the true
                    // B-rep cylinder; no stable feature reference or manually
                    // arranged dimension is implied at this stage.
                    let radius_px = d as f32 * scale * 0.5;
                    let leader_start = p + vec2(radius_px * 0.70, -radius_px * 0.70);
                    let leader_end = leader_start + vec2(13.0, -14.0);
                    ui.painter().line_segment([leader_start, leader_end], Stroke::new(0.7,t.text_dim));
                    ui.painter().text(leader_end + vec2(2.0,-1.0), Align2::LEFT_BOTTOM,
                        format!("Ø{d:.2}"),FontId::proportional(11.0),t.text_dim);
                }
            }
        }
    }

    let lx = point(drawing, "overall_x").unwrap_or([(min[0]+max[0])*0.5, min[1]-12.0]);
    let ly = point(drawing, "overall_y").unwrap_or([min[0]-12.0,(min[1]+max[1])*0.5]);
    let dim_x=map(lx);
    let dim_y=map(ly);
    let thin = Stroke::new(0.8,t.text_dim);
    let l=map([min[0],min[1]]);
    let r=map([max[0],min[1]]);
    ui.painter().line_segment([l,pos2(l.x,dim_x.y)],thin);
    ui.painter().line_segment([r,pos2(r.x,dim_x.y)],thin);
    ui.painter().line_segment([pos2(l.x,dim_x.y),pos2(r.x,dim_x.y)],thin);
    let top=map([min[0],max[1]]);
    ui.painter().line_segment([top,pos2(dim_y.x,top.y)],thin);
    ui.painter().line_segment([l,pos2(dim_y.x,l.y)],thin);
    ui.painter().line_segment([pos2(dim_y.x,top.y),pos2(dim_y.x,l.y)],thin);
    let mut stopped = false;
    let width = drawing["dimensions"][0]["value_mm"].as_f64().unwrap_or(f64::from(w));
    let height = drawing["dimensions"][1]["value_mm"].as_f64().unwrap_or(f64::from(h));
    stopped |= draw_label(ui,drawing,"overall_x",format!("{width:.2} mm"),dim_x,scale,editable,t.text);
    stopped |= draw_label(ui,drawing,"overall_y",format!("{height:.2} mm"),dim_y,scale,editable,t.text);
    // User-selected dimensions are anchored by an exact named 3D face.
    // Their labels are independent of cylinder list order and remain visible
    // with a warning after a reference is removed or becomes ambiguous.
    let annotations = drawing["annotations"].as_array().cloned().unwrap_or_default();
    for annotation in annotations {
        let Some(id) = annotation["id"].as_str() else { continue };
        let Some(source) = xy(&annotation["center_mm"]) else { continue };
        let Some(at_xy) = point(drawing,id) else { continue };
        let at = map(at_xy);
        let from = map(source);
        let resolved = annotation["reference_status"] == "resolved";
        let color = if resolved { t.text } else { t.warning };
        ui.painter().line_segment([from,at],Stroke::new(0.9,color));
        let value = annotation["value_mm"].as_f64().or_else(||
            annotation["last_value_mm"].as_f64()).unwrap_or(0.0);
        let kind = annotation["kind"].as_str().unwrap_or("");
        let prefix = if !resolved { "? " } else { "" };
        let text = match kind {
            "diameter" => format!("{prefix}Ø{value:.2} mm"),
            "x_from_left" => format!("{prefix}X {value:.2} mm"),
            "y_from_bottom" => format!("{prefix}Y {value:.2} mm"),
            "center_distance" => format!("{prefix}M–M {value:.2} mm"),
            _ => continue,
        };
        if kind == "center_distance"
            && let Some(second) = xy(&annotation["second_center_mm"])
        {
            let second = map(second);
            ui.painter().line_segment([from,second],Stroke::new(0.9,color));
            ui.painter().line_segment([second,at],Stroke::new(0.9,color));
        }
        stopped |= draw_label(ui,drawing,id,text,at,scale,editable,color);
    }
    stopped
}

/// Two extra orthographic reference panels inside the same native Drawing
/// window. The source is the saved 3D geometry snapshot, not live geometry
/// mixed into a stale drawing awaiting user confirmation.
fn render_reference_views(ui: &mut egui::Ui, drawing: &Value) {
    let t=Tokens::get();
    let Some(views)=drawing["geometry"]["reference_views"].as_object() else {return};
    ui.separator();
    ui.label(RichText::new("3D-verknüpfte Referenzansichten (vorläufig)").strong());
    ui.label(RichText::new("Die Linien stammen von äußeren 3D-Kantenebenen. Verdeckte Linien, vollständige Silhouetten und Fertigungsansichten werden noch nicht sicher berechnet.").small().color(t.text_dim));
    ui.horizontal(|ui|{
        for (key,title) in [
            ("front","Vorderansicht (XZ, Y min) · vorläufig"),
            ("right","Seitenansicht (YZ, X max) · vorläufig")
        ] {
            let Some(view)=views.get(key) else {continue};
            ui.vertical(|ui|{
                ui.label(title);
                let (rect,_)=ui.allocate_exact_size(vec2(295.0,150.0),Sense::hover());
                ui.painter().rect_filled(rect,4.0,t.panel);
                ui.painter().rect_stroke(rect,4.0,Stroke::new(1.0,t.border),StrokeKind::Inside);
                let (Some(min),Some(max))=(
                    xy(&view["bounds"]["min_xy_mm"]),
                    xy(&view["bounds"]["max_xy_mm"])
                ) else {return};
                let (w,h)=(max[0]-min[0],max[1]-min[1]);
                if w<=0.0 || h<=0.0 {return;}
                let scale=((rect.width()-32.0)/w).min((rect.height()-35.0)/h).max(0.001);
                let centre=rect.center();
                let map=|p:[f32;2]| pos2(
                    centre.x+(p[0]-(min[0]+max[0])*0.5)*scale,
                    centre.y-(p[1]-(min[1]+max[1])*0.5)*scale
                );
                ui.painter().rect_stroke(Rect::from_min_max(
                    map([min[0],max[1]]),map([max[0],min[1]])),0.0,
                    Stroke::new(0.8,t.text_dim),StrokeKind::Inside);
                if let Some(paths)=view["paths"].as_array() {
                    for path in paths {
                        let Some(raw)=path["points_xy_mm"].as_array() else {continue};
                        let points:Vec<Pos2>=raw.iter().filter_map(xy).map(map).collect();
                        if points.len()>=2 && points.len()<=1024 {
                            ui.painter().add(egui::Shape::line(points,Stroke::new(1.7,t.accent)));
                        }
                    }
                }
                ui.painter().text(pos2(rect.center().x,rect.bottom()-8.0),
                    Align2::CENTER_CENTER,format!("{w:.2} × {h:.2} mm"),
                    FontId::proportional(11.0),t.text_dim);
            });
        }
    });
}

/// Show measured cylinder Ø and XY offsets. Users may select dimensions
/// only when a cylindrical face has a verified, unambiguous historical name.
fn render_cylinder_measurements(
    ui: &mut egui::Ui, drawing: &Value, can_add: bool,
) -> Option<(String,String)> {
    let mut request = None;
    let Some(values) = drawing["geometry"]["cylinder_dimensions"].as_array() else { return None };
    if values.is_empty() { return None; }
    ui.separator();
    ui.label(RichText::new("Aus 3D gemessene Zylindermaße (vorläufig)").strong());
    ui.label(RichText::new("X ab linker Hüllrahmenkante, Y ab unterer Hüllrahmenkante. Setzbare Maße sind mit einer eindeutig bestätigten 3D-Flächenreferenz verknüpft; Zylinder sind nicht automatisch als Bohrungen klassifiziert.").small());
    egui::ScrollArea::vertical().max_height(145.0).show(ui, |ui| {
        egui::Grid::new("drawing_measured_cylinders").striped(true).show(ui, |ui| {
            for title in ["Nr.", "Durchmesser", "X", "Y", "Bemaßung"] {
                ui.strong(title);
            }
            ui.end_row();
            for (i, item) in values.iter().enumerate() {
                let (Some(d),Some(x),Some(y)) = (
                    item["diameter_mm"].as_f64(),
                    item["x_from_left_mm"].as_f64(),
                    item["y_from_bottom_mm"].as_f64(),
                ) else { continue };
                ui.label((i+1).to_string());
                ui.label(format!("Ø {d:.2} mm"));
                ui.label(format!("{x:.2} mm"));
                ui.label(format!("{y:.2} mm"));
                if can_add {
                    if let Some(face) = item["eligible_face_name"].as_str() {
                        for (kind,caption) in [
                            ("diameter","Ø setzen"),("x_from_left","X setzen"),
                            ("y_from_bottom","Y setzen")
                        ] {
                            let already = drawing["annotations"].as_array().is_some_and(|list|
                                list.iter().any(|a| a["face_name"].as_str() == Some(face)
                                    && a["kind"].as_str() == Some(kind)));
                            if !already && ui.button(caption).clicked() {
                                request = Some((face.to_string(),kind.to_string()));
                            }
                        }
                    } else {
                        ui.label("Referenz nicht eindeutig");
                    }
                } else {
                    ui.label("Zeichnung zuerst speichern / aktualisieren");
                }
                ui.end_row();
            }
        });
    });
    request
}

/// Render a visible window and a separate explicit stale-model confirmation.
pub fn show(app: &mut SolveApp, ctx: &egui::Context) {
    let mut state=std::mem::take(&mut app.drawing_ui);
    if !state.open { return; }
    if state.checked_revision != Some(app.session.revision) {
        refresh(app,&mut state);
    }
    let t=Tokens::get();
    let mut window_open=true;
    let mut request_save:Option<Value>=None;
    let mut request_layout_save=false;
    let mut request_annotation:Option<(String,String)>=None;
    let mut request_second_face:Option<String>=None;
    let names=get_body_names(app);
    crate::frame::window(ctx, "2D-Zeichnung (Testversion)", crate::frame::Width::Wide)
        .id(egui::Id::new("sc_drawing_preview"))
        .open(&mut window_open).resizable(true).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("3D-Körper:");
            let mut chosen=state.body.clone();
            egui::ComboBox::from_id_salt("sc_drawing_body")
                .selected_text(&state.body)
                .show_ui(ui, |ui| {
                    for n in &names {
                        ui.selectable_value(&mut chosen,n.clone(),n);
                    }
                });
            if chosen != state.body {
                state.selected_face = None;
                state.distance_anchor = None;
                state.body=chosen;
                state.snapshot=app.session.doc.metadata.get(KEY)
                    .filter(|v| v["body"].as_str() == Some(state.body.as_str())).cloned();
                state.dismissed_revision=None;
                refresh(app,&mut state);
            }
        });
        ui.label(RichText::new("Experimentelle Draufsicht plus Vorder-/Seitenansicht aus 3D-Kanten; keine vollständige Fertigungsprojektion.").small().color(t.text_dim));
        ui.separator();

        if let Some(err) = &state.error {
            ui.colored_label(t.warning,err);
        }
        if state.proposed.is_some() {
            ui.colored_label(t.warning,"Das 3D-Modell wurde geändert. Diese Zeichnung zeigt noch den alten Stand.");
        } else if state.snapshot.is_some() {
            ui.colored_label(t.accent,"Zeichnung gespeichert und mit dem 3D-Modell abgeglichen.");
        } else {
            ui.label("Noch kein Zeichnungsblatt gespeichert.");
        }
        // The saved snapshot owns manual label placements. A recomputed draft
        // must NOT replace it each frame: otherwise a drag is reverted before
        // its position is committed to document metadata. Older snapshots can
        // still gain the newly introduced read-only measurement table without
        // discarding their manually edited dimension positions.
        let displayed = if let Some(mut saved) = state.snapshot.clone() {
            if state.proposed.is_none()
                && saved["geometry"].get("cylinder_dimensions").is_none()
                && let Some(current) = &state.draft
            {
                saved["geometry"]["cylinder_dimensions"] =
                    current["geometry"]["cylinder_dimensions"].clone();
            }
            Some(saved)
        } else {
            state.draft.clone()
        };
        if let Some(mut drawing) = displayed {
            ui.separator();
            // Edit the label coordinates on a draft. Persist only when drag ends.
            let editable = state.snapshot.is_some() && state.proposed.is_none();
            if state.proposed.is_some() {state.selected_face = None; state.distance_anchor = None;}
            if render_plate(ui,&mut drawing,editable,&mut state.selected_face) {
                state.layout_changed=true;
                request_layout_save=true;
            }
            render_reference_views(ui,&drawing);
            if editable && drawing.get("label_positions") != state.snapshot.as_ref().and_then(|d| d.get("label_positions")) {
                state.snapshot=Some(drawing.clone());
                state.layout_changed=true;
            }
            if editable {
                if let Some(face) = state.selected_face.clone() {
                    let selected= drawing["geometry"]["cylinder_dimensions"].as_array()
                        .and_then(|list| list.iter().find(|d|
                            d["eligible_face_name"].as_str() == Some(face.as_str())));
                    if let Some(selected)=selected {
                        ui.separator();
                        ui.label(RichText::new("Im 2D-Bild ausgewählte 3D-Fläche").strong());
                        let d=selected["diameter_mm"].as_f64().unwrap_or(0.0);
                        ui.label(format!("Zylinder Ø {d:.2} mm – Maßart auswählen:"));
                        ui.horizontal_wrapped(|ui| {
                            for (kind, caption) in [
                                ("diameter","Ø aus Auswahl"),
                                ("x_from_left","X aus Auswahl"),
                                ("y_from_bottom","Y aus Auswahl"),
                            ] {
                                let exists=drawing["annotations"].as_array().is_some_and(|list|
                                    list.iter().any(|a| a["face_name"].as_str() == Some(face.as_str())
                                        && a["kind"].as_str() == Some(kind)));
                                if exists {ui.add_enabled(false,egui::Button::new(format!("{caption} ✓")));}
                                else if ui.button(caption).clicked() {
                                    request_annotation=Some((face.to_string(),kind.to_string()));
                                }
                            }
                            if ui.button("Ersten Mittelpunkt merken").clicked() {
                                state.distance_anchor=Some(face.clone());
                            }
                            if let Some(first) = state.distance_anchor.clone() {
                                let exists=drawing["annotations"].as_array().is_some_and(|list|
                                    list.iter().any(|a| a["kind"] == "center_distance"
                                        && ((a["face_name"].as_str() == Some(first.as_str())
                                            && a["second_face_name"].as_str() == Some(face.as_str()))
                                            || (a["face_name"].as_str() == Some(face.as_str())
                                            && a["second_face_name"].as_str() == Some(first.as_str())))));
                                if ui.add_enabled(first != face && !exists,
                                    egui::Button::new("Mitte-zu-Mitte setzen")).clicked() {
                                    request_annotation=Some((first,"center_distance".to_string()));
                                    request_second_face=Some(face.clone());
                                }
                            }
                            if ui.button("Auswahl aufheben").clicked() {
                                state.distance_anchor=None;
                                state.selected_face=None;
                            }
                        });
                        if let Some(first) = state.distance_anchor.as_deref() {
                            if let Some(center) = drawing["geometry"]["cylinder_dimensions"]
                                .as_array().and_then(|list| list.iter().find(|d|
                                    d["eligible_face_name"].as_str() == Some(first)))
                                .and_then(|d| xy(&d["center_mm"]))
                            {
                                ui.label(format!("Erster Mittelpunkt: X {:.2}, Y {:.2} mm. Zweiten Kreis anklicken.", center[0], center[1]));
                            } else {
                                state.distance_anchor=None;
                            }
                        }
                    } else {
                        state.distance_anchor=None;
                        state.selected_face=None;
                    }
                }
            }
            let table_request = render_cylinder_measurements(ui, &drawing,editable);
            if request_annotation.is_none() { request_annotation=table_request; }
            ui.label(RichText::new("Außenmaße und gesetzte Ø-/X-/Y-/Mitte-zu-Mitte-Maße sind verschiebbar. Bei Verlust der 3D-Flächenreferenz bleibt das Maß sichtbar, aber ungeklärt.").small().color(t.text_dim));
            ui.horizontal(|ui| {
                if state.snapshot.is_none() {
                    if ui.button("Zeichnung im Modell speichern").clicked() {
                        request_save=Some(drawing.clone());
                    }
                } else if state.proposed.is_none() {
                    if ui.button("Zeichnung prüfen").clicked() {
                        refresh(app,&mut state);
                    }
                    if state.layout_changed && ui.button("Maßpositionen speichern").clicked() {
                        request_layout_save=true;
                    }
                } else {
                    ui.label("Aktualisierung ausstehend");
                }
            });
        }
    });
    if let Some(v)=request_save {
        persist(app,&mut state,v);
    } else if request_layout_save {
        save_layout(app,&mut state);
    } else if let Some((face_name,kind)) = request_annotation {
        if let Some(previous) = &state.snapshot {
            let mut params = json!({"body":state.body,"previous":previous,
                "face_name":face_name,"kind":kind});
            if let Some(second) = request_second_face {
                params["second_face_name"] = json!(second);
            }
            match app.session.execute("drawing.annotation.add",&params) {
                Ok(reply) if reply["status"] == "annotated" => {
                    if let Some(drawing) = reply.get("drawing") {
                        persist(app,&mut state,drawing.clone());
                    }
                }
                Ok(_) => state.error=Some("Die Bemaßung konnte nicht erstellt werden.".into()),
                Err(err) => state.error=Some(err.to_string()),
            }
        }
    }

    if state.proposed.is_some() && state.dismissed_revision != Some(app.session.revision) {
        let mut confirm_open=true;
        crate::frame::window(ctx,"Originalzeichnung geändert",crate::frame::Width::Normal)
            .id(egui::Id::new("sc_drawing_stale"))
            .open(&mut confirm_open).show(ctx,|ui|{
                ui.label("Die Maße des 3D-Originalmodells haben sich verändert.");
                ui.label("Soll die gespeicherte 2D-Zeichnung an das Modell angepasst werden?");
                ui.label(RichText::new("Bisherige Maßtextpositionen bleiben erhalten. Nicht eindeutig zuordenbare Bohrungsmaße werden nicht automatisch übertragen.").small().color(t.text_dim));
                ui.separator();
                ui.horizontal(|ui|{
                    if ui.button("Zeichnung aktualisieren").clicked() {
                        accept(app,&mut state);
                    }
                    if ui.button("Alte Zeichnung beibehalten").clicked() {
                        state.dismissed_revision=Some(app.session.revision);
                    }
                });
            });
        if !confirm_open {
            state.dismissed_revision=Some(app.session.revision);
        }
    }

    state.open=window_open;
    app.drawing_ui=state;
}

#[cfg(test)]
mod tests {
    use super::KEY;
    use crate::scenario::{Harness, handle_at};
    use serde_json::json;

    fn drawing_harness() -> Harness {
        let mut h = Harness::new();
        h.app.home.open = false;
        h.app.run("parameters.add", json!({"name":"PanelLength","expression":"90 mm"})).unwrap();
        h.app.run("solid.box", json!({
            "length":"PanelLength", "width":45, "height":8,
            "body_name":"DrawingPlate", "name":"Stock"
        })).unwrap();
        h.frames(3);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.app.drawing_ui.open, "INSPECT / DRAWING opens the real egui window");
        assert!(h.widget("Zeichnung im Modell speichern").is_some(), "save action is visible");
        h
    }

    fn saved_length(h: &Harness) -> f64 {
        h.app.session.doc.metadata.get(KEY).unwrap()["dimensions"][0]["value_mm"]
            .as_f64().unwrap()
    }

    #[test]
    fn native_drawing_window_contains_front_and_right_reference_views() {
        let h=drawing_harness();
        let draft=h.app.drawing_ui.draft.as_ref().unwrap();
        assert_eq!(draft["geometry"]["reference_views"]["front"]["bounds"]
            ["max_xy_mm"],json!([90.0,8.0]));
        assert_eq!(draft["geometry"]["reference_views"]["right"]["bounds"]
            ["max_xy_mm"],json!([45.0,8.0]));
        assert!(h.widget("Vorderansicht (XZ, Y min) · vorläufig").is_some());
        assert!(h.widget("Seitenansicht (YZ, X max) · vorläufig").is_some());
    }

    #[test]
    fn deferred_model_update_keeps_saved_front_and_side_views() {
        let mut h=drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        let original=h.app.session.doc.metadata.get(KEY).unwrap().clone();
        h.app.run("parameters.change",json!({
            "name":"PanelLength","expression":"120 mm"
        })).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&original);
        assert_eq!(h.app.drawing_ui.proposed.as_ref().unwrap()
            ["geometry"]["reference_views"]["front"]["bounds"]["max_xy_mm"],
            json!([120.0,8.0]));
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        h.frames(3);
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["geometry"]["reference_views"]["front"]["bounds"]["max_xy_mm"],json!([120.0,8.0]));
    }

    #[test]
    fn upstream_m2_closed_sheet_reopen_defers_three_view_change_until_clicked() {
        let mut h=drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        let initial=h.app.session.doc.metadata.get(KEY).unwrap().clone();
        assert_eq!(initial["geometry"]["reference_views"]["front"]["bounds"]
            ["max_xy_mm"][0].as_f64(),Some(90.0));
        h.app.drawing_ui.open=false;
        h.frames(2);
        h.app.run("parameters.change",json!({"name":"PanelLength","expression":"126 mm"})).unwrap();
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&initial);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        assert_eq!(h.app.drawing_ui.snapshot.as_ref().unwrap(),&initial);
        assert_eq!(h.app.drawing_ui.proposed.as_ref().unwrap()
            ["geometry"]["reference_views"]["front"]["bounds"]["max_xy_mm"][0].as_f64(),Some(126.0));
        h.step(&json!({"widget":"Alte Zeichnung beibehalten"})).unwrap();
        h.frames(2);
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&initial);
        h.app.drawing_ui.open=false;
        h.frames(2);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        h.frames(3);
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["geometry"]["reference_views"]["front"]["bounds"]["max_xy_mm"][0].as_f64(),Some(126.0));
    }

    #[test]
    fn upstream_m2_two_bores_2d_clicks_select_separate_real_faces() {
        let mut h=drawing_harness();
        for (name,x,y) in [("BoreLeft",18,18),("BoreRight",70,30)] {
            h.app.run("solid.hole",json!({
                "name":name,"position":[x,y,8],"diameter":8
            })).unwrap();
        }
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(2);
        let before=h.app.session.doc.metadata.get(KEY).unwrap().clone();
        assert_eq!(before["geometry"]["cylinder_dimensions"].as_array().unwrap().len(),2);
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        let first=h.app.drawing_ui.selected_face.clone().unwrap();
        h.step(&json!({"widget":"Ersten Mittelpunkt merken"})).unwrap();
        h.frames(2);
        h.step(&json!({"click":{"handle":"drawing:cylinder:1"}})).unwrap();
        h.frames(2);
        let second=h.app.drawing_ui.selected_face.clone().unwrap();
        assert_ne!(first,second,"two similar circles must not use the same historical 3D face");
        assert!(h.widget("Mitte-zu-Mitte setzen").is_some());
        h.step(&json!({"widget":"Mitte-zu-Mitte setzen"})).unwrap();
        h.frames(3);
        let saved=h.app.session.doc.metadata.get(KEY).unwrap();
        assert_eq!(saved["annotations"].as_array().unwrap().len(),1);
        let pair=&saved["annotations"][0];
        assert_eq!(pair["kind"],"center_distance");
        assert_eq!(pair["face_name"],first);
        assert_eq!(pair["second_face_name"],second);
        let actual=(52.0_f64.hypot(12.0)*1000.0).round()/1000.0;
        assert_eq!(pair["value_mm"].as_f64(),Some(actual));
        assert_eq!(pair["reference_status"],"resolved");
    }

    #[test]
    fn unchanged_saved_upper_contour_reopens_without_warning() {
        let mut h = drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        let saved = h.app.session.doc.metadata.get(KEY).unwrap().clone();
        assert_eq!(saved["geometry"]["top_edge_candidates"].as_array().unwrap().len(),4);
        h.app.drawing_ui.open = false;
        h.frames(2);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.app.drawing_ui.open);
        assert!(h.widget("Zeichnung aktualisieren").is_none(),
            "unchanged real 3D contour must not create a false revision popup");
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&saved);
    }

    #[test]
    fn declining_contour_only_edit_preserves_sheet_until_reopening() {
        let mut h = drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        let saved = h.app.session.doc.metadata.get(KEY).unwrap().clone();
        let edges=h.app.run("model.edges",json!({"body":"DrawingPlate"})).unwrap();
        let corner=edges["edges"].as_array().unwrap().iter().find(|e| {
            e["length"].as_f64().is_some_and(|l|(l-8.0).abs()<0.01) &&
            e["mid"][0].as_f64().is_some_and(|x|x.abs()<0.01) &&
            e["mid"][1].as_f64().is_some_and(|y|y.abs()<0.01)
        }).unwrap();
        h.app.run("solid.chamfer",json!({
            "edges":[corner["mid"].clone()],"distance":2.5,"name":"ContourOnly"
        })).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        h.step(&json!({"widget":"Alte Zeichnung beibehalten"})).unwrap();
        h.frames(2);
        assert!(h.widget("Zeichnung aktualisieren").is_none(),
            "declining a contour-only revision must suppress repeated prompts");
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&saved);
        h.app.drawing_ui.open = false;
        h.frames(2);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some(),
            "reopening a stale contour must allow deliberate acceptance");
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&saved);
    }

    #[test]
    fn real_gui_displays_sampled_top_edge_paths_from_3d() {
        let mut h=drawing_harness();
        let draft=h.app.drawing_ui.draft.as_ref().unwrap();
        assert_eq!(draft["geometry"]["top_edge_candidates"].as_array().unwrap().len(),4);
        assert_eq!(draft["accuracy"],"bbox_envelope_with_unclassified_upper_edge_samples");
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        assert!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["geometry"]["top_edge_candidates"].is_array());
    }

    #[test]
    fn gui_contour_only_model_edit_opens_update_confirmation() {
        let mut h=drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        let first=h.app.session.doc.metadata.get(KEY).unwrap().clone();
        let edges=h.app.run("model.edges",json!({"body":"DrawingPlate"})).unwrap();
        let vertical=edges["edges"].as_array().unwrap().iter().find(|e| {
            e["length"].as_f64().is_some_and(|l|(l-8.0).abs()<0.01) &&
            e["mid"][0].as_f64().is_some_and(|x|x.abs()<0.01) &&
            e["mid"][1].as_f64().is_some_and(|y|y.abs()<0.01)
        }).unwrap();
        h.app.run("solid.chamfer",json!({
            "edges":[vertical["mid"].clone()],"distance":2.5,"name":"CornerRelief"
        })).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&first);
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_none());
        assert_ne!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["geometry"]["top_edge_candidates"],first["geometry"]["top_edge_candidates"]);
    }

    #[test]
    fn real_drawing_window_keeps_or_updates_model_changes_by_click() {
        let mut h = drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        assert_eq!(saved_length(&h), 90.0);
        assert!(h.app.drawing_ui.proposed.is_none());

        h.app.run("parameters.change", json!({
            "name":"PanelLength", "expression":"115 mm"
        })).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some(),
            "actual German stale-model dialog appears after a model edit");
        assert!(h.widget("Alte Zeichnung beibehalten").is_some(),
            "user can explicitly decline an update");
        assert_eq!(saved_length(&h), 90.0, "old drawing remains unchanged until user confirms");

        h.step(&json!({"widget":"Alte Zeichnung beibehalten"})).unwrap();
        h.frames(2);
        assert_eq!(saved_length(&h), 90.0,
            "declining does not modify stored dimensions");
        assert!(h.widget("Zeichnung aktualisieren").is_none(),
            "the declined popup does not immediately reappear in the same revision");

        // Reopening explicitly rechecks the source and asks again.
        h.app.drawing_ui.open = false;
        h.frames(2);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some(),
            "reopening an outdated drawing offers an update");
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        h.frames(3);
        assert_eq!(saved_length(&h), 115.0);
        assert!(h.app.drawing_ui.proposed.is_none());
        assert!(h.widget("Zeichnung aktualisieren").is_none(),
            "there is no warning after accepting and saving");
    }

    #[test]
    fn drag_overall_dimension_in_real_egui_window_saves_new_label_coordinates() {
        let mut h = drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(3);
        let old = h.app.session.doc.metadata.get(KEY).unwrap()
            ["label_positions"]["overall_x"].clone();
        let at = handle_at("drawing:label:overall_x")
            .expect("the overall X dimension has a real mouse target");
        let reply = h.call("ui.drag", json!({
            "x0":at.x, "y0":at.y, "x1":at.x+36.0, "y1":at.y-12.0, "steps":12
        }));
        assert_eq!(reply["ok"], true, "{reply}");
        h.frames(3);
        let new = &h.app.session.doc.metadata.get(KEY).unwrap()
            ["label_positions"]["overall_x"];
        assert_ne!(new, &old,
            "dragging a measure text must persist an actually changed position");
        assert!(new[0].as_f64().unwrap() > old[0].as_f64().unwrap(),
            "moving label right increases its X coordinate");
        assert_eq!(saved_length(&h), 90.0, "moving only a label never changes the part");
        assert!(h.app.drawing_ui.proposed.is_none());
    }
    #[test]
    fn undo_and_redo_model_length_toggle_drawing_warning_correctly() {
        let mut h = drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.app.run("parameters.change", json!({"name":"PanelLength","expression":"120 mm"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        assert_eq!(saved_length(&h), 90.0);
        h.app.run("edit.undo", json!({})).unwrap();
        h.frames(3);
        assert_eq!(saved_length(&h), 90.0);
        assert!(h.widget("Zeichnung aktualisieren").is_none(),
            "undo back to the stored geometry must dismiss the stale notice");
        h.app.run("edit.redo", json!({})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some(),
            "redo of a geometric edit must warn again");
        assert_eq!(saved_length(&h), 90.0);
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        assert_eq!(saved_length(&h), 120.0);
    }

    #[test]
    fn moving_y_dimension_preserves_placement_after_closing_and_reopening() {
        let mut h = drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(3);
        let initial = h.app.session.doc.metadata.get(KEY).unwrap()["label_positions"]["overall_y"].clone();
        let at = handle_at("drawing:label:overall_y").expect("Y dimension must expose a drag handle");
        let reply = h.call("ui.drag", json!({
            "x0":at.x,"y0":at.y,"x1":at.x-30.0,"y1":at.y+16.0,"steps":12
        }));
        assert_eq!(reply["ok"], true, "{reply}");
        h.frames(4);
        let moved = h.app.session.doc.metadata.get(KEY).unwrap()["label_positions"]["overall_y"].clone();
        assert_ne!(moved, initial);
        assert!(moved[0].as_f64().unwrap() < initial[0].as_f64().unwrap(),
            "dragging left must decrease stored X coordinate");
        h.app.drawing_ui.open = false;
        h.frames(3);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        let reopened = h.app.session.doc.metadata.get(KEY).unwrap()["label_positions"]["overall_y"].clone();
        assert_eq!(reopened, moved, "the position survives closing and reopening");
        assert_eq!(saved_length(&h), 90.0);
        assert!(h.widget("Zeichnung aktualisieren").is_none());
    }

    #[test]
    fn empty_design_shows_an_error_but_still_allows_a_future_drawing() {
        let mut h = Harness::new();
        h.app.home.open = false;
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.app.drawing_ui.open);
        assert!(h.widget("Kein 3D-Körper vorhanden.").is_some(),
            "empty designs must show a user-friendly explanation");
        h.app.run("solid.box", json!({"length":50,"width":35,"height":4,
            "body_name":"FreshStock"})).unwrap();
        h.frames(2);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung im Modell speichern").is_some(),
            "the drawing can be created once a valid part exists");
    }

    #[test]
    fn new_design_clears_the_transient_drawing_window_and_saved_snapshot() {
        let mut h = drawing_harness();
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        assert!(h.app.session.doc.metadata.get(KEY).is_some());
        h.app.run("file.new", json!({"name":"Another design"})).unwrap();
        h.frames(3);
        assert!(!h.app.drawing_ui.open,
            "the old drawing window must not remain open for another document");
        assert!(h.app.session.doc.metadata.get(KEY).is_none(),
            "a brand-new design must not inherit an old sheet");
    }

    #[test]
    fn measured_diameter_and_xy_offsets_are_visible_in_the_drawing_window() {
        let mut h = drawing_harness();
        h.app.run("solid.hole", json!({"name":"FirstDrill",
            "position":[15,12,8],"diameter":6})).unwrap();
        h.frames(3);
        assert!(h.widget("Aus 3D gemessene Zylindermaße (vorläufig)").is_some(),
            "the new measured geometry table must be visible in real egui");
        assert!(h.widget("Ø 6.00 mm").is_some(),
            "a diameter measured from the real 3D cylinder must reach the drawing");
        assert!(h.widget("15.00 mm").is_some(), "X-offset of the centre is visible");
        assert!(h.widget("12.00 mm").is_some(), "Y-offset of the centre is visible");
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(3);
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["geometry"]["cylinder_dimensions"][0]["diameter_mm"].as_f64(),Some(6.0));
        assert!(h.widget("Ø 6.00 mm").is_some(),
            "3D-derived diameter stays visible after saving the drawing");
    }

    #[test]
    fn added_cylinder_requires_confirmation_before_new_2d_diameter_appears() {
        let mut h = drawing_harness();
        h.app.run("solid.hole", json!({"name":"FirstDrill",
            "position":[15,12,8],"diameter":6})).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(2);
        assert!(h.widget("Ø 6.00 mm").is_some());
        h.app.run("solid.hole", json!({"name":"SecondDrill",
            "position":[65,35,8],"diameter":10})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some(),
            "new 3D cylinder triggers drawing revision confirmation");
        assert!(h.widget("Ø 6.00 mm").is_some(),
            "old cylinder dimensions remain while confirmation is outstanding");
        assert!(h.widget("Ø 10.00 mm").is_none(),
            "new diameter must not silently appear in the saved old drawing");
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        h.frames(3);
        assert!(h.widget("Ø 10.00 mm").is_some(),
            "confirmed model revision transfers second measured diameter");
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["geometry"]["cylinder_dimensions"].as_array().unwrap().len(),2);
    }

    #[test]
    fn older_saved_drawing_displays_new_measurements_without_losing_dragged_labels() {
        let mut h = drawing_harness();
        h.app.run("solid.hole", json!({
            "name":"OlderFileDrill", "position":[18,14,8], "diameter":6
        })).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        let before = h.app.session.doc.metadata.get(KEY).unwrap()
            ["label_positions"]["overall_x"].clone();
        let mut older = h.app.session.doc.metadata.get(KEY).unwrap().clone();
        older["geometry"].as_object_mut().unwrap()
            .remove("cylinder_dimensions");
        h.app.run("document.metadata.set", json!({
            "key":KEY,"value":older
        })).unwrap();
        h.app.drawing_ui.open = false;
        h.frames(3);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.widget("Ø 6.00 mm").is_some(),
            "an older stored drawing shows newly recomputed read-only dimensions");
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["label_positions"]["overall_x"],before,
            "reopening a prior-format snapshot retains its previous manual placement");
        assert!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["geometry"]["cylinder_dimensions"].is_null(),
            "merely displaying a current older snapshot must not rewrite stored data");
        let at = handle_at("drawing:label:overall_x")
            .expect("older saved drawings still have a draggable X dimension");
        let reply = h.call("ui.drag", json!({
            "x0":at.x,"y0":at.y,"x1":at.x+30.0,"y1":at.y-9.0,"steps":12
        }));
        assert_eq!(reply["ok"],true,"{reply}");
        h.frames(3);
        let revised=h.app.session.doc.metadata.get(KEY).unwrap();
        assert_ne!(revised["label_positions"]["overall_x"],before,
            "dragging the migrated old drawing must still persist new coordinates");
        assert!(revised["geometry"]["cylinder_dimensions"].is_array(),
            "saving the dragged drawing safely upgrades its measured 3D geometry table");
    }

    #[test]
    fn real_drawing_window_adds_3d_verified_diameter_dimension_and_drags_its_label() {
        let mut h = drawing_harness();
        h.app.run("solid.hole",json!({
            "name":"VerifiedHole","position":[16,14,8],"diameter":8
        })).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(3);
        assert!(h.widget("Ø setzen").is_some(),
            "the real drawing table offers the diameter of a history-verified 3D cylinder");
        h.step(&json!({"widget":"Ø setzen"})).unwrap();
        h.frames(3);
        let saved=h.app.session.doc.metadata.get(KEY).unwrap();
        let notes=saved["annotations"].as_array().unwrap();
        assert_eq!(notes.len(),1);
        assert_eq!(notes[0]["reference_status"],"resolved");
        assert_eq!(notes[0]["value_mm"].as_f64(),Some(8.0));
        let id=notes[0]["id"].as_str().unwrap().to_string();
        let old=saved["label_positions"][id.as_str()].clone();
        let expected_face=notes[0]["face_name"].clone();
        let at=handle_at(&format!("drawing:label:{id}"))
            .expect("the linked diameter has a draggable screen target");
        let r=h.call("ui.drag",json!({
            "x0":at.x,"y0":at.y,"x1":at.x+24.0,"y1":at.y-12.0,"steps":12
        }));
        assert_eq!(r["ok"],true,"{r}");
        h.frames(4);
        let updated=h.app.session.doc.metadata.get(KEY).unwrap();
        assert_ne!(updated["label_positions"][id.as_str()],old);
        assert_eq!(updated["annotations"][0]["face_name"],expected_face);
    }

    #[test]
    fn linked_3d_dimension_requires_user_confirmation_before_changing_value() {
        let mut h=drawing_harness();
        h.app.run("solid.hole",json!({
            "name":"NamedHole","position":[16,14,8],"diameter":6
        })).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(2);
        assert!(h.widget("Ø setzen").is_some());
        h.step(&json!({"widget":"Ø setzen"})).unwrap();
        h.frames(3);
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["annotations"][0]["value_mm"].as_f64(),Some(6.0));
        h.app.run("solid.hole",json!({
            "name":"AnotherHole","position":[70,30,8],"diameter":10
        })).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["annotations"][0]["value_mm"].as_f64(),Some(6.0));
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        h.frames(3);
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["annotations"][0]["reference_status"],"resolved");
        assert!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["annotations"][0]["face_name"].as_str().is_some());
    }

    #[test]
    fn clicking_2d_cylinder_selects_verified_face_and_adds_diameter_dimension() {
        let mut h=drawing_harness();
        h.app.run("solid.hole",json!({
            "name":"DirectlySelectedHole","position":[17,16,8],"diameter":8
        })).unwrap();
        h.frames(3);
        assert!(handle_at("drawing:cylinder:0").is_some(),
            "the actual drawing must expose a real mouse hit target");
        assert!(h.app.drawing_ui.selected_face.is_none());
        // Before the drawing is saved a circle cannot be used for editing.
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        assert!(h.app.drawing_ui.selected_face.is_none(),
            "unsaved drawing is deliberately read-only");
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(3);
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        assert!(h.app.drawing_ui.selected_face.is_some(),
            "real click directly on the projected circle selects its exact 3D face");
        assert!(h.widget("Ø aus Auswahl").is_some());
        h.step(&json!({"widget":"Ø aus Auswahl"})).unwrap();
        h.frames(3);
        let saved=h.app.session.doc.metadata.get(KEY).unwrap();
        assert_eq!(saved["annotations"].as_array().unwrap().len(),1);
        assert_eq!(saved["annotations"][0]["kind"],"diameter");
        assert_eq!(saved["annotations"][0]["reference_status"],"resolved");
        assert_eq!(saved["annotations"][0]["value_mm"].as_f64(),Some(8.0));
        assert_eq!(saved["annotations"][0]["face_name"].as_str(),
            h.app.drawing_ui.selected_face.as_deref());
        assert!(h.widget("X aus Auswahl").is_some(),
            "the same clicked face can be used for another independent dimension");
    }

    #[test]
    fn direct_2d_selection_chooses_exact_hole_not_neighbor_and_three_dimension_types() {
        let mut h=drawing_harness();
        for (name,x,y,diameter) in [
            ("Smaller",17,12,6),("Larger",71,33,10)
        ] {
            h.app.run("solid.hole",json!({
                "name":name,"position":[x,y,8],"diameter":diameter
            })).unwrap();
        }
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(2);
        h.step(&json!({"click":{"handle":"drawing:cylinder:1"}})).unwrap();
        h.frames(2);
        let selected=h.app.drawing_ui.selected_face.clone().unwrap();
        let model=h.app.session.doc.metadata.get(KEY).unwrap();
        let second=model["geometry"]["cylinder_dimensions"][1]["eligible_face_name"].as_str().unwrap();
        assert_eq!(selected,second);
        h.step(&json!({"widget":"Ø aus Auswahl"})).unwrap();
        h.frames(2);
        h.step(&json!({"widget":"X aus Auswahl"})).unwrap();
        h.frames(2);
        h.step(&json!({"widget":"Y aus Auswahl"})).unwrap();
        h.frames(3);
        let saved=h.app.session.doc.metadata.get(KEY).unwrap();
        let labels=saved["annotations"].as_array().unwrap();
        assert_eq!(labels.len(),3);
        assert!(labels.iter().all(|a|a["face_name"].as_str()==Some(selected.as_str())));
        for (kind,val) in [("diameter",10.0),("x_from_left",71.0),("y_from_bottom",33.0)] {
            let found=labels.iter().find(|a|a["kind"].as_str()==Some(kind)).unwrap();
            assert_eq!(found["value_mm"].as_f64(),Some(val));
        }
        assert!(h.widget("Ø aus Auswahl ✓").is_some(),
            "duplicate dimension selection visibly disabled");
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        assert_ne!(h.app.drawing_ui.selected_face.as_deref(),Some(selected.as_str()),
            "the other circle has its own distinct history-verified source");
    }

    #[test]
    fn unsaved_or_stale_drawing_cannot_select_a_new_2d_dimension() {
        let mut h=drawing_harness();
        h.app.run("solid.hole",json!({
            "name":"First","position":[18,12,8],"diameter":6
        })).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(2);
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        assert!(h.app.drawing_ui.selected_face.is_some());
        h.app.run("solid.hole",json!({
            "name":"Second","position":[64,31,8],"diameter":10
        })).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        assert!(h.app.drawing_ui.selected_face.is_none(),
            "selected 3D reference cleared while an old drawing is being displayed");
        assert!(h.widget("Ø aus Auswahl").is_none(),
            "no annotation may be inserted before confirming a model revision");
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        assert!(h.app.drawing_ui.selected_face.is_none());
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()
            ["annotations"].as_array().unwrap().len(),0);
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        h.frames(3);
        h.step(&json!({"click":{"handle":"drawing:cylinder:1"}})).unwrap();
        h.frames(2);
        assert!(h.app.drawing_ui.selected_face.is_some(),
            "click selection works again after explicit drawing update");
    }

    #[test]
    fn direct_2d_selection_can_be_cleared_and_is_not_saved_as_drawing_metadata() {
        let mut h=drawing_harness();
        h.app.run("solid.hole",json!({
            "name":"OnlyHole","position":[18,12,8],"diameter":6
        })).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(3);
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        assert!(h.widget("Auswahl aufheben").is_some());
        assert!(h.app.session.doc.metadata.get(KEY).unwrap()
            .get("selected_face").is_none(),"temporary selection never belongs in the CAD file");
        h.step(&json!({"widget":"Auswahl aufheben"})).unwrap();
        h.frames(2);
        assert!(h.app.drawing_ui.selected_face.is_none());
        assert!(h.widget("Ø aus Auswahl").is_none());
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        assert!(h.app.drawing_ui.selected_face.is_some());
        h.app.drawing_ui.open=false;
        h.frames(2);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.app.drawing_ui.selected_face.is_none(),
            "reopening the window must not inherit an accidental user selection");
        assert!(h.widget("Ø aus Auswahl").is_none());
    }

    fn centre_pair_harness() -> Harness {
        let mut h=drawing_harness();
        h.app.run("parameters.add",json!({"name":"PairX","expression":"65 mm"})).unwrap();
        h.app.run("solid.hole",json!({"name":"PairFirst","position":[15,12,8],"diameter":6})).unwrap();
        h.app.run("solid.hole",json!({"name":"PairSecond","position":["PairX",32,8],"diameter":6})).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung im Modell speichern"})).unwrap();
        h.frames(3);
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Ersten Mittelpunkt merken"})).unwrap();
        h.frames(3);
        h.step(&json!({"click":{"handle":"drawing:cylinder:1"}})).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Mitte-zu-Mitte setzen"})).unwrap();
        h.frames(3);
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap()["annotations"][0]["kind"],"center_distance");
        h
    }

    #[test]
    fn centre_distance_updates_only_after_reopening_and_confirming_the_dialog() {
        let mut h=centre_pair_harness();
        let before=h.app.session.doc.metadata.get(KEY).unwrap().clone();
        let id=before["annotations"][0]["id"].as_str().unwrap().to_string();
        let at=handle_at(&format!("drawing:label:{id}")).unwrap();
        let reply=h.call("ui.drag",json!({"x0":at.x,"y0":at.y,"x1":at.x+20.0,"y1":at.y-10.0,"steps":12}));
        assert_eq!(reply["ok"],true);
        h.frames(3);
        let saved=h.app.session.doc.metadata.get(KEY).unwrap().clone();
        assert_ne!(saved["label_positions"][id.as_str()],before["label_positions"][id.as_str()]);
        h.app.drawing_ui.open=false;
        h.frames(2);
        h.app.run("parameters.change",json!({"name":"PairX","expression":"75 mm"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_none(),"3D work must not open a drawing popup");
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&saved);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        assert!(h.widget("Zeichnung aktualisieren").is_some());
        assert!(h.app.drawing_ui.distance_anchor.is_none());
        h.step(&json!({"widget":"Alte Zeichnung beibehalten"})).unwrap();
        h.frames(2);
        assert_eq!(h.app.session.doc.metadata.get(KEY).unwrap(),&saved);
        h.app.drawing_ui.open=false;
        h.frames(2);
        h.step(&json!({"start":"drawing.preview"})).unwrap();
        h.frames(3);
        h.step(&json!({"widget":"Zeichnung aktualisieren"})).unwrap();
        h.frames(3);
        let updated=h.app.session.doc.metadata.get(KEY).unwrap();
        assert!((updated["annotations"][0]["value_mm"].as_f64().unwrap()-60.0_f64.hypot(20.0)).abs()<0.002);
        assert_eq!(updated["annotations"][0]["face_name"],saved["annotations"][0]["face_name"]);
        assert_eq!(updated["annotations"][0]["second_face_name"],saved["annotations"][0]["second_face_name"]);
        assert_eq!(updated["label_positions"][id.as_str()],saved["label_positions"][id.as_str()]);
        assert!(h.widget("Zeichnung aktualisieren").is_none());
    }

    #[test]
    fn centre_distance_anchor_is_cleared_when_selection_is_cleared_or_drawing_stales() {
        let mut h=centre_pair_harness();
        assert!(h.app.drawing_ui.distance_anchor.is_some());
        h.step(&json!({"widget":"Auswahl aufheben"})).unwrap();
        h.frames(2);
        assert!(h.app.drawing_ui.distance_anchor.is_none());
        h.step(&json!({"click":{"handle":"drawing:cylinder:0"}})).unwrap();
        h.frames(2);
        h.step(&json!({"widget":"Ersten Mittelpunkt merken"})).unwrap();
        h.frames(2);
        h.app.run("parameters.change",json!({"name":"PairX","expression":"70 mm"})).unwrap();
        h.frames(3);
        assert!(h.app.drawing_ui.distance_anchor.is_none());
        assert!(h.app.drawing_ui.selected_face.is_none());
        assert!(h.widget("Mitte-zu-Mitte setzen").is_none());
        assert!(h.widget("Zeichnung aktualisieren").is_some());
    }

}
