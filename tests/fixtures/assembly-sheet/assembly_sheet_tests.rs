// ---------------------------------------------------------------------------------------------
// Assembly sheet acceptance, stage A (draft by @Benny1496 for a child issue of #4894; fork only).
// A small test assembly whose parts list can be counted by hand, read through what 'drawings' can
// do today: the bill of materials and its saved item numbers, balloons on a view, and revision rows.
// The parts list and the revision table on the sheet, and a balloon's item number, are stage B
// (not run here: they wait for the rest of #3874).
// ---------------------------------------------------------------------------------------------

use crate::tests::Folder;

/// The test assembly, its parts countable by hand:
/// - a base plate BP-100, 120 x 80 x 10 mm from the origin, placed once;
/// - a screw SC-M6, a 6 mm cylinder 20 long standing on the plate at (5, 5), placed four times on
///   the plate's corners: (5, 5), (105, 5), (5, 65) and (105, 65);
/// - a holder HL-300, an assembly placed twice (the second 40 mm along Y), each holding a block
///   BL-200 (20 x 20 x 30 mm from (40, 10, 10)) and two of the same screws, at (35, 5) and (35, 25).
///
/// Parts only: the block 2, the plate 1 and the screw 8 (4 + 2 x 2).
struct SheetAssembly {
    s: Session,
    plate: u64,
    screw: u64,
    holders: [u64; 2],
    block: u64,
}

fn occurrence(reply: &Value) -> u64 {
    reply["occurrence"].as_u64().unwrap_or_else(|| panic!("no occurrence in {reply}"))
}

fn sheet_assembly() -> SheetAssembly {
    let mut s = Session::default();
    let plate = occurrence(&run(&mut s, "component.create", json!({"name": "BasePlate"})));
    run(&mut s, "solid.box", json!({"length": 120, "width": 80, "height": 10, "body_name": "Plate"}));
    run(&mut s, "component.activate", json!({"component": 0}));
    let screw = occurrence(&run(&mut s, "component.create", json!({"name": "Screw"})));
    run(&mut s, "solid.cylinder", json!({"diameter": 6, "height": 20, "base": [5, 5, 10]}));
    run(&mut s, "component.activate", json!({"component": 0}));
    for at in [[100, 0, 0], [0, 60, 0], [100, 60, 0]] {
        run(&mut s, "occurrence.copy", json!({"component": "Screw", "translate": at}));
    }
    let first = occurrence(&run(&mut s, "component.create", json!({"name": "Holder", "activate": false})));
    let block = occurrence(&run(&mut s, "component.create", json!({"name": "Block", "parent": "Holder"})));
    run(&mut s, "solid.box", json!({"length": 20, "width": 20, "height": 30, "corner": [40, 10, 10], "body_name": "Block"}));
    run(&mut s, "component.activate", json!({"component": 0}));
    for at in [[30, 0, 0], [30, 20, 0]] {
        run(&mut s, "occurrence.copy", json!({"component": "Screw", "parent": "Holder", "translate": at}));
    }
    let second = occurrence(&run(&mut s, "occurrence.copy", json!({"occurrence": first, "translate": [0, 40, 0]})));
    for (component, part_number, description) in [
        ("BasePlate", "BP-100", "Base plate 120 x 80 x 10"),
        ("Screw", "SC-M6", "Screw M6 x 20"),
        ("Holder", "HL-300", "Holder"),
        ("Block", "BL-200", "Block 20 x 20 x 30"),
    ] {
        run(&mut s, "component.metadata.set", json!({"component": component, "part_number": part_number, "description": description}));
    }
    SheetAssembly { s, plate, screw, holders: [first, second], block }
}

