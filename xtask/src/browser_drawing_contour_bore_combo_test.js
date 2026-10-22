(async()=>{
 const s=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await s.execute('file.new',{name:'Chamfer with two bore contours'});
 await s.execute('solid.box',{length:130,width:78,height:12,body_name:'Plate'});
 await s.execute('solid.hole',{name:'First',position:[30,30,12],diameter:8});
 await s.execute('solid.hole',{name:'Second',position:[95,48,12],diameter:12});
 const before=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 ok(before.geometry.cylinders.length===2,'both cylinders present before chamfer');
 const corner=(await s.execute('model.edges',{body:'Plate'})).edges.find(e=>
  Math.abs(e.length-12)<.01&&Math.abs(e.mid[0])<.01&&Math.abs(e.mid[1])<.01);
 ok(corner,'real stock corner identified');
 await s.execute('solid.chamfer',{edges:[corner.mid],distance:2,name:'Corner'});
 const pending=await s.execute('drawing.preview',{body:'Plate',previous:before});
 ok(pending.status==='stale','top contour changed without moving bores');
 ok(pending.proposed.geometry.cylinders.length===2,'both cylinders still measured');
 ok(pending.proposed.geometry.top_edge_candidates.length>=7,'outer chamfer plus bore rim candidates');
 ok(pending.proposed.geometry.top_edge_candidates.some(p=>p.kind==='sampled_curve'),'bore rim curves preserved');
 return {test:'chamfer with two independent real bore rim curves',result:'passed'};
})()