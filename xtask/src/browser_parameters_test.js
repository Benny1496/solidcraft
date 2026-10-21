(async () => {
    const app = window.solidcraft;
    const check = (ok, msg) => { if (!ok) throw new Error(msg); };
    const close = (a, b, tol, msg) =>
        check(Number.isFinite(a) && Math.abs(a - b) <= tol,
              msg + ': measured ' + a + ', expected ' + b);
    await app.execute('file.new', {name: 'Parametric plate test'});
    await app.execute('parameters.add', {name: 'PlateLength', expression: '150 mm'});
    await app.execute('solid.box', {
        length: 'PlateLength', width: 80, height: 10, body_name: 'AdjustablePlate'
    });
    const read = async (expected) => {
        const doc = await app.inspect({measure: true});
        check(doc.bodies.length === 1 && doc.timeline.length === 1,
              'parametric plate stays a single body and feature');
        const body = doc.bodies[0];
        close(body.bbox.max[0] - body.bbox.min[0], expected, 0.01,
              'model X size follows the parameter');
        close(body.bbox.max[1] - body.bbox.min[1], 80, 0.01, 'unchanged width');
        close(body.bbox.max[2] - body.bbox.min[2], 10, 0.01, 'unchanged thickness');
        close(body.volume_mm3, expected * 800, 0.1, 'correct parametric volume');
        check(!doc.timeline.some(f => f.error), 'no timeline recompute errors');
        return body.volume_mm3;
    };
    const initialVolume = await read(150);
    await app.execute('parameters.change', {name: 'PlateLength', expression: '180 mm'});
    const expandedVolume = await read(180);
    await app.execute('edit.undo');
    const restoredVolume = await read(150);
    await app.execute('edit.redo');
    const redoneVolume = await read(180);
    return {test: 'parameters, recomputation, undo and redo',
            lengths_mm: [150, 180, 150, 180],
            volumes_mm3: [initialVolume, expandedVolume, restoredVolume, redoneVolume],
            result: 'passed'};
})()
