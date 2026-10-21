(async () => {
    const app = window.solidcraft;
    const check = (ok, message) => { if (!ok) throw new Error(message); };
    const close = (actual, expected, tolerance, label) => {
        check(Number.isFinite(actual) && Math.abs(actual - expected) <= tolerance,
              label + ': measured ' + actual + ', expected ' + expected);
    };
    const results = [];
    // Exercise simple analytic primitives independent of sketch/hole features.
    for (const radius of [3, 5, 8]) {
        await app.execute('file.new', {name: 'Cylinder radius ' + radius});
        await app.execute('solid.cylinder', {
            radius, height: 20, base: [0, 0, 0], body_name: 'TestCylinder'
        });
        const doc = await app.inspect({measure: true});
        check(doc.bodies.length === 1 && doc.timeline.length === 1,
              'one cylinder body and one feature, radius ' + radius);
        check(!doc.timeline.some(f => f.error),
              'no cylinder timeline errors, radius ' + radius);
        const body = doc.bodies[0];
        const expected = Math.PI * radius * radius * 20;
        close(body.volume_mm3, expected, Math.max(1, expected * 0.002),
              'cylinder volume, radius ' + radius);
        close(body.bbox.max[0] - body.bbox.min[0], 2 * radius, 0.02,
              'cylinder X extent');
        close(body.bbox.max[1] - body.bbox.min[1], 2 * radius, 0.02,
              'cylinder Y extent');
        close(body.bbox.max[2] - body.bbox.min[2], 20, 0.02,
              'cylinder height');
        const topo = await app.execute('model.faces', {body: 'TestCylinder'});
        const walls = topo.faces.filter(face => face.cylinder);
        check(walls.length > 0,
              'cylinder has at least one cylindrical B-rep face');
        for (const wall of walls) {
            close(2 * wall.cylinder.radius, radius * 2, 0.05,
                  'cylinder wall diameter');
        }
        results.push({radius_mm: radius, measured_volume_mm3: body.volume_mm3,
                      cylinder_faces: walls.length});
    }
    return {test: 'analytic cylinder primitive sizes', cylinders: results, result: 'passed'};
})()
