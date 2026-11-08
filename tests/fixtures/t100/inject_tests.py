#!/usr/bin/env python3
"""Inject test-only D07/D26 prototype into the pinned Loohi PR runner checkout."""
from pathlib import Path
sheet=Path("crates/ui-egui/src/drawing_sheet_page.rs")
tests=Path("crates/ui-egui/src/drawing_sheet_page_tests.rs")
base=sheet.read_text(encoding="utf-8")
existing=tests.read_text(encoding="utf-8")
prototype=Path("../overlay/tests/fixtures/t100/warning_poc.rs").read_text(encoding="utf-8")
cases=Path("../overlay/tests/fixtures/t100/warning_poc_tests.rs").read_text(encoding="utf-8")
needle="    zoom_card(app, ui, canvas, at.scale / base);"
finish='    }));\n    true\n}'
assert base.count(needle)==1, "Loohi sheet page location changed"
assert base.count(finish)==1, "Loohi sheet page JSON output changed"
assert base.count('mod tests;')==1
assert existing.count('fn plate_sheet()')==1 and existing.count('fn page(')==1
assert 'fn t100_native_current_sheet_shows_all_current_status_pill' not in existing
assert prototype.count('pub(super) fn show(')==1
# The status card is strictly cfg(test) and does not appear in production builds.
base=base.replace(needle,needle+'\n    #[cfg(test)]\n    let t100_warning_poc = t100_warning_poc::show(app, ui);')
base=base.replace(finish,'    }));\n    #[cfg(test)]\n    if let Some(state) = app.drawing_workspace.page.shown.as_mut() {\n        state["t100_warning_poc"] = t100_warning_poc;\n    }\n    true\n}')
base=base.replace('mod tests;','mod tests;\n\n#[cfg(test)]\n#[path = "t100_warning_poc.rs"]\nmod t100_warning_poc;')
sheet.write_text(base,encoding="utf-8")
tests.write_text(existing.rstrip("\n")+"\n\n"+cases.rstrip("\n")+"\n",encoding="utf-8")
Path("crates/ui-egui/src/t100_warning_poc.rs").write_text(prototype,encoding="utf-8")
print("T100_INJECTED=TEMPORARY_TEST_ONLY_NATIVE_EGUI_STATUS_PILL_AND_6_CASES",flush=True)
