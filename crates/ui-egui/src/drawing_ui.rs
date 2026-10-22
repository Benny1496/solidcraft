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

fn render_plate(ui: &mut egui::Ui, drawing: &mut Value, editable: bool) -> bool {
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
    ui.painter().rect_stroke(Rect::from_min_max(left_top, right_bottom), 0.0, Stroke::new(1.8, t.text), StrokeKind::Inside);

    if let Some(circles) = drawing["geometry"]["cylinders"].as_array() {
        for c in circles {
            if let (Some(at), Some(d)) = (xy(&c["center_mm"]), c["diameter_mm"].as_f64()) {
                if d.is_finite() && d > 0.0 && d < 1e6 {
                    let p = map(at);
                    ui.painter().circle_stroke(p, d as f32 * scale * 0.5, Stroke::new(1.5, t.text));
                    let cross = 5.0;
                    ui.painter().line_segment([p - vec2(cross,0.0), p + vec2(cross,0.0)], Stroke::new(0.7,t.text_dim));
                    ui.painter().line_segment([p - vec2(0.0,cross), p + vec2(0.0,cross)], Stroke::new(0.7,t.text_dim));
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
    stopped
}

/// Show measured Ø and edge offsets, but intentionally without a persistent
/// bore identifier. This is a read-only measured-geometry table, not an
/// associative manufacturing callout selection.
fn render_cylinder_measurements(ui: &mut egui::Ui, drawing: &Value) {
    let Some(values) = drawing["geometry"]["cylinder_dimensions"].as_array() else { return };
    if values.is_empty() { return; }
    ui.separator();
    ui.label(RichText::new("Aus 3D gemessene Zylindermaße (vorläufig)").strong());
    ui.label(RichText::new("X ab linker Hüllrahmenkante, Y ab unterer Hüllrahmenkante. Zylinder sind noch nicht als Bohrungen zertifiziert; keine dauerhafte Maßreferenz.").small());
    egui::ScrollArea::vertical().max_height(145.0).show(ui, |ui| {
        egui::Grid::new("drawing_measured_cylinders").striped(true).show(ui, |ui| {
            for title in ["Nr.", "Durchmesser", "X", "Y"] {
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
                ui.end_row();
            }
        });
    });
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
                state.body=chosen;
                state.snapshot=app.session.doc.metadata.get(KEY)
                    .filter(|v| v["body"].as_str() == Some(state.body.as_str())).cloned();
                state.dismissed_revision=None;
                refresh(app,&mut state);
            }
        });
        ui.label(RichText::new("Experimentelle Draufsicht: Außenkontur als Rechteck-Hüllrahmen, keine vollständige Fertigungsprojektion.").small().color(t.text_dim));
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
        if let Some(mut drawing) = state.snapshot.clone().or_else(|| state.draft.clone()) {
            ui.separator();
            // Edit the label coordinates on a draft. Persist only when drag ends.
            let editable = state.snapshot.is_some() && state.proposed.is_none();
            if render_plate(ui,&mut drawing,editable) {
                state.layout_changed=true;
                request_layout_save=true;
            }
            if editable && drawing.get("label_positions") != state.snapshot.as_ref().and_then(|d| d.get("label_positions")) {
                state.snapshot=Some(drawing.clone());
                state.layout_changed=true;
            }
            render_cylinder_measurements(ui, &drawing);
            ui.label(RichText::new("Außenmaßtexte sind verschiebbar. Ø- und X/Y-Zylindermaße werden aus der 3D-Geometrie gemessen, aber noch nicht als sichere Fertigungsmaße referenziert.").small().color(t.text_dim));
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

}
