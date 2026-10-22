(async()=>{
 const s=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await s.execute('file.new',{name:'Fillet revision undo and redo'});
 await s.execute('solid.box',{length:52,width:37,height:10,body_name:'Plate'});
 const old=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const edges=(await s.execute('model.edges',{body:'Plate'})).edges;
 const corner=edges.find(e=>Math.abs(e.length-10)<.01&&Math.abs(e.mid[0])<.01&&Math.abs(e.mid[1])<.01);
 ok(corner,'actual vertical corner edge located');
 await s.execute('solid.fillet',{edges:[corner.mid],radius:2,name:'RoundEdge'});
 const pending=await s.execute('drawing.preview',{body:'Plate',previous:old});
 ok(pending.status==='stale'&&pending.requires_confirmation,'rounding requires update');
 ok(pending.proposed.geometry.top_edge_candidates.some(p=>p.kind==='sampled_curve'),'proposed arc visible');
 ok(JSON.stringify(pending.drawing)===JSON.stringify(old),'old version preserved');
 await s.execute('edit.undo');
 ok((await s.execute('drawing.preview',{body:'Plate',previous:old})).status==='current','undo restores stored contour');
 await s.execute('edit.redo');
 ok((await s.execute('drawing.preview',{body:'Plate',previous:old})).status==='stale','redo requests confirmation again');
 const yes=await s.execute('drawing.preview',{body:'Plate',previous:old,accept:true});
 ok(yes.status==='updated','accept new round contour');
 return {test:'3D corner rounding and contour-only stale/undo/redo',result:'passed'};
})()