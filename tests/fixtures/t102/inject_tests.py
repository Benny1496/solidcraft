#!/usr/bin/env python3
"""Inject T102 click-to-update/candidate-repair UI only into temporary checkout."""
from pathlib import Path
sheet=Path("crates/ui-egui/src/drawing_sheet_page.rs")
tests=Path("crates/ui-egui/src/drawing_sheet_page_tests.rs")
source=sheet.read_text(encoding="utf-8")
existing=tests.read_text(encoding="utf-8")
poc=Path("../overlay/tests/fixtures/t102/warning_repair_poc.rs").read_text(encoding="utf-8")
cases=Path("../overlay/tests/fixtures/t102/warning_repair_tests.rs").read_text(encoding="utf-8")
anchor="    zoom_card(app, ui, canvas, at.scale / base);"
finish='    }));\n    true\n}'
assert source.count(anchor)==1, "Loohi's sheet host has changed; adapt the test"
assert source.count(finish)==1, "Loohi's sheet inspection has changed"
assert source.count("mod tests;")==1
assert existing.count("fn plate_sheet()")==1 and existing.count("fn page(")==1
assert "fn t102_native_update_button" not in existing
assert "pub(super) fn show(" in poc
assert cases.count("#[test]")==7, "Expected five new integration tests"
assert poc.count("Order::Foreground")==1
source=source.replace(anchor,anchor+'\n    #[cfg(test)]\n    let t102_warning_poc = t102_warning_poc::show(app, ui);')
source=source.replace(finish,
    '    }));\n'
    '    #[cfg(test)]\n'
    '    if let Some(shown) = app.drawing_workspace.page.shown.as_mut() {\n'
    '        shown["t102_warning_poc"] = t102_warning_poc;\n'
    '    }\n    true\n}')
source=source.replace("mod tests;", 'mod tests;\n\n#[cfg(test)]\n#[path = "t102_warning_poc.rs"]\nmod t102_warning_poc;')
sheet.write_text(source,encoding="utf-8")
tests.write_text(existing.rstrip("\n")+"\n\n"+cases.rstrip("\n")+"\n",encoding="utf-8")
Path("crates/ui-egui/src/t102_warning_poc.rs").write_text(poc,encoding="utf-8")
print("T102_INJECTED=5_REAL_NATIVE_EGUI_UPDATE_AND_REPAIR_ACTION_TESTS_CI_ONLY",flush=True)