/// A list's rows as (part number, component, count).
fn sheet_rows(r: &Value) -> Vec<(String, String, u64)> {
    r["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("no rows in {r}"))
        .iter()
        .map(|row| {
            (row["part_number"].as_str().unwrap().to_string(), row["definition"].as_str().unwrap().to_string(), row["count"].as_u64().unwrap())
        })
        .collect()
}

fn sheet_row(part_number: &str, definition: &str, count: u64) -> (String, String, u64) {
    (part_number.into(), definition.into(), count)
}

/// Every row of all levels, children under their parent, as (item, part number, each, count).
fn sheet_deep(r: &Value) -> Vec<(String, String, u64, u64)> {
    fn walk(rows: &Value, out: &mut Vec<(String, String, u64, u64)>) {
        for row in rows.as_array().into_iter().flatten() {
            let item = row["item"].as_str().map_or_else(|| row["item"].to_string(), str::to_string);
            let count = row["count"].as_u64().unwrap();
            out.push((item, row["part_number"].as_str().unwrap().into(), row["each"].as_u64().unwrap_or(count), count));
            walk(&row["children"], out);
        }
    }
    let mut out = Vec::new();
    walk(&r["rows"], &mut out);
    out
}

/// A list's rows as (item, part number, flags): p provisional, l locked, d duplicate.
fn sheet_numbers(r: &Value) -> Vec<(String, String, String)> {
    r["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let flags: String =
                [("provisional", 'p'), ("locked", 'l'), ("duplicate", 'd')].iter().filter(|(k, _)| row[*k] == true).map(|(_, c)| *c).collect();
            (row["item"].to_string().trim_matches('"').to_string(), row["part_number"].as_str().unwrap().to_string(), flags)
        })
        .collect()
}

fn sheet_number(item: &str, part_number: &str) -> (String, String, String) {
    (item.into(), part_number.into(), String::new())
}

