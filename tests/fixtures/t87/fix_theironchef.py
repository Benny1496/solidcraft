#!/usr/bin/env python3
"""Test-only, exact-match repair patches proposed in upstream PR reviews.
No GitHub writes. Applied ONLY inside ephemeral Actions checkout.
"""
import sys
from pathlib import Path

def replace(path, old, new):
    file = Path(path)
    txt = file.read_text(encoding="utf-8")
    count = txt.count(old)
    if count != 1:
        raise SystemExit(f"REFUSE PATCH {path}: expected one exact anchor, got {count}")
    file.write_text(txt.replace(old, new, 1), encoding="utf-8")
    print(f"PATCHED {path}")

case = sys.argv[1]
if case == "12394":
    replace("crates/engine/src/cmd/drawing_view.rs",
        '        SetBy::Parent(parent) => format!(" (set on view {})", ids::encode(parent)),\n        SetBy::This | SetBy::Default => String::new(),',
        '        SetBy::Parent(parent) => format!(" (set on view {})", ids::encode(parent)),\n        SetBy::Sheet => " (set on its sheet)".to_string(),\n        SetBy::Drawing => " (set on the drawing)".to_string(),\n        SetBy::This | SetBy::Default => String::new(),')
elif case == "9704":
    replace("crates/geom/src/drawing_arrange.rs", "use crate::{Aabb2, Vec2};",
            "use crate::Aabb2;\nuse solidcore_record::Vec2;")
    replace("crates/geom/src/drawing_arrange_tests.rs", "use super::*;",
            "use super::*;\nuse crate::prelude::*;")
elif case == "7579":
    replace("crates/engine/src/cmd/drawing_break.rs", "use solidcraft_geom::Vec2;",
            "use solidcore_record::Vec2;")
    replace("crates/geom/src/drawing_break.rs", "use crate::{EPS, Vec2};",
            "use crate::EPS;\nuse crate::prelude::*;\nuse solidcore_record::Vec2;")
    replace("crates/ui-egui/src/broken_view_preview.rs", "use solidcraft_engine::geom::Vec2;",
            "use solidcore_record::Vec2;")
    replace("crates/ui-egui/src/broken_view_preview.rs",
            "use solidcraft_engine::geom::drawing_view::Scale;",
            "use solidcraft_engine::geom::drawing_view::Scale;\nuse solidcraft_engine::geom::prelude::*;")
elif case == "9720":
    replace("crates/geom/src/drawing_detail.rs", "use crate::{Aabb2, Seg2, Similarity2, Vec2};",
            "use crate::prelude::*;\nuse crate::{Aabb2, Similarity2};\nuse solidcore_record::{Seg2, Vec2};")
    replace("crates/geom/src/drawing_detail_tests.rs",
            "use crate::Vec3;\nuse crate::drawing_view::{NamedView, PRESETS};\nuse crate::profile::Loop2;",
            "use crate::drawing_view::{NamedView, PRESETS};\nuse solidcore_record::{Loop2, Vec3};")
else:
    raise SystemExit("Unknown or unauthorized test case")
