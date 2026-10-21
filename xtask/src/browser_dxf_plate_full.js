(async () => {
    const app = window.solidcraft;
    const check = (ok, msg) => { if (!ok) throw new Error(msg); };
    const close = (a, b, tol, msg) =>
        check(Number.isFinite(a) && Math.abs(a - b) <= tol,
              msg + ': measured ' + a + ', expected ' + b);
    const camera = (yaw, pitch) => ({
        target: [75, 40, 5], yaw, pitch, roll: 0,
        distance: 350, fov: 0, up: 'z'
    });
    // Parse the geometry in the DXF ENTITIES section, not the header
    // ($EXTMIN/$EXTMAX), whose values must not count as geometry.
    const geometry = (dxf) => {
        check(typeof dxf === 'string' && dxf.length > 100, 'nonempty DXF content');
        const lines = dxf.replace(/\r/g, '').split('\n');
        const entities = [];
        let inEntities = false, section = false, entity = null;
        for (let i = 0; i + 1 < lines.length; i += 2) {
            const code = lines[i].trim(), value = lines[i + 1].trim();
            if (code === '0' && value === 'SECTION') { section = true; continue; }
            if (section && code === '2') {
                inEntities = value === 'ENTITIES';
                section = false;
                continue;
            }
            if (code === '0' && value === 'ENDSEC') {
                if (inEntities && entity) entities.push(entity);
                inEntities = false; entity = null; continue;
            }
            if (!inEntities) continue;
            if (code === '0') {
                if (entity) entities.push(entity);
                entity = {type: value, layer: '', points: []};
            } else if (entity) {
                if (code === '8') entity.layer = value;
                if (code === '10') entity.x = Number(value);
                if (code === '20' && Number.isFinite(entity.x)) {
                    entity.points.push([entity.x, Number(value)]);
                    entity.x = undefined;
                }
                if (code === '11') entity.endX = Number(value);
                if (code === '21' && Number.isFinite(entity.endX)) {
                    entity.points.push([entity.endX, Number(value)]);
                    entity.endX = undefined;
                }
            }
        }
        return entities;
    };

    const before = await app.inspect({measure: true});
    check(before.bodies.length === 1 && before.timeline.length === 5,
          'projection starts with the complete four-hole plate');
    const results = [];
    const views = [
        ['top', 0, Math.PI / 2, 150, 80],
        ['front', 0, 0, 150, 10],
        ['right', -Math.PI / 2, 0, 80, 10]
    ];
    for (const [name, yaw, pitch, expectedWidth, expectedHeight] of views) {
        const out = await app.execute('file.export_view_dxf', {
            camera: camera(yaw, pitch), hidden: 'omit', tolerance: 0.05
        });
        check(out.projection === 'orthographic' && out.units === 'mm',
              name + ': true-size metric projection');
        check(out.visible > 0 && out.hidden === 0, name + ': visible geometry');
        check(out.stats && out.stats.bodies === 1, name + ': one projected body');
        const entities = geometry(out.dxf).filter(e => e.layer === 'VISIBLE');
        const pts = entities.flatMap(e => e.points).filter(p => p.every(Number.isFinite));
        check(pts.length >= 8, name + ': enough 2D geometric vertices');
        const xs = pts.map(p => p[0]), ys = pts.map(p => p[1]);
        const width = Math.max(...xs) - Math.min(...xs);
        const height = Math.max(...ys) - Math.min(...ys);
        close(width, expectedWidth, 0.3, name + ': orthographic width mm');
        close(height, expectedHeight, 0.3, name + ': orthographic height mm');
        results.push({view: name, projected_size_mm: [width, height],
                      entities: entities.length});
    }

    const hidden = await app.execute('file.export_view_dxf', {
        camera: camera(0, Math.PI / 2), hidden: 'dashed', tolerance: 0.05
    });
    check(hidden.projection === 'orthographic' && hidden.units === 'mm',
          'hidden-line version keeps metric projection');
    check(typeof hidden.dxf === 'string' && hidden.dxf.includes('HIDDEN'),
          'hidden-line DXF defines a HIDDEN layer');
    const after = await app.inspect({measure: true});
    check(after.timeline.length === before.timeline.length &&
          after.bodies.length === 1,
          'DXF exports do not modify the model features');
    close(after.bodies[0].volume_mm3, before.bodies[0].volume_mm3, 0.01,
          'DXF exports do not change the body volume');
    return {test: 'orthographic 2D DXF views', views: results,
            hidden_layer: 'available', result: 'passed'};
})()
