(async () => {
    const app = window.solidcraft;
    const assert = (ok, msg) => { if (!ok) throw new Error(msg); };
    const near = (actual, expected, tol, msg) => assert(
        Number.isFinite(actual) && Math.abs(actual - expected) <= tol,
        msg + ': actual=' + actual + ', expected=' + expected
    );
    await app.execute('file.new', {name:'Timeline edit and stable face reference'});
    await app.execute('solid.box', {
        length:40, width:30, height:10,
        body_name:'EditBlock', name:'BaseBlock'
    });
    const first = await app.execute('model.faces', {body:'EditBlock'});
    const exactNames = first.faces.filter(f =>
        f.exact === true && typeof f.name === 'string').map(f=>f.name);
    assert(exactNames.length >= 2, 'initial block has certified named faces');
    const checkpoints = [];
    const verify = async (length, height, stage) => {
        const d = await app.inspect({measure:true});
        assert(d.bodies.length === 1 && d.timeline.length === 1,
               stage + ': one body and one timeline feature');
        assert(!d.timeline.some(f=>f.error), stage + ': no timeline errors');
        const b=d.bodies[0], size=b.bbox.max.map((v,i)=>v-b.bbox.min[i]);
        near(size[0],length,.01,stage+' X');
        near(size[1],30,.01,stage+' Y');
        near(size[2],height,.01,stage+' Z');
        near(b.volume_mm3,length*30*height,.1,stage+' volume');
        const faces = await app.execute('model.faces', {body:'EditBlock'});
        const names = new Set(faces.faces.filter(f=>f.exact===true).map(f=>f.name));
        assert(exactNames.filter(name=>names.has(name)).length >= 2,
               stage + ': certified face names survive ordinary size edits');
        checkpoints.push({stage, size_mm:size,
                          preserved_exact_names:exactNames.filter(n=>names.has(n)).length});
    };
    await verify(40,10,'initial');
    const feature=await app.execute('timeline.inspect',{feature:'BaseBlock'});
    assert(feature && feature.definition, 'timeline inspect returns editable feature definition');
    await app.execute('timeline.edit', {
        feature:'BaseBlock', set:{length:'65',height:'16'}
    });
    await verify(65,16,'feature edited');
    await app.execute('edit.undo');
    await verify(40,10,'undo');
    await app.execute('edit.redo');
    await verify(65,16,'redo');
    return {test:'timeline editing and persistent model-face names',
            checkpoints, result:'passed'};
})()
