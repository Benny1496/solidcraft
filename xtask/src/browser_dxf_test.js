(async () => {
    const app = window.solidcraft;
    const check = (ok, message) => { if (!ok) throw new Error(message); };
    const close = (a, b, tol, label) => {
        check(Number.isFinite(a) && Math.abs(a - b) <= tol,
              label + ': measured ' + a + ' expected ' + b);
    };
    // The advanced four-hole projections remain saved for later investigation.
    await app.execute('file.new', {name: 'Tiny DXF projection diagnostic'});
    await app.execute('solid.box', {
        length: 20, width: 12, height: 3, body_name: 'TinyPlate'
    });
    const camera = {
        target: [10, 6, 1.5], yaw: 0, pitch: Math.PI / 2,
        roll: 0, distance: 100, fov: 0, up: 'z'
    };
    const before = await app.inspect({measure: true});
    const out = await app.execute('file.export_view_dxf', {
        camera, hidden: 'omit', tolerance: 0.5
    });
    check(out.units === 'mm' && out.projection === 'orthographic',
          'DXF coordinates remain metric and orthographic');
    check(out.visible > 0 && typeof out.dxf === 'string' && out.dxf.length > 100,
          'DXF contains projected geometry');
    check(out.dxf.includes('ENTITIES'), 'DXF includes entities section');
    const after = await app.inspect({measure: true});
    check(after.bodies.length === 1 && after.timeline.length === 1,
          'export does not change the model');
    close(after.bodies[0].volume_mm3, before.bodies[0].volume_mm3, 0.01,
          'export preserves volume');
    return {test:'minimal DXF top-view 20 x 12 x 3 mm',
            visible:out.visible, chars:out.dxf.length, result:'passed'};
})()
