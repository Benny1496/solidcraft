(async()=>{
 const s=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await s.execute('file.new',{name:'Chamfer contour manual dimension layout'});
 await s.execute('solid.box',{length:65,width:48,height:12,body_name:'Plate'});
 const old=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 old.label_positions.overall_x=[32,-21];
 old.label_positions.overall_y=[-18,24];
 const edge=(await s.execute('model.edges',{body:'Plate'})).edges.find(e=>
   Math.abs(e.length-12)<.01&&Math.abs(e.mid[0])<.01&&Math.abs(e.mid[1])<.01);
 ok(edge,'vertical edge');
 await s.execute('solid.chamfer',{edges:[edge.mid],distance:3,name:'Relief'});
 const pending=await s.execute('drawing.preview',{body:'Plate',previous:old});
 ok(pending.status==='stale','same bounding box but real corner changed');
 ok(pending.drawing.geometry.top_edge_candidates.length===4,'old four lines retained');
 ok(pending.proposed.geometry.top_edge_candidates.length>=5,'new chamfer geometry proposed');
 ok(pending.proposed.label_positions.overall_x[0]===32&&pending.proposed.label_positions.overall_y[0]===-18,'both manual dimension placements retained');
 const yes=await s.execute('drawing.preview',{body:'Plate',previous:old,accept:true});
 ok(yes.status==='updated','confirmed update');
 ok(yes.drawing.label_positions.overall_x[1]===-21&&yes.drawing.label_positions.overall_y[1]===24,'accepted placement retained');
 return {test:'contour-only chamfer and both manual dimension placements',result:'passed'};
})()