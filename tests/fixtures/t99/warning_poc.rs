// T99 proof-of-concept is injected into the runner only, behind cfg(test).
// Not part of Loohi PR #13211 or any production GUI implementation.
// This is an actual egui-painted, clickable status pill for technical feasibility.
use std::cell::Cell;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use serde_json::{json,Value};

use crate::SolveApp;

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static EXPANDED: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn enable() {
    ENABLED.with(|a| a.set(true));
    EXPANDED.with(|a| a.set(false));
}
pub(super) fn disable() {
    ENABLED.with(|a| a.set(false));
    EXPANDED.with(|a| a.set(false));
}

fn message(row: &Value) -> Option<Value> {
    let state = row["status"].as_str().unwrap_or("unknown");
    let outdated = row["out_of_date"] == true;
    let kind = match state {
        "lost" => "Lost",
        "ambiguous" => "Ambiguous",
        _ if outdated => "Needs update",
        _ => return None,
    };
    let icon = match kind { "Lost" => "✕", "Ambiguous" => "⚠", _ => "⟳" };
    let why = row["why"].as_str().filter(|s| !s.is_empty()).unwrap_or_else(|| if outdated { "Drawing is not up to date" } else { "Reference needs attention" });
    Some(json!({"dimension":row["dimension"],"kind":kind,"symbol":icon,"why":why}))
}

/// Returns precisely the visible status metadata of the CI-only prototype.
/// The real egui card is painted and hit-targeted in the same call.
pub(super) fn show(app: &mut SolveApp, ui: &mut egui::Ui) -> Value {
    if !ENABLED.with(Cell::get) { return Value::Null; }
    let status = match app.run("drawing.status", json!({})) {
        Ok(value) => value,
        Err(e) => return json!({"error":e,"poc":true}),
    };
    let findings: Vec<Value> = status["dimensions"].as_array().map_or_else(Vec::new, |rows| rows.iter().filter_map(message).collect());
    let count = findings.len();
    let label = if count == 0 { "All current".to_string() } else { format!("{count} need attention") };
    let area = ui.max_rect();
    // The prototype deliberately sits above the sheet on Loohi's existing canvas;
    // the final D07 placement is Nathan's decision after #13159 lands.
    let top = area.top() + 14.0;
    let pill = Rect::from_min_size(Pos2::new(area.center().x - 104.0, top), Vec2::new(208.0, 32.0));
    let response = ui.interact(pill, egui::Id::new("t99_ci_only_status_pill"), Sense::click());
    if response.clicked() {
        EXPANDED.with(|v| v.set(!v.get()));
    }
    let painter = ui.painter_at(area);
    painter.rect_filled(pill, 6.0, Color32::from_gray(32));
    painter.rect_stroke(pill, 6.0, Stroke::new(1.0, Color32::LIGHT_GRAY), egui::StrokeKind::Inside);
    painter.text(pill.center(), Align2::CENTER_CENTER, &label, FontId::proportional(14.0), Color32::WHITE);
    crate::scenario::publish_target("t99:status_pill", pill);
    let expanded = EXPANDED.with(Cell::get);
    if expanded {
        let card = Rect::from_min_size(pill.left_bottom() + Vec2::new(0.0, 5.0), Vec2::new(440.0, (42 + findings.len().min(6) * 52) as f32));
        painter.rect_filled(card, 6.0, Color32::from_gray(39));
        painter.text(card.min + Vec2::new(12.0, 9.0), Align2::LEFT_TOP, "Drawing status", FontId::proportional(14.0), Color32::WHITE);
        for (index, row) in findings.iter().take(6).enumerate() {
            let id = row["dimension"].as_str().unwrap_or("?");
            let text = format!("{} {}  #{}", row["symbol"].as_str().unwrap_or("?"), row["kind"].as_str().unwrap_or("?"), id);
            let y = 35.0 + index as f32 * 52.0;
            painter.text(card.min + Vec2::new(12.0, y), Align2::LEFT_TOP, text, FontId::proportional(13.0), Color32::WHITE);
            let why = row["why"].as_str().unwrap_or("reason unavailable");
            let short: String = why.chars().take(75).collect();
            painter.text(card.min + Vec2::new(14.0, y + 17.0), Align2::LEFT_TOP, short, FontId::proportional(12.0), Color32::LIGHT_GRAY);
        }
        crate::scenario::publish_rect("t99:status_details", card);
    }
    json!({"poc":true,"count":count,"label":label,"expanded":expanded,"findings":findings,"status_source":"drawing.status","interactive_status_pill":true,"real_repair_button":false})
}
