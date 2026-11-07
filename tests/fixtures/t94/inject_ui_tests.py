#!/usr/bin/env python3
"""Append T94 native egui acceptance checks to the disposable official-code checkout."""
from pathlib import Path
target=Path("crates/ui-egui/src/drawing_workspace.rs")
extra=Path("../overlay/tests/fixtures/t94/drawing_user_acceptance.rs")
original=target.read_text(encoding="utf-8")
snippet=extra.read_text(encoding="utf-8")
for needle in [
    'ui.label(crate::i18n::tr_label("No drawing sheet is available yet."))',
    'No drawing geometry available',
    'Apply (not implemented)',
    'pub fn enter(app: &mut SolveApp, drawing: bool)',
    '#[cfg(test)]',
]:
    assert needle in original, f"UI changed, do not silently rely on old placeholder: {needle}"
for name in [
    "t94_real_gui_shows_placeholder_despite_an_existing_engine_drawing_document",
    "t94_real_user_dimension_tool_opens_only_a_disabled_placeholder_panel",
    "t94_a_real_lost_engine_dimension_is_not_shown_as_a_user_warning_on_the_drawing_page",
]:
    assert ("fn "+name+"()") not in original
    assert snippet.count("fn "+name+"()")==1
target.write_text(original.rstrip("\n")+"\n\n"+snippet.rstrip("\n")+"\n",encoding="utf-8")
print("T94_NATIVE_EGUI_TESTS_INJECTED=3; TEMPORARY_CI_CHECKOUT_ONLY",flush=True)
