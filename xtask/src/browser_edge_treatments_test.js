(async () => {
    const app=window.solidcraft;
    const assert=(ok,msg)=>{if(!ok)throw new Error(msg);};
    const cases=[
        {name:'fillet',command:'solid.fillet',params:{radius:2.5}},
        {name:'chamfer',command:'solid.chamfer',params:{distance:2.5}}
    ];
    const results=[];
    for(const item of cases) {
        await app.execute('file.new', {name:'Manufacturing '+item.name});
        await app.execute('solid.box', {
            length:40,width:30,height:12,
            body_name:'MachinedBlock',name:'Stock'
        });
        const before=await app.inspect({measure:true});
        const edges=await app.execute('model.edges', {body:'MachinedBlock'});
        assert(Array.isArray(edges.edges)&&edges.edges.length>=12,
               item.name+': block contains measurable edges');
        // Use a real edge midpoint from the model, not a hardcoded guess.
        const chosen=edges.edges.find(e=>
            Array.isArray(e.mid) && e.mid.length===3 &&
            Math.abs(e.length-12)<.01 &&
            Math.abs(e.mid[0])<.01 && Math.abs(e.mid[1])<.01);
        assert(chosen,item.name+': locate vertical stock edge');
        await app.execute(item.command,{
            edges:[chosen.mid],...item.params,name:'EdgeFinish'
        });
        const after=await app.inspect({measure:true});
        assert(after.bodies.length===1&&after.timeline.length===2,
               item.name+': one solid and two timeline features');
        assert(after.timeline.every(f=>!f.error),
               item.name+': no feature error');
        const delta=before.bodies[0].volume_mm3-after.bodies[0].volume_mm3;
        assert(Number.isFinite(delta)&&delta>1&&delta<500,
               item.name+': a small amount of stock was removed, delta='+delta);
        const faces=await app.execute('model.faces',{body:'MachinedBlock'});
        assert(faces.faces.length>6,item.name+': generated new geometry');
        results.push({treatment:item.name, removed_volume_mm3:delta,
                      surface_faces:faces.faces.length});
    }
    return {test:'fillet and chamfer on selected real model edges',
            results, result:'passed'};
})()
