(async () => {
    const app = window.solidcraft;
    const assert = (ok, text) => { if (!ok) throw new Error(text); };
    const key = 'org.solidcraft.experimental_drawing';
    await app.execute('file.new', {name: 'Stored drawing sheet smoke'});
    await app.execute('parameters.add', {name:'L',expression:'90 mm'});
    await app.execute('solid.box', {length:'L',width:45,height:8,body_name:'SheetPlate'});
    const initial = await app.execute('drawing.preview',{body:'SheetPlate'});
    assert(initial.status==='created','initial drawing created');
    const old = initial.drawing;
    old.label_positions.overall_x=[46,-20];
    await app.execute('document.metadata.set',{key,value:old});
    const saved = await app.execute('document.metadata',{});
    assert(saved.metadata[key].source.size_mm[0]===90,'snapshot saved in design metadata');
    assert(saved.metadata[key].label_positions.overall_x[1]===-20,
           'manual dimension position persisted');
    await app.execute('parameters.change',{name:'L',expression:'115 mm'});
    const now = await app.execute('document.metadata',{});
    const stale = await app.execute('drawing.preview',{
        body:'SheetPlate',previous:now.metadata[key]
    });
    assert(stale.status==='stale'&&stale.requires_confirmation,
           'stored drawing stale after 3D model changed');
    assert(stale.drawing.dimensions[0].value_mm===90,
           'original drawing does not change without confirmation');
    const upd = await app.execute('drawing.preview',{
        body:'SheetPlate',previous:now.metadata[key],accept:true
    });
    assert(upd.status==='updated','user-confirmed drawing update');
    await app.execute('document.metadata.set',{key,value:upd.drawing});
    const final = await app.execute('document.metadata',{});
    assert(final.metadata[key].dimensions[0].value_mm===115,
           'confirmed dimensions stored in design');
    assert(final.metadata[key].label_positions.overall_x[1]===-20,
           'manually positioned dimension preserved');
    const again=await app.execute('drawing.preview',{
        body:'SheetPlate',previous:final.metadata[key]
    });
    assert(again.status==='current','saved drawing current after acceptance');
    return {test:'stored experimental drawing, stale confirmation and labels',
        transitions:['created','stale','updated','current'], result:'passed'};
})()
