(async () => {
    const app = window.solidcraft;
    const check = (ok, message) => {
        if (!ok) throw new Error(message);
    };
    const close = (actual, expected, tolerance, message) => {
        check(Number.isFinite(actual) && Math.abs(actual - expected) <= tolerance,
            message + ': measured ' + actual + ', expected ' + expected + ' ± ' + tolerance);
    };

    // The stock browser API smoke test finishes with one box in the document.
    // Undo only that test box, keeping this test isolated from the original smoke test.
    await app.execute('edit.undo');
    check((await app.inspect()).bodies.length === 0, 'plate test starts with no bodies');

    const length = 150;
    const width = 80;
    const thickness = 10;
    const diameter = 8;
    const edgeOffset = 15;
    const positions = [
        [edgeOffset, edgeOffset],
        [length - edgeOffset, edgeOffset],
        [edgeOffset, width - edgeOffset],
        [length - edgeOffset, width - edgeOffset],
    ];
    const plateVolume = length * width * thickness;
    const removedPerHole = Math.PI * (diameter / 2) ** 2 * thickness;

    await app.execute('solid.box', {
        length, width, height: thickness, body_name: 'Testplatte'
    });

    const plate = await app.inspect({measure: true});
    check(plate.bodies.length === 1, 'the plate is one solid body');
    check(plate.bodies[0].name === 'Testplatte', 'the plate has the requested body name');
    close(plate.bodies[0].volume_mm3, plateVolume, 0.01, 'un-drilled plate volume');
    const bbox = plate.bodies[0].bbox;
    check(bbox && Array.isArray(bbox.min) && Array.isArray(bbox.max), 'plate has a measurable bounding box');
    for (let axis = 0; axis < 3; axis++) {
        close(bbox.min[axis], 0, 0.001, 'plate lower boundary, axis ' + axis);
        close(bbox.max[axis], [length, width, thickness][axis], 0.001,
              'plate upper boundary, axis ' + axis);
    }

    const measuredAfterHoles = [];
    for (const [index, xy] of positions.entries()) {
        await app.execute('solid.hole', {
            position: [xy[0], xy[1], thickness],
            diameter
        });
        const result = await app.inspect({measure: true});
        check(result.bodies.length === 1, 'hole ' + (index + 1) + ' keeps one body');
        check(!result.timeline.some(feature => feature.error),
              'hole ' + (index + 1) + ' creates no timeline error');
        const expectedVolume = plateVolume - (index + 1) * removedPerHole;
        // The kernel measures B-reps numerically. This tolerance is much smaller
        // than the ~503 mm³ removed by a single missing through-hole.
        close(result.bodies[0].volume_mm3, expectedVolume, 30,
              'volume after through-hole ' + (index + 1));
        measuredAfterHoles.push(result.bodies[0].volume_mm3);
    }

    const final = await app.inspect({measure: true});
    check(final.timeline.length === 5, 'one box plus four hole features in the timeline');
    check(final.bodies[0].faces > 6, 'four through-holes add cylindrical faces');
    for (let axis = 0; axis < 3; axis++) {
        close(final.bodies[0].bbox.min[axis], 0, 0.001, 'final lower boundary, axis ' + axis);
        close(final.bodies[0].bbox.max[axis], [length, width, thickness][axis],
              0.001, 'final upper boundary, axis ' + axis);
    }

    return {
        test: '150 x 80 x 10 mm plate, four through-holes of diameter 8 mm',
        hole_centres_mm: positions,
        hole_spacing_mm: [length - 2 * edgeOffset, width - 2 * edgeOffset],
        expected_volume_mm3: plateVolume - 4 * removedPerHole,
        measured_volume_mm3: final.bodies[0].volume_mm3,
        measured_after_each_hole_mm3: measuredAfterHoles,
        body_count: final.bodies.length,
        timeline_features: final.timeline.length,
        result: 'passed'
    };
})()
