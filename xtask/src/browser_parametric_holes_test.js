(async () => {
    const app = window.solidcraft;
    const check = (condition, label) => { if (!condition) throw new Error(label); };
    const near = (v, expected, tol, label) => {
        check(Number.isFinite(v) && Math.abs(v - expected) <= tol,
            label + ': measured ' + v + ', expected ' + expected);
    };
    await app.execute('file.new', {name: 'Parametric hole alignment'});
    await app.execute('parameters.add', {name: 'PartLength', expression: '120 mm'});
    await app.execute('parameters.add', {name: 'HoleInset', expression: '12 mm'});
    await app.execute('solid.box', {
        length: 'PartLength', width: 60, height: 8, body_name: 'ParametricPlate'
    });
    await app.execute('solid.hole', {
        position: ['HoleInset', 15, 8], diameter: 8, name: 'FirstHole'
    });
    await app.execute('solid.hole', {
        position: ['PartLength - HoleInset', 45, 8],
        diameter: 8, name: 'SecondHole'
    });
    const checks = [];
    const validate = async (length, inset, stage) => {
        const doc = await app.inspect({measure: true});
        check(doc.bodies.length === 1 && doc.timeline.length === 3,
            stage + ': one plate and three features');
        check(doc.timeline.every(f => !f.error), stage + ': no failed feature');
        const b = doc.bodies[0];
        near(b.bbox.max[0] - b.bbox.min[0], length, .05, stage + ': plate length');
        near(b.bbox.max[1] - b.bbox.min[1], 60, .05, stage + ': plate width');
        near(b.bbox.max[2] - b.bbox.min[2], 8, .05, stage + ': plate thickness');
        near(b.volume_mm3, length * 480 - 2 * Math.PI * 16 * 8,
             20, stage + ': cut plate volume');
        const topo = await app.execute('model.faces', {body: 'ParametricPlate'});
        const faces = topo.faces.filter(f => f.cylinder);
        const targets = [[inset, 15], [length - inset, 45]];
        const found = [];
        for (const [x, y] of targets) {
            const matching = faces.filter(f =>
                Math.hypot(f.cylinder.origin[0] - x, f.cylinder.origin[1] - y) < .2 &&
                Math.abs(f.cylinder.radius - 4) < .1);
            check(matching.length >= 1,
                stage + ': actual Ø8 cylindrical wall at (' + x + ',' + y + ')');
            found.push([matching[0].cylinder.origin[0], matching[0].cylinder.origin[1]]);
        }
        checks.push({stage, expected_length_mm:length, inset_mm:inset, measured_holes_mm:found});
    };
    await validate(120, 12, 'initial');
    await app.execute('parameters.change', {name:'PartLength', expression:'150 mm'});
    await validate(150, 12, 'extend');
    await app.execute('parameters.change', {name:'HoleInset', expression:'15 mm'});
    await validate(150, 15, 'move inset');
    await app.execute('edit.undo');
    await validate(150, 12, 'undo inset');
    await app.execute('edit.redo');
    await validate(150, 15, 'redo inset');
    return {test:'associative hole positions on parametric plate', checks, result:'passed'};
})()
