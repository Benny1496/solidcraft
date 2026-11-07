/// Native egui frames (not browser screenshots) prove exactly what a person
/// can currently see in the Drawing workspace at this pinned commit.
/// An expected UI gap is NOT a successful end-to-end drawing user journey.
#[cfg(test)]
mod t94_drawing_user_acceptance {
    use super::*;
    use crate::scenario::Harness;
    use egui::Shape;
    use serde_json::json;
    use std::sync::Arc;

    fn rendered_text(h: &Harness) -> Vec<String> {
        let mut labels = Vec::new();
        for clipped in &h.shapes {
            crate::test_support::each_shape(&clipped.shape, &mut |shape| {
                if let Shape::Text(t) = shape {
                    labels.push(t.galley.text().to_owned());
                }
            });
        }
        labels
    }

    fn record(stage: &str, items: &[String]) {
        let path = std::env::var("SOLIDCRAFT_T94_UI_EVIDENCE_DIR")
            .expect("UI evidence directory required");
        std::fs::create_dir_all(&path).unwrap();
        let path = std::path::Path::new(&path).join(format!("{stage}.json"));
        std::fs::write(path, serde_json::to_vec_pretty(&json!({"stage":stage,"rendered_egui_text":items})).unwrap()).unwrap();
    }

    fn contains(items: &[String], needle: &str) -> bool {
        items.iter().any(|s| s.contains(needle))
    }

    fn drawing_workspace(h: &mut Harness) -> Vec<String> {
        h.app.preferences.language = "en".into();
        h.app.home.open = false;
        enter(&mut h.app, true).unwrap();
        h.frames(5);
        assert!(active(&h.app));
        rendered_text(h)
    }

    #[test]
    fn t94_real_gui_shows_placeholder_despite_an_existing_engine_drawing_document() {
        let mut h = Harness::new();
        h.app.run("solid.box", json!({"length":100,"width":80,"height":20,"body_name":"Plate"})).unwrap();
        h.app.run("drawing.create", json!({})).unwrap();
        assert!(h.app.session.drawing("test").is_ok(), "real engine drawing created");
        let before = h.app.session.doc.to_json();
        let shown = drawing_workspace(&mut h);
        record("01-drawing-with-existing-document", &shown);
        assert!(contains(&shown, "No drawing sheet is available yet."), "actual GUI should show the placeholder: {shown:?}");
        assert!(contains(&shown, "Drawing files and the paper renderer are not implemented"), "actual UI explicitly describes unimplemented renderer: {shown:?}");
        assert!(crate::scenario::rect_of("drawing:sheets").is_some(), "sheet strip belongs to the UI");
        assert_eq!(h.app.session.doc.to_json(), before, "opening empty GUI must not mutate a real drawing");
        println!("T94_GUI_1=CONFIRMED_GAP; ENGINE_DRAWING_EXISTS=YES; UI_SHOWS_SHEET=NO; UI_SHOWS_PLACEHOLDER=YES");
    }

    #[test]
    fn t94_real_user_dimension_tool_opens_only_a_disabled_placeholder_panel() {
        let mut h = Harness::new();
        let first = drawing_workspace(&mut h);
        assert!(contains(&first, "Drawing workspace"), "entered UI {first:?}");
        let before = h.app.session.doc.to_json();
        // Same control path as the real toolbar tool starter.
        h.step(&json!({"start":"drawing.dimension"})).unwrap();
        h.frames(4);
        let shown = rendered_text(&h);
        record("02-dimension-option-panel", &shown);
        assert_eq!(h.app.drawing_workspace.options, Some("Dimension"));
        assert!(contains(&shown, "No drawing geometry available"), "drawing Dimension can't pick model references: {shown:?}");
        assert!(contains(&shown, "Apply (not implemented)"), "the user cannot apply the dimension: {shown:?}");
        assert_eq!(h.app.session.doc.to_json(), before, "opening an inactive tool must not fake a drawing dimension");
        println!("T94_GUI_2=CONFIRMED_GAP; USER_OPENED_DIMENSION_TOOL=YES; APPLY_ENABLED=NO; DIMENSION_NOT_USABLE=YES");
    }

    #[test]
    fn t94_a_real_lost_engine_dimension_is_not_shown_as_a_user_warning_on_the_drawing_page() {
        use solidcraft_engine::doc::drawing::view::{View, ViewKind, ViewPlace};
        use solidcraft_engine::doc::drawing_view::{ViewDirection, ViewOverrides};
        use solidcraft_engine::doc::ids::{self, IdSource};
        use solidcraft_engine::geom::drawing_view::NamedView;

        let mut h = Harness::new();
        h.app.run("solid.box", json!({"length":100,"width":80,"height":20,"body_name":"Plate"})).unwrap();
        h.app.run("drawing.create", json!({})).unwrap();
        let mut drawing = (**h.app.session.drawing("test").unwrap()).clone();
        let drawing_id = drawing.id;
        let (sheet, source) = (drawing.sheets[0].id, drawing.sources[0].id);
        let mut ids = IdSource::seeded(9400);
        let view = drawing.allocate_id(&mut ids).unwrap();
        drawing.add_views([Arc::new(View {
            id: view,
            sheet,
            kind: ViewKind::Base { source, view: ViewDirection::Named(NamedView::Front) },
            place: ViewPlace::At([140.0, 150.0]),
            settings: ViewOverrides { scale: Some("1:2".parse().unwrap()), ..ViewOverrides::default() },
            reference_shown_percent: 0.0,
        })]);
        h.app.session.doc_mut().put_drawing(Arc::new(drawing));
        let added = h.app.run("drawing.dimension.add", json!({
            "view": ids::encode(view),
            "kind": {"length": {"edge": [50, 0, 20]}}
        })).unwrap();
        let dim = added["dimension"].clone();
        assert!(dim.is_string(), "{added}");
        let initially = h.app.run("drawing.dimension.inspect", json!({"dimension":dim})).unwrap();
        assert_eq!(initially["status"], "ok", "{initially}");

        h.app.run("drawing.close", json!({})).unwrap();
        let feature = h.app.session.doc.features[0].id;
        h.app.run("timeline.delete", json!({"feature":feature.to_string()})).unwrap();
        h.app.run("drawing.open", json!({"drawing":ids::encode(drawing_id)})).unwrap();
        let lost = h.app.run("drawing.dimension.inspect", json!({"dimension":dim})).unwrap();
        assert_eq!(lost["status"], "lost", "engine must establish real lost reference first: {lost}");
        let shown = drawing_workspace(&mut h);
        record("03-existing-lost-dimension-on-drawing-page", &shown);
        assert!(contains(&shown, "No drawing sheet is available yet."), "UI never shows the actual sheet: {shown:?}");
        let lower = shown.join("\n").to_lowercase();
        assert!(!lower.contains("lost reference") && !lower.contains("invalid dimension") && !lower.contains("dimension lost"),
            "the known placeholder must not be mistaken for a working warning UI: {shown:?}");
        println!("T94_GUI_3=CONFIRMED_GAP; BACKEND_DIMENSION_STATUS=LOST; DRAWING_PAGE_SHOWS_WARNING=NO; PAGE_SHOWS_PLACEHOLDER=YES");
    }
}
