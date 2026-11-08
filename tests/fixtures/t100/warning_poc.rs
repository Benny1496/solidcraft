// Test 100: runner-only egui foreground status-card prototype, never product UI.
// Uses an actual egui::Button in a Foreground Area so the full-page sheet
// drag response cannot swallow status-card clicks.
use std::cell::Cell;

use egui::{Id, Order, Pos2, Vec2};
use serde_json::{Value, json};

use crate::SolveApp;

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static EXPANDED: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn enable() {
    ENABLED.with(|v| v.set(true));
    EXPANDED.with(|v| v.set(false));
}

pub(super) fn disable() {
    ENABLED.with(|v| v.set(false));
    EXPANDED.with(|v| v.set(false));
}

pub(super) fn message(row: &Value) -> Option<Value> {
    let state = row["status"].as_str().unwrap_or("unknown");
    let outdated = row["out_of_date"] == true;
    let (kind, symbol) = match state {
        "lost" => ("Lost", "✕"),
        "ambiguous" => ("Ambiguous", "⚠"),
        _ if outdated => ("Needs update", "⟳"),
        _ => return None,
    };
    let reason = row["why"].as_str().filter(|s| !s.is_empty()).unwrap_or(if outdated {
        "Drawing is not up to date"
    } else {
        "Reference needs attention"
    });
    Some(json!({"dimension":row["dimension"],"kind":kind,"symbol":symbol,"why":reason}))
}

/// The card is both painted and clickable in real headless native egui frames.
/// Metadata reports exactly what was rendered, not a finished product feature.
pub(super) fn show(app: &mut SolveApp, ui: &mut egui::Ui) -> Value {
    if !ENABLED.with(Cell::get) {
        return Value::Null;
    }
    let status = match app.run("drawing.status", json!({})) {
        Ok(reply) => reply,
        Err(err) => return json!({"poc":true,"error":err.to_string()}),
    };
    let findings: Vec<Value> = status["dimensions"]
        .as_array()
        .map_or_else(Vec::new, |rows| rows.iter().filter_map(message).collect());
    let count = findings.len();
    let label = if count == 0 {
        "All current".to_owned()
    } else {
        format!("{count} need attention")
    };
    let bounds = ui.max_rect();
    let top_left = Pos2::new(bounds.center().x - 107.0, bounds.top() + 14.0);
    let mut is_open = EXPANDED.with(Cell::get);

    // The parent sheet uses a full-canvas click/drag response. In T99,
    // ui.interact() was placed into that response and did not get the click.
    // A separate Foreground Area with a standard button fixes hit-testing.
    egui::Area::new(Id::new("t100_ci_only_status_card"))
        .order(Order::Foreground)
        .fixed_pos(top_left)
        .show(ui.ctx(), |overlay| {
            let pill = overlay.add(egui::Button::new(&label).min_size(Vec2::new(214.0, 32.0)));
            crate::scenario::publish_target("t100:status_pill", pill.rect);
            if pill.clicked() {
                is_open = !is_open;
                EXPANDED.with(|v| v.set(is_open));
            }
            if is_open {
                overlay.separator();
                overlay.label("Drawing status");
                for finding in findings.iter().take(8) {
                    let symbol = finding["symbol"].as_str().unwrap_or("?");
                    let kind = finding["kind"].as_str().unwrap_or("?");
                    let dimension = finding["dimension"].as_str().unwrap_or("?");
                    overlay.label(format!("{symbol} {kind} — {dimension}"));
                    let reason = finding["why"].as_str().unwrap_or("Reason unavailable");
                    overlay.label(reason.chars().take(100).collect::<String>());
                }
                crate::scenario::publish_rect("t100:status_details", overlay.min_rect());
            }
        });
    json!({
        "poc":true,
        "count":count,
        "label":label,
        "expanded":is_open,
        "findings":findings,
        "status_source":"drawing.status",
        "interactive_status_pill":true,
        "real_repair_button":false
    })
}
