// SolidCraft T102: temporary native egui repair/update panel for acceptance testing.
// Test-only foreground UI injected into Loohi's unmerged sheet PR. No product UI.
use std::cell::{Cell, RefCell};

use egui::{Id, Order, Pos2, Vec2};
use serde_json::{Value, json};

use crate::SolveApp;

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static EXPANDED: Cell<bool> = const { Cell::new(false) };
    static LAST_ACTION: RefCell<Value> = RefCell::new(Value::Null);
}
pub(super) fn enable() {
    ENABLED.with(|v| v.set(true));
    EXPANDED.with(|v| v.set(false));
    LAST_ACTION.with(|v| *v.borrow_mut() = Value::Null);
}
pub(super) fn disable() {
    ENABLED.with(|v| v.set(false));
    EXPANDED.with(|v| v.set(false));
    LAST_ACTION.with(|v| *v.borrow_mut() = Value::Null);
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
    let reason = row["why"].as_str().filter(|text| !text.is_empty()).unwrap_or(if outdated {
        "Drawing is not up to date"
    } else {
        "Reference needs attention"
    });
    Some(json!({"dimension":row["dimension"],"kind":kind,"symbol":symbol,"why":reason}))
}

/// Proof-of-concept-only, actual clickable buttons. Candidates originate from
/// drawing.dimension.inspect and are handed unchanged to drawing.dimension.repair.
pub(super) fn show(app: &mut SolveApp, ui: &mut egui::Ui) -> Value {
    if !ENABLED.with(Cell::get) { return Value::Null; }
    let status = match app.run("drawing.status", json!({})) {
        Ok(value) => value,
        Err(err) => return json!({"poc":true,"error":err.to_string()}),
    };
    let findings: Vec<Value> = status["dimensions"].as_array().map_or_else(
        Vec::new, |rows| rows.iter().filter_map(message).collect()
    );
    let label = if findings.is_empty() { "All current".to_owned() } else {
        format!("{} need attention", findings.len())
    };
    let bounds = ui.max_rect();
    let at = Pos2::new(bounds.center().x - 107.0, bounds.top() + 14.0);
    let mut expanded = EXPANDED.with(Cell::get);
    let mut pending: Option<(&'static str, Value)> = None;
    let mut repair_buttons = 0;
    let mut update_buttons = 0;

    egui::Area::new(Id::new("t102_ci_only_status_card"))
        .order(Order::Foreground)
        .fixed_pos(at)
        .show(ui.ctx(), |overlay| {
            let pill = overlay.add(egui::Button::new(&label).min_size(Vec2::new(214.0, 32.0)));
            crate::scenario::publish_target("t102:status_pill", pill.rect);
            if pill.clicked() {
                expanded = !expanded;
                EXPANDED.with(|value| value.set(expanded));
            }
            if !expanded { return; }
            overlay.separator();
            overlay.label("Drawing status");
            for (row_number, finding) in findings.iter().take(8).enumerate() {
                let id = finding["dimension"].as_str().unwrap_or("?");
                let kind = finding["kind"].as_str().unwrap_or("?");
                let symbol = finding["symbol"].as_str().unwrap_or("?");
                let why = finding["why"].as_str().unwrap_or("Reason unavailable");
                overlay.label(format!("{symbol} {kind} — {id}"));
                overlay.label(why.chars().take(110).collect::<String>());

                match kind {
                    "Needs update" => {
                        let b = overlay.button("Update drawing");
                        update_buttons += 1;
                        crate::scenario::publish_target("t102:update_drawing", b.rect);
                        if b.clicked() { pending = Some(("drawing.update", json!({}))); }
                    },
                    "Lost" | "Ambiguous" => {
                        if let Ok(details) = app.run("drawing.dimension.inspect", json!({"dimension":id})) {
                            if let Some(candidates) = details["candidates"].as_array() {
                                for (candidate_number, candidate) in candidates.iter().take(6).enumerate() {
                                    // The current proof-of-concept's fixture has a length/edge dimension;
                                    // other dimension kinds need slot selection in the real product UI.
                                    if candidate.get("edge").is_none() { continue; }
                                    let btn = overlay.button(format!("Repair candidate {}", candidate_number + 1));
                                    repair_buttons += 1;
                                    crate::scenario::publish_target(
                                        &format!("t102:repair:{row_number}:{candidate_number}"), btn.rect
                                    );
                                    if btn.clicked() {
                                        pending = Some(("drawing.dimension.repair", json!({
                                            "dimension":id,"slot":"edge","pick":candidate
                                        })));
                                    }
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
            crate::scenario::publish_rect("t102:status_details", overlay.min_rect());
        });

    if let Some((command, params)) = pending {
        let reply = match app.run(command, params.clone()) {
            Ok(value) => json!({"success":true,"command":command,"params":params,"result":value}),
            Err(err) => json!({"success":false,"command":command,"params":params,"error":err.to_string()}),
        };
        LAST_ACTION.with(|saved| *saved.borrow_mut() = reply);
    }
    let last = LAST_ACTION.with(|saved| saved.borrow().clone());
    json!({
        "poc":true,"count":findings.len(),"label":label,"expanded":expanded,
        "findings":findings,"status_source":"drawing.status",
        "repair_buttons":repair_buttons,"update_buttons":update_buttons,
        "last_action":last,"production_gui":false
    })
}
