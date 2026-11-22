from pathlib import Path
p=Path("crates/ui-egui/src/drawing_repair_ui.rs")
s=p.read_text()
start='            ui_kit::dropdown_menu(ui, "drawing_repair_candidate", &caption, |ui| {'
end='''            });
            if !state.awaiting_confirmation {'''
assert s.count(start)==1 and s.count(end)==1
s=s.replace(start,start.replace("ui_kit::dropdown_menu","let t239_response = ui_kit::dropdown_menu"))
s=s.replace(end,'''            });
            let t239_shown = t239_response.rect.intersect(ui.clip_rect());
            if t239_shown.is_positive() {
                crate::scenario::publish_target("t239:candidate_combo", t239_shown);
            }
            if !state.awaiting_confirmation {''')
p.write_text(s)