/// The test assembly's bill of materials, counted by hand, in every view; its item numbers saved,
/// kept through a new part, undo and redo, a model edit and its undo, a second view's own
/// numbering, and save and reopen in both formats.
#[test]
fn assembly_sheet_parts_list_counts_by_hand_and_keeps_its_item_numbers() {
    let SheetAssembly { mut s, holders, .. } = sheet_assembly();
    let steps = s.undo.len();

    // Parts only: 2 blocks, 1 plate, 8 screws (4 on the plate, 2 in each holder).
    let flat = run(&mut s, "assembly.bom", json!({}));
    assert_eq!(sheet_rows(&flat), [sheet_row("BL-200", "Block", 2), sheet_row("BP-100", "BasePlate", 1), sheet_row("SC-M6", "Screw", 8)], "{flat}");
    assert_eq!(flat["total_occurrences"], 11, "1 + 2 + 8 parts placed: {flat}");
    assert_eq!(flat["numbered"], false);
    // The first level: the plate, the holder twice, the four screws on the plate.
    let top = run(&mut s, "assembly.bom", json!({"levels": "top"}));
    assert_eq!(sheet_rows(&top), [sheet_row("BP-100", "BasePlate", 1), sheet_row("HL-300", "Holder", 2), sheet_row("SC-M6", "Screw", 4)], "{top}");
    assert_eq!(top["total_occurrences"], 7, "1 + 2 + 4: {top}");
    // All levels: each holder's rows under it, 1 block and 2 screws in each, 2 and 4 in all.
    let all = run(&mut s, "assembly.bom", json!({"levels": "all"}));
    let deep = |item: &str, pn: &str, each: u64, count: u64| (item.to_string(), pn.to_string(), each, count);
    assert_eq!(
        sheet_deep(&all),
        [deep("1", "BP-100", 1, 1), deep("2", "HL-300", 2, 2), deep("2.1", "BL-200", 1, 2), deep("2.2", "SC-M6", 2, 4), deep("3", "SC-M6", 4, 4)],
        "{all}"
    );
    assert_eq!(all["total_occurrences"], 13, "1 + 2 + 2 + 4 + 4: {all}");
    // Each holder's own list.
    let holder = run(&mut s, "assembly.bom", json!({"component": "Holder"}));
    assert_eq!(sheet_rows(&holder), [sheet_row("BL-200", "Block", 1), sheet_row("SC-M6", "Screw", 2)], "{holder}");
    assert_eq!(s.undo.len(), steps, "reading a bill of materials changes nothing");

    // Numbered from 10 in tens, in one undo step: 10, 20, 30 in part-number order.
    let numbered = run(&mut s, "assembly.bom.renumber", json!({"start": 10, "step": 10}));
    let three = [sheet_number("10", "BL-200"), sheet_number("20", "BP-100"), sheet_number("30", "SC-M6")];
    assert_eq!((numbered["numbered"].as_bool(), sheet_numbers(&numbered)), (Some(true), three.to_vec()), "{numbered}");
    assert_eq!(s.undo.len(), steps + 1, "one undo step");

    // A washer added: it takes the highest number + 1; undone, it goes with its number; redone, it
    // comes back with the same number.
    run(&mut s, "component.create", json!({"name": "Washer", "activate": false}));
    run(&mut s, "component.metadata.set", json!({"component": "Washer", "part_number": "WS-010", "description": "Washer M6"}));
    let four = [sheet_number("10", "BL-200"), sheet_number("20", "BP-100"), sheet_number("30", "SC-M6"), sheet_number("31", "WS-010")];
    assert_eq!(sheet_numbers(&run(&mut s, "assembly.bom", json!({}))), four);
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(sheet_numbers(&run(&mut s, "assembly.bom", json!({}))), three);
    run(&mut s, "edit.redo", json!({}));
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(sheet_numbers(&run(&mut s, "assembly.bom", json!({}))), four, "the washer is 31 again");

    // The second holder deleted: 1 block and 6 screws, every number kept. Undone: 2 and 8 again.
    run(&mut s, "selection.delete", json!({"occurrences": [holders[1]]}));
    let fewer = run(&mut s, "assembly.bom", json!({}));
    assert_eq!(
        sheet_rows(&fewer),
        [sheet_row("BL-200", "Block", 1), sheet_row("BP-100", "BasePlate", 1), sheet_row("SC-M6", "Screw", 6), sheet_row("WS-010", "Washer", 1)],
        "{fewer}"
    );
    assert_eq!(sheet_numbers(&fewer), four, "a model edit keeps the numbers");
    run(&mut s, "edit.undo", json!({}));
    let back = run(&mut s, "assembly.bom", json!({}));
    assert_eq!(sheet_rows(&back)[2], sheet_row("SC-M6", "Screw", 8), "{back}");
    assert_eq!(sheet_numbers(&back), four);

    // The first level numbered on its own: 1 to 4, and parts only keeps its numbers.
    let top = run(&mut s, "assembly.bom.renumber", json!({"levels": "top"}));
    assert_eq!(
        sheet_numbers(&top),
        [sheet_number("1", "BP-100"), sheet_number("2", "HL-300"), sheet_number("3", "SC-M6"), sheet_number("4", "WS-010")],
        "{top}"
    );
    assert_eq!(sheet_numbers(&run(&mut s, "assembly.bom", json!({}))), four, "each view keeps its own numbers");
    assert_eq!(s.doc.assembly.item_numbers.len(), 2);

    // Saved and opened again, in both formats: the same rows, counts and numbers in both views.
    let folder = Folder::new("assembly-sheet-numbers");
    for (file, format) in [("assembly.json", "json"), ("assembly.solidcraft", "container")] {
        let path = folder.path(file);
        run(&mut s, "file.save_as", json!({"path": path, "format": format}));
        let mut back = Session::default();
        run(&mut back, "doc.open", json!({"path": path}));
        for view in [json!({}), json!({"levels": "top"}), json!({"levels": "all"})] {
            let (was, is) = (run(&mut s, "assembly.bom", view.clone()), run(&mut back, "assembly.bom", view.clone()));
            assert_eq!(sheet_numbers(&is), sheet_numbers(&was), "{format} {view}");
            assert_eq!(sheet_deep(&is), sheet_deep(&was), "{format} {view}");
        }
        assert_eq!(back.doc.assembly.item_numbers, s.doc.assembly.item_numbers, "{format}");
    }
}

