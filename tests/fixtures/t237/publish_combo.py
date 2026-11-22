from pathlib import Path
p=Path("crates/ui-egui/src/drawing_repair_ui.rs")
s=p.read_text()
start='            ui_kit::dropdown_menu(ui, "drawing_repair_candidate", &caption, |ui| {'
end='''            });
            if !state.awaiting_confirmation {'''
assert s.count(start)==1 and s.count(end)==1
s=s.replace(start,start.replace("ui_kit::dropdown_menu","let t235_response = ui_kit::dropdown_menu"))
s=s.replace(end,'''            });
            let t235_shown = t235_response.rect.intersect(ui.clip_rect());
            if t235_shown.is_positive() {
                crate::scenario::publish_target("t235:candidate_combo", t235_shown);
            }
            if !state.awaiting_confirmation {''')
p.write_text(s)
