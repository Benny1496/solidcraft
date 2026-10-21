(async () => {
    const app = window.solidcraft;
    const check = (ok, label) => { if (!ok) throw new Error(label); };
    const styles = [
        {type:'counterbore', diameter:6, cb_diameter:12, cb_depth:3},
        {type:'countersink', diameter:6, cs_diameter:12, cs_angle:'90 deg'},
        {type:'simple', diameter:6}
    ];
    const stock = 70 * 50 * 12, pilotRadius = 3;
    const result = [];
    for (const style of styles) {
        await app.execute('file.new', {name:'Manufacturing hole - '+style.type});
        await app.execute('solid.box', {
            length:70, width:50, height:12, body_name:'HoleStylePlate'
        });
        await app.execute('solid.hole', {
            position:[35,25,12], ...style, name:'StyleHole'
        });
        const doc = await app.inspect({measure:true});
        check(doc.bodies.length===1 && doc.timeline.length===2,
              style.type+': one body and two timeline features');
        check(doc.timeline.every(f=>!f.error),style.type+': feature timeline remains valid');
        const bore = await app.execute('model.faces', {body:'HoleStylePlate'});
        const walls = bore.faces.filter(f=>f.cylinder &&
            Math.hypot(f.cylinder.origin[0]-35,f.cylinder.origin[1]-25)<.2);
        check(walls.some(f=>Math.abs(f.cylinder.radius-pilotRadius)<.1),
              style.type+': Ø6 drilled pilot found on physical geometry');
        if (style.type==='counterbore')
            check(walls.some(f=>Math.abs(f.cylinder.radius-6)<.1),
                  'counterbore has physical Ø12 cylindrical recess');
        const removed = stock-doc.bodies[0].volume_mm3;
        check(Number.isFinite(removed) && removed>300,
              style.type+': material was removed');
        const pilotVolume=Math.PI*pilotRadius**2*12;
        let expected = pilotVolume;
        if(style.type==='counterbore') expected += Math.PI*(6**2-pilotRadius**2)*3;
        if(style.type==='countersink') {
            const h=6-pilotRadius;
            const frustum=Math.PI*h/3*(6**2+6*pilotRadius+pilotRadius**2);
            expected += frustum - Math.PI*pilotRadius**2*h;
        }
        check(Math.abs(removed-expected)<20,
              style.type+': measured volume difference '+removed+
              ' not close to analytic '+expected);
        result.push({type:style.type, actual_removed_mm3:removed,
                     expected_removed_mm3:expected,cylindrical_faces:walls.length});
    }
    return {test:'simple, counterbore, countersink machining holes',
            styles:result,result:'passed'};
})()
