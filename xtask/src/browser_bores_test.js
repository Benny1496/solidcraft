(async () => {
    const app = window.solidcraft;
    const check = (ok, msg) => { if (!ok) throw new Error(msg); };
    const close = (a, b, tol, msg) =>
        check(Number.isFinite(a) && Math.abs(a - b) <= tol,
              msg + ': measured ' + a + ', expected ' + b);
    await app.execute('file.new', {name: 'Blind and through bore test'});
    await app.execute('solid.box', {
        length: 60, width: 40, height: 12, body_name: 'BorePlate'
    });
    await app.execute('solid.hole', {
        position: [12, 10, 12], diameter: 6, depth: 5
    });
    let doc = await app.inspect({measure: true});
    const blindExpected = 60 * 40 * 12 - Math.PI * 3 * 3 * 5;
    check(doc.bodies.length === 1, 'blind hole leaves one body');
    close(doc.bodies[0].volume_mm3, blindExpected, 18, 'blind 6 mm by 5 mm hole volume');
    await app.execute('solid.hole', {
        position: [48, 30, 12], diameter: 8
    });
    doc = await app.inspect({measure: true});
    check(doc.bodies.length === 1 && doc.timeline.length === 3,
          'three features: plate, blind hole and through-hole');
    check(!doc.timeline.some(f => f.error), 'all bore timeline features succeed');
    const expectedVolume = blindExpected - Math.PI * 4 * 4 * 12;
    close(doc.bodies[0].volume_mm3, expectedVolume, 24,
          'combined blind and through-hole volume');
    const topology = await app.execute('model.faces', {body: 'BorePlate'});
    const cylinders = topology.faces.filter(f => f.cylinder);
    check(cylinders.length >= 2, 'both hole walls appear as cylindrical faces');
    for (const [x, y, diameter] of [[12, 10, 6], [48, 30, 8]]) {
        const walls = cylinders.filter(f => {
            const c = f.cylinder;
            return Math.hypot(c.origin[0] - x, c.origin[1] - y) < 0.2
                && Math.abs(c.radius * 2 - diameter) < 0.15;
        });
        check(walls.length > 0, 'measured cylindrical wall for ' + diameter + ' mm hole');
    }
    return {test: 'blind and through-hole mixed geometry',
            expected_volume_mm3: expectedVolume,
            measured_volume_mm3: doc.bodies[0].volume_mm3,
            cylindrical_faces: cylinders.length, result: 'passed'};
})()
