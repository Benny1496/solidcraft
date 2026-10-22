(async () => {
  const app = window.solidcraft;
  const check = (value, message) => { if (!value) throw Error(message); };
  const near = (value, expected, message) =>
    check(Number.isFinite(value) && Math.abs(value - expected) < 0.002,
      message + ': got ' + value + ', expected ' + expected);
  const key = 'org.solidcraft.experimental_drawing';

  await app.execute('file.new', {name: 'W23 M2 exact workshop acceptance matrix'});
  await app.execute('parameters.add', {name: 'M2Length', expression: '150 mm'});
  await app.execute('parameters.add', {name: 'M2BoreA', expression: '8 mm'});
  await app.execute('parameters.add', {name: 'M2BX', expression: '70 mm'});
  await app.execute('parameters.add', {name: 'M2BY', expression: '50 mm'});
  await app.execute('solid.box', {length: 'M2Length', width: 80, height: 10, body_name: 'Plate'});
  await app.execute('solid.hole', {name: 'M2A', position: [30, 20, 10], diameter: 'M2BoreA'});
  await app.execute('solid.hole', {name: 'M2B', position: ['M2BX', 'M2BY', 10], diameter: 8});

  let drawing = (await app.execute('drawing.preview', {body: 'Plate'})).drawing;
  const bore = (d, x, y) => d.geometry.cylinder_dimensions.find(c =>
    Math.abs(c.center_mm[0] - x) < 0.01 && Math.abs(c.center_mm[1] - y) < 0.01);
  const originalA = bore(drawing, 30, 20);
  const originalB = bore(drawing, 70, 50);
  check(originalA && originalB && originalA.eligible_face_name &&
    originalB.eligible_face_name && originalA.eligible_face_name !== originalB.eligible_face_name,
    'two independent verified W23 fixture bores');
  near(drawing.dimensions[0].value_mm, 150, 'original length');
  near(drawing.dimensions[1].value_mm, 80, 'original width');
  near(originalA.diameter_mm, 8, 'A diameter');
  near(originalB.diameter_mm, 8, 'B diameter');

  const add = async (params) => app.execute('drawing.annotation.add', {
    body: 'Plate', previous: drawing, ...params
  });
  let reply = await add({face_name: originalA.eligible_face_name, kind: 'diameter',
    position_mm: [20, 88]});
  drawing = reply.drawing;
  const aId = reply.annotation_id;
  reply = await add({face_name: originalB.eligible_face_name, kind: 'diameter',
    position_mm: [105, 89]});
  drawing = reply.drawing;
  const bId = reply.annotation_id;
  reply = await add({face_name: originalA.eligible_face_name,
    second_face_name: originalB.eligible_face_name, kind: 'center_distance',
    position_mm: [84, 98]});
  drawing = reply.drawing;
  const distId = reply.annotation_id;
  const annotation = (d, id) => d.annotations.find(a => a.id === id);
  near(annotation(drawing, aId).value_mm, 8, 'annotated A diameter');
  near(annotation(drawing, bId).value_mm, 8, 'annotated B diameter');
  near(annotation(drawing, distId).value_mm, 50, 'original centre distance');
  const positions = JSON.parse(JSON.stringify(drawing.label_positions));

  await app.execute('document.metadata.set', {key, value: drawing});
  const variants = [
    {name: 'length 150 to 180', changes: [['M2Length', '180 mm']],
      length: 180, bx: 70, by: 50, diaA: 8, distance: 50},
    {name: 'move B to 90,65', changes: [['M2BX', '90 mm'], ['M2BY', '65 mm']],
      length: 150, bx: 90, by: 65, diaA: 8, distance: 75},
    {name: 'A diameter 8 to 10', changes: [['M2BoreA', '10 mm']],
      length: 150, bx: 70, by: 50, diaA: 10, distance: 50}
  ];

  for (const v of variants) {
    for (const [name, expression] of v.changes)
      await app.execute('parameters.change', {name, expression});

    // The stored drawing is the old, deliberately closed snapshot.
    const saved = (await app.execute('document.metadata', {})).metadata[key];
    near(saved.dimensions[0].value_mm, 150, v.name + ': stored old length');
    near(annotation(saved, aId).value_mm, 8, v.name + ': stored old diameter');
    const pending = await app.execute('drawing.preview', {body: 'Plate', previous: saved});
    check(pending.status === 'stale' && pending.requires_confirmation,
      v.name + ': requires explicit approval');
    check(JSON.stringify(pending.drawing) === JSON.stringify(saved),
      v.name + ': rejects automatic mutation of stored drawing');
    const proposed = pending.proposed;
    near(proposed.dimensions[0].value_mm, v.length, v.name + ': live 3D length');
    near(proposed.dimensions[1].value_mm, 80, v.name + ': live 3D width');
    const changedA = bore(proposed, 30, 20);
    const changedB = bore(proposed, v.bx, v.by);
    check(changedA && changedB, v.name + ': 3D verified cylinder centres');
    near(changedA.diameter_mm, v.diaA, v.name + ': A real B-rep diameter');
    near(changedB.diameter_mm, 8, v.name + ': B real B-rep diameter');
    near(annotation(proposed, aId).value_mm, v.diaA, v.name + ': annotated A diameter');
    near(annotation(proposed, bId).value_mm, 8, v.name + ': annotated B diameter');
    near(annotation(proposed, distId).value_mm, v.distance,
      v.name + ': annotated centre distance');
    for (const id of [aId, bId, distId]) {
      check(annotation(proposed, id).reference_status === 'resolved',
        v.name + ': stable source identity ' + id);
      check(JSON.stringify(proposed.label_positions[id]) === JSON.stringify(positions[id]),
        v.name + ': manual paper-space label position ' + id);
    }
    const accepted = await app.execute('drawing.preview', {
      body: 'Plate', previous: saved, accept: true
    });
    check(accepted.status === 'updated' &&
      (await app.execute('drawing.preview', {body: 'Plate', previous: accepted.drawing})).status === 'current',
      v.name + ': accepted revision remains current');

    // Every edit is independent: undo all driving changes before the next row.
    for (let i = v.changes.length - 1; i >= 0; i--) await app.execute('edit.undo');
    const restored = await app.execute('drawing.preview', {body: 'Plate', previous: saved});
    check(restored.status === 'current', v.name + ': undo returns to baseline');
  }
  return {
    test: 'official W23/M2 150x80x10 two-bore model, three independent edits, confirmed updates and three retained annotation positions',
    variants: variants.length, result: 'passed'
  };
})()
