(async () => {
 const sc=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await sc.execute('file.new',{name:'3D drilling changes reflected in drawing'});
 await sc.execute('solid.box',{length:120,width:60,height:10,body_name:'Plate'});
 const zero=await sc.execute('drawing.preview',{body:'Plate'});
 ok(zero.drawing.geometry.cylinder_dimensions.length===0,'empty hole dimension table');
 await sc.execute('solid.hole',{name:'One',position:[20,20,10],diameter:6});
 const first=await sc.execute('drawing.preview',{body:'Plate',previous:zero.drawing});
 ok(first.status==='stale'&&first.proposed.geometry.cylinder_dimensions.length===1,'added cylinder stales 2D');
 const accepted=await sc.execute('drawing.preview',{body:'Plate',previous:zero.drawing,accept:true});
 ok(accepted.status==='updated'&&accepted.drawing.geometry.cylinder_dimensions.length===1,'accepted new cylinder');
 await sc.execute('solid.hole',{name:'Two',position:[90,40,10],diameter:12});
 const second=await sc.execute('drawing.preview',{body:'Plate',previous:accepted.drawing});
 ok(second.status==='stale'&&second.proposed.geometry.cylinder_dimensions.length===2,'second cylinder appears as stale');
 await sc.execute('edit.undo');
 ok((await sc.execute('drawing.preview',{body:'Plate',previous:accepted.drawing})).status==='current',
   'undo removed cylinder restores drawing current');
 await sc.execute('edit.redo');
 const final=await sc.execute('drawing.preview',{body:'Plate',previous:accepted.drawing,accept:true});
 ok(final.status==='updated'&&final.drawing.geometry.cylinder_dimensions.length===2,'redo cylinder confirmed');
 for(const c of final.drawing.geometry.cylinder_dimensions)
   ok(c.reference_status==='unverified','no fabricated persistent bore name');
 return {test:'0→1→2 bores, stale status and undo/redo of 3D features',result:'passed'};
})()
