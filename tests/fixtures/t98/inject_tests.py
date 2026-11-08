#!/usr/bin/env python3
"""Inject native egui T98 tests into Loohi PR head in runner checkout ONLY."""
from pathlib import Path
target=Path("crates/ui-egui/src/drawing_sheet_page_tests.rs")
addition=Path("../overlay/tests/fixtures/t98/sheet_warning_integration.rs").read_text(encoding="utf-8")
base=target.read_text(encoding="utf-8")
for anchor in ("fn plate_sheet()", "fn settle(", "fn page(", "fn the_plate_sheet_draws_its_lines_and_text()", "fn a_views_state_is_in_words()"):
    assert anchor in base, f"PR #13211 changed, adapt T98 fixture: {anchor}"
for name in ("t98_dim_id", "t98_drawing_id",
             "t98_native_sheet_after_3d_model_edit_keeps_updated_dimension",
             "t98_diagnostic_deferred_dimension_only_missing_from_sheet_page",
             "t98_diagnostic_lost_dimension_has_no_visible_repair_card_yet"):
    assert "fn " + name + "(" not in base, name
    assert addition.count("fn "+name+"(")==1, name
assert addition.count("T98_GAP=")==2
target.write_text(base.rstrip("\n")+"\n\n"+addition.rstrip("\n")+"\n",encoding="utf-8")
print("T98_INJECTED=3_NEW_NATIVE_EGUI_TESTS (1 POSITIVE UI + 2 DOCUMENTED UI GAPS)",flush=True)
