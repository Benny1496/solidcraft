
/*
 * T107 runner-only compatibility probe.
 *
 * PR #13055's baseline does not yet emit notes through SheetScene::out_of_date.
 * The newer drawings branch does. We feed the newer, valid vector contract
 * into the unmodified production on_sheet() function here, without cherry-picking
 * Cam's PR or changing his code.
 *
 * This should FAIL on the original PR head if the note is mislabelled as a
 * dimension. A failing assertion is evidence only for the simulated contract
 * mismatch; it is not proof that the unmerged combination runs in the app.
 */
#[test]
fn t107_out_of_date_note_retains_note_record_kind_after_scene_contract_expands() {
    let (mut s, _) = plate();
    let (sheet, view) = drawn(&mut s);
    let note = run(
        &mut s,
        "drawing.note.add",
        json!({
            "anchor": {"view": ids::encode(view)},
            "at": [25, 20],
            "content": "T107 note"
        }),
    )["note"]
        .clone();
    let note_id = id(&note);
    let drawing = s.drawing("t107").unwrap().clone();
    let mut shown = crate::drawing_scene::as_shown(&mut s, &drawing, sheet).unwrap();

    // PR #13060 introduced notes into this existing Vec<id> result.
    // Recreate only that new result variant; call Cam's real on_sheet() below.
    assert!(!shown.out_of_date.contains(&note_id), "the old baseline is our control");
    shown.out_of_date.push(note_id);
    let findings = on_sheet(&drawing, sheet, shown).unwrap();
    let finding = findings
        .iter()
        .find(|found| found.kind == FindingKind::OutOfDate && found.object.id == ids::encode(note_id))
        .expect("the new note id must appear as an out-of-date finding");

    assert_eq!(
        finding.object.kind,
        CheckedKind::Note,
        "PR #13055 must not classify an out-of-date note as a dimension after PR #13060"
    );
}
