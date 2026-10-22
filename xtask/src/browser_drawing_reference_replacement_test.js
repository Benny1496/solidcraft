(async () => {
  const app = window.solidcraft;
  const check = (value, message) => { if (!value) throw Error(message); };
  const near = (v, want, message) =>
    check(Number.isFinite(v) && Math.abs(v - want) < 0.002, message + ' ' + v + ' vs ' + want);
  await app.execute('file.new', {name: 'W23 lost bore and geometrically identical replacement'});
  await app.execute('solid.box', {length: 150, width: 80, height: 10, body_name: 'Plate'});
  // A is deliberately the LAST timeline feature, so an undo removes A but keeps B.
  await app.execute('solid.hole', {name: 'M2B', position: [70, 50, 10], diameter: 8});
  await app.execute('solid.hole', {name: 'M2A', position: [30, 20, 10], diameter: 8});
  let d = (await app.execute('drawing.preview', {body: 'Plate'})).drawing;
  const bore = (drawing, x) => drawing.geometry.cylinder_dimensions.find(c =>
    Math.abs(c.center_mm[0] - x) < 0.01);
  const a = bore(d, 30), b = bore(d, 70);
  check(a && b && a.eligible_face_name && b.eligible_face_name &&
    a.eligible_face_name !== b.eligible_face_name, 'two independently verified original faces');
  const originalA = a.eligible_face_name, originalB = b.eligible_face_name;

  let reply = await app.execute('drawing.annotation.add', {
    body: 'Plate', previous: d, face_name: originalA, kind: 'diameter', position_mm: [19, 90]
  });
  d = reply.drawing;
  const aId = reply.annotation_id;
  reply = await app.execute('drawing.annotation.add', {
    body: 'Plate', previous: d, face_name: originalB, kind: 'diameter', position_mm: [105, 88]
  });
  d = reply.drawing;
  const bId = reply.annotation_id;
  reply = await app.execute('drawing.annotation.add', {
    body: 'Plate', previous: d, face_name: originalA, second_face_name: originalB,
    kind: 'center_distance', position_mm: [83, 97]
  });
  d = reply.drawing;
  const distanceId = reply.annotation_id;
  const note = (drawing, id) => drawing.annotations.find(x => x.id === id);
  const manual = JSON.parse(JSON.stringify(d.label_positions));

  await app.execute('edit.undo'); // remove A, retain B
  const removed = await app.execute('drawing.preview', {body: 'Plate', previous: d});
  check(removed.status === 'stale' && removed.requires_confirmation, 'A deletion is stale');
  check(removed.proposed.geometry.cylinder_dimensions.length === 1, 'only B survives deletion');
  check(note(removed.proposed, aId).reference_status === 'missing_reference' &&
    note(removed.proposed, distanceId).reference_status === 'missing_reference',
    'A and distance are unresolved after deletion');
  check(note(removed.proposed, bId).reference_status === 'resolved',
    'B remains an independently verified bore');

  // Same geometry, but deliberately a NEW feature, not a repair of the old reference.
  await app.execute('solid.hole', {
    name: 'M2ReplacementA', position: [30, 20, 10], diameter: 8
  });
  const replaced = await app.execute('drawing.preview', {body: 'Plate', previous: d});
  check(replaced.proposed || replaced.drawing, 'comparison result returned');
  const geometry = (replaced.proposed || replaced.drawing).geometry;
  check(geometry.cylinder_dimensions.length === 2, 'the model again has two real cylinders');
  const replacement = geometry.cylinder_dimensions.find(c =>
    Math.abs(c.center_mm[0] - 30) < 0.01 && Math.abs(c.center_mm[1] - 20) < 0.01);
  check(replacement && replacement.eligible_face_name,
    'replacement bore has an independently verified face');
  check(replacement.eligible_face_name !== originalA,
    'NEW feature must never recycle original A face identity');
  check(replaced.status === 'stale' && replaced.requires_confirmation,
    'replacement must not silently make the stored drawing current');
  check(JSON.stringify(replaced.drawing) === JSON.stringify(d), 'stored sheet remains unchanged');
  for (const id of [aId, bId, distanceId])
    check(JSON.stringify(replaced.proposed.label_positions[id]) === JSON.stringify(manual[id]),
      'manual label coordinates survive replacement ' + id);
  const missingA = note(replaced.proposed, aId);
  check(missingA.reference_status === 'missing_reference' && missingA.value_mm === null,
    'historical A dimension remains unresolved, despite equal coordinates');
  near(missingA.last_value_mm, 8, 'historical diameter');
  check(note(replaced.proposed, distanceId).reference_status === 'missing_reference',
    'historical centre distance is not reassociated');
  near(note(replaced.proposed, distanceId).last_value_mm, 50, 'historical distance');
  check(note(replaced.proposed, bId).reference_status === 'resolved',
    'unaffected B remains resolved');
  const accepted = await app.execute('drawing.preview', {
    body: 'Plate', previous: d, accept: true
  });
  check(accepted.status === 'updated' &&
    note(accepted.drawing, aId).reference_status === 'missing_reference',
    'confirmation does not implicitly repair historical identity');
  check((await app.execute('drawing.preview', {
    body: 'Plate', previous: accepted.drawing
  })).status === 'current', 'accepted missing reference does not loop');
  return {
    test: 'new bore at exactly old XY and diameter cannot silently heal lost source identity',
    result: 'passed'
  };
})()
