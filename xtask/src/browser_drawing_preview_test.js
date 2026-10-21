(async () => {
    const app = window.solidcraft;
    const check = (ok,msg) => { if (!ok) throw new Error(msg); };
    const near = (v,w,t,msg) =>
        check(typeof v === 'number' && Math.abs(v-w) < t,
              msg+': got '+v+', expected '+w);
    await app.execute('file.new', {name:'Drawing preview - manufacturing plate'});
    await app.execute('parameters.add',{name:'DrawingLength',expression:'150 mm'});
    await app.execute('solid.box',{
        length:'DrawingLength',width:80,height:10,
        body_name:'DrawPlate',name:'Stock'
    });
    const centres=[[15,15],[135,15],[15,65],[135,65]];
    for (const [i,point] of centres.entries())
        await app.execute('solid.hole',{
            position:[point[0]===135?'DrawingLength - 15 mm':15,point[1],10],
            diameter:8,name:'Hole'+(i+1)
        });
    const initialState=await app.inspect({measure:true});
    const start=await app.execute('drawing.preview',{body:'DrawPlate'});
    check(start.status==='created' && !start.requires_confirmation,
        'first drawing was created without requiring an update');
    const first=start.drawing;
    check(first.format==='solidcraft-drawing-preview/1',
        'the drawing is explicitly versioned');
    check(first.geometry.outline.type==='rectangle_envelope',
        'the bounded preview labels its envelope');
    check(first.geometry.cylinders.length===4,
        'the drawing reports four real bores');
    near(first.dimensions[0].value_mm,150,.01,'drawing X dimension');
    near(first.dimensions[1].value_mm,80,.01,'drawing Y dimension');
    for (const [x,y] of centres)
        check(first.geometry.cylinders.some(c=>
            Math.abs(c.center_mm[0]-x)<.02 &&
            Math.abs(c.center_mm[1]-y)<.02 &&
            Math.abs(c.diameter_mm-8)<.02),
            'drawing reports actual Ø8 bore at ('+x+','+y+')');
    check((await app.inspect({measure:true})).timeline.length===initialState.timeline.length,
        'drawing generation did not change model timeline');

    // User-positioned dimension labels remain untouched during update.
    first.label_positions.overall_x=[99,-22];
    first.label_positions.overall_y=[-23,40];
    const current=await app.execute('drawing.preview',{
        body:'DrawPlate',previous:first
    });
    check(current.status==='current'&&!current.requires_confirmation,
        'unchanged model is current');

    await app.execute('parameters.change',{name:'DrawingLength',expression:'180 mm'});
    const changed=await app.execute('drawing.preview',{body:'DrawPlate',previous:first});
    check(changed.status==='stale'&&changed.requires_confirmation,
        'model change requires drawing update confirmation');
    check(changed.drawing===first||changed.drawing.dimensions[0].value_mm===150,
        'the old drawing stays unmodified');
    near(changed.proposed.dimensions[0].value_mm,180,.01,
        'proposed updated X dimension');
    check(changed.proposed.geometry.cylinders.some(c=>
        Math.abs(c.center_mm[0]-165)<.02&&Math.abs(c.center_mm[1]-65)<.02),
        'hole follows 3D model in proposed drawing');
    check(changed.proposed.label_positions.overall_x[0]===99 &&
          changed.proposed.label_positions.overall_x[1]===-22,
          'manually positioned X dimension survives recompute');
    const accepted=await app.execute('drawing.preview',{
        body:'DrawPlate',previous:first,accept:true
    });
    check(accepted.status==='updated'&&!accepted.requires_confirmation,
        'explicit user confirmation updates the drawing');
    near(accepted.drawing.dimensions[0].value_mm,180,.01,
        'accepted drawing X dimension');
    check(accepted.drawing.label_positions.overall_y[0]===-23,
        'manually positioned Y dimension survives confirmation');

    // A second check must compare with the accepted revision, not the old one.
    const already=await app.execute('drawing.preview',{
        body:'DrawPlate',previous:accepted.drawing
    });
    check(already.status==='current'&&!already.requires_confirmation,
        'accepted drawing matches its own source model');
    const unchanged=await app.inspect({measure:true});
    check(unchanged.timeline.length===initialState.timeline.length,
        'preview calls did not create CAD features');
    return {
        test:'model-linked read-only drawing, stale warning, user acceptance and layout',
        before_mm:150,after_mm:180,bores:first.geometry.cylinders.length,
        retained_label_positions:true,statuses:[
            start.status,current.status,changed.status,accepted.status,already.status
        ],result:'passed'
    };
})()
