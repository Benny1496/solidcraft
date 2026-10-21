(async () => {
    const app = window.solidcraft;
    const check = (condition, label) => { if (!condition) throw new Error(label); };
    const near = (actual, expected, tolerance, message) => {
        check(Number.isFinite(actual) && Math.abs(actual - expected) <= tolerance,
              message + ': measured ' + actual + ', expected ' + expected);
    };
    await app.execute('file.new', {name: 'Dimension-driven manufacturing sketch'});
    await app.execute('parameters.add', {name:'DrawingWidth', expression:'80 mm'});
    await app.execute('parameters.add', {name:'DrawingDepth', expression:'50 mm'});
    await app.execute('parameters.add', {name:'DrawingThickness', expression:'8 mm'});
    await app.execute('sketch.create', {plane:'XY', name:'DimensionedBase'});
    await app.execute('sketch.rectangle.two_point', {p0:[0,0], p1:[80,50]});
    await app.execute('sketch.constraint.coincident', {a:'p1', b:'origin'});
    await app.execute('sketch.dimension', {entities:['l1'], value:'DrawingWidth'});
    await app.execute('sketch.dimension', {entities:['l2'], value:'DrawingDepth'});
    await app.execute('sketch.finish');
    await app.execute('solid.extrude', {
        distance:'DrawingThickness', body_name:'DimensionedPlate',
        name:'BaseExtrusion'
    });
    const history = [];
    const validate = async (width, depth, thickness, stage) => {
        const doc = await app.inspect({measure:true});
        check(doc.bodies.length === 1 && doc.timeline.length === 2,
              stage + ': one sketch and one extrusion');
        check(doc.timeline.every(f => !f.error), stage + ': no sketch or extrusion errors');
        const b = doc.bodies[0], span = b.bbox.max.map((v,i) => v - b.bbox.min[i]);
        for (const [i, expected] of [width,depth,thickness].entries())
            near(span[i], expected, .1, stage + ': model dimension axis ' + i);
        near(b.volume_mm3, width*depth*thickness, 1,
             stage + ': sketch-driven model volume');
        const sketch = await app.execute('sketch.inspect', {sketch:'DimensionedBase'});
        check(Array.isArray(sketch.constraints) && sketch.constraints.length >= 3,
              stage + ': dimensional sketch constraints remain');
        history.push({stage, size_mm:span, constraints:sketch.constraints.length});
    };
    await validate(80,50,8,'initial');
    await app.execute('parameters.change', {name:'DrawingWidth', expression:'110 mm'});
    await validate(110,50,8,'width changed');
    await app.execute('parameters.change', {name:'DrawingDepth', expression:'65 mm'});
    await validate(110,65,8,'depth changed');
    await app.execute('parameters.change', {name:'DrawingThickness', expression:'12 mm'});
    await validate(110,65,12,'thickness changed');
    await app.execute('edit.undo');
    await validate(110,65,8,'undo thickness');
    await app.execute('edit.redo');
    await validate(110,65,12,'redo thickness');
    return {test:'dimensional sketch associativity and extrude recomputation',
            history, result:'passed'};
})()