/// Balloons on the test assembly's top view and the drawing's revision rows, through model edits
/// made with the drawing closed (CamWight's workspace rule), their undo and redo, and save and
/// reopen: a balloon follows its occurrence, a deleted holder's balloon reads lost and is never
/// moved to the other holder or removed, and the drawing's saved record never changes with the
/// model. The revision rows stay as they were through all of it.
#[test]
fn assembly_sheet_balloons_and_revisions_follow_the_model_and_the_file() {
    let SheetAssembly { mut s, plate, screw, holders: [first, second], block } = sheet_assembly();
    run(&mut s, "drawing.create", json!({}));
    let top = run(&mut s, "drawing.view.base", json!({"sheet": "Sheet 1", "view": "top", "scale": "1:2", "position_mm": [150, 150]}))["view"].clone();
    let drawing = ids::encode(s.drawing("test").unwrap().id);

    // Four balloons: the plate's top front edge, the first screw's top rim, and each holder's block
    // by its top front edge through that holder.
    let put = |s: &mut Session, point: [f64; 3], path: Value, at: [f64; 2]| {
        let reply = run(s, ADD, json!({"view": top, "edge": {"point": point, "path": path}, "at": at}));
        assert_eq!(reply["status"], "ok", "{reply}");
        assert_eq!(reply["occurrence"], path, "{reply}");
        reply
    };
    let on_plate = put(&mut s, [60.0, 0.0, 10.0], json!([plate]), [-60.0, -50.0]);
    let on_screw = put(&mut s, [8.0, 5.0, 30.0], json!([screw]), [-70.0, 40.0]);
    let on_first = put(&mut s, [50.0, 10.0, 40.0], json!([first, block]), [0.0, 60.0]);
    let on_second = put(&mut s, [50.0, 50.0, 40.0], json!([second, block]), [40.0, 60.0]);
    ends_at(&on_plate, [60.0, 0.0, 10.0]);
    ends_at(&on_first, [50.0, 10.0, 40.0]);
    ends_at(&on_second, [50.0, 50.0, 40.0]);
    assert_ne!(on_first["record"]["edge"], on_second["record"]["edge"], "one block, two placements, two balloons");
    let balloons = [&on_plate, &on_screw, &on_first, &on_second].map(|b| b["balloon"].clone());

    // Revision rows 1, 2 and 3; row 2 deleted: 1 and 3 stay, and the next is still 4.
    let mut rows = Vec::new();
    for (description, date) in [("First issue", "2026-11-03"), ("Withdrawn", "2026-11-04"), ("Holder moved", "2026-11-05")] {
        let added = run(&mut s, "drawing.revision.add", json!({"set": {"description": description, "designer": "BS", "date": date}}));
        rows.push(added["row"].clone());
    }
    run(&mut s, "drawing.revision.delete", json!({"row": rows[1]}));
    let revisions = run(&mut s, "drawing.inspect", json!({}))["revisions"].clone();
    let numbers: Vec<&str> = revisions["rows"].as_array().unwrap().iter().map(|r| r["revision"].as_str().unwrap()).collect();
    assert_eq!((numbers, revisions["next"].as_str()), (vec!["1", "3"], Some("4")), "{revisions}");

    let record = saved(&s);
    let on_the_model = |s: &mut Session, id: &str, p: Value| -> Value {
        run(s, "drawing.close", json!({}));
        let reply = run(s, id, p);
        run(s, "drawing.open", json!({"drawing": drawing}));
        reply
    };
    let read = |s: &mut Session| balloons.clone().map(|b| run(s, INSPECT, json!({"balloon": b})));
    let statuses = |reads: &[Value; 4]| reads.clone().map(|r| r["status"].as_str().unwrap().to_string());
    let ok = ["ok", "ok", "ok", "ok"].map(String::from);
    let parts_only = |s: &mut Session| on_the_model(s, "assembly.bom", json!({}));

    // The second holder moved 30 mm along Y: its balloon's arrow follows, the others stay.
    on_the_model(&mut s, "occurrence.move", json!({"occurrence": second, "space": "world", "translate": [0, 30, 0]}));
    let moved = read(&mut s);
    assert_eq!(statuses(&moved), ok);
    ends_at(&moved[0], [60.0, 0.0, 10.0]);
    ends_at(&moved[2], [50.0, 10.0, 40.0]);
    ends_at(&moved[3], [50.0, 80.0, 40.0]);
    assert_eq!(saved(&s), record, "a model edit leaves the drawing's record alone");
    on_the_model(&mut s, "edit.undo", json!({}));
    ends_at(&read(&mut s)[3], [50.0, 50.0, 40.0]);

    // The second holder deleted: its balloon is lost, says why, keeps its occurrence and is still
    // there; the first holder's, the plate's and the screw's stay ok. Parts only: 1 block, 6 screws.
    on_the_model(&mut s, "selection.delete", json!({"occurrences": [second]}));
    let gone = read(&mut s);
    assert_eq!(statuses(&gone), ["ok", "ok", "ok", "lost"].map(String::from), "{gone:?}");
    assert!(gone[3]["why"].is_string(), "{}", gone[3]);
    assert_eq!(gone[3]["occurrence"], json!([second, block]), "never moved to the first holder");
    assert_eq!(gone[3]["record"], on_second["record"], "never removed or changed");
    ends_at(&gone[2], [50.0, 10.0, 40.0]);
    assert_eq!(saved(&s), record);
    let fewer = parts_only(&mut s);
    assert_eq!(sheet_rows(&fewer), [sheet_row("BL-200", "Block", 1), sheet_row("BP-100", "BasePlate", 1), sheet_row("SC-M6", "Screw", 6)], "{fewer}");

    // Saved with the holder deleted and opened again: the same record, statuses and revisions.
    let folder = Folder::new("assembly-sheet-balloons");
    let path = folder.path("assembly.solidcraft");
    run(&mut s, "file.save", json!({"path": path}));
    let mut opened = Session::default();
    run(&mut opened, "doc.open", json!({"path": path}));
    run(&mut opened, "drawing.open", json!({"drawing": drawing}));
    assert_eq!(saved(&opened), record, "opened again");
    let again = read(&mut opened);
    assert_eq!(statuses(&again), statuses(&gone));
    for (was, is) in gone.iter().zip(&again) {
        assert_eq!((&is["record"], &is["occurrence"]), (&was["record"], &was["occurrence"]), "{is}");
    }
    assert_eq!(run(&mut opened, "drawing.inspect", json!({}))["revisions"], revisions, "revision rows opened again");

    // Undone: the second holder and its balloon are back; redone, lost again; undone once more.
    on_the_model(&mut s, "edit.undo", json!({}));
    let back = read(&mut s);
    assert_eq!(statuses(&back), ok, "{back:?}");
    ends_at(&back[3], [50.0, 50.0, 40.0]);
    on_the_model(&mut s, "edit.redo", json!({}));
    assert_eq!(statuses(&read(&mut s))[3], "lost");
    on_the_model(&mut s, "edit.undo", json!({}));
    assert_eq!(statuses(&read(&mut s)), ok);
    assert_eq!(sheet_rows(&parts_only(&mut s))[2], sheet_row("SC-M6", "Screw", 8));

    // Through every model edit, the drawing's record and its revision rows are as they were.
    assert_eq!(saved(&s), record);
    assert_eq!(run(&mut s, "drawing.inspect", json!({}))["revisions"], revisions);
}
