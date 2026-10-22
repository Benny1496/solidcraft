(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Chamfered 3 view plate'});
 await s.execute('solid.box',{length:70,width:50,height:12,body_name:'Plate'});
 const first=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const edges=(await s.execute('model.edges',{body:'Plate'})).edges;
 const corner=edges.find(e=>Math.abs(e.length-12)<.01&&Math.abs(e.mid[0])<.01&&Math.abs(e.mid[1])<.01);
 ok(corner,'corner found');
 await s.execute('solid.chamfer',{edges:[corner.mid],distance:3,name:'MachinedCorner'});
 const stale=await s.execute('drawing.preview',{body:'Plate',previous:first});
 ok(stale.status==='stale','corner changed though bounding dimensions same');
 ok(stale.proposed.geometry.top_edge_candidates.length>=5,'chamfer reaches top');
 ok(stale.proposed.geometry.reference_views.front.bounds.max_xy_mm[0]===70,'front bounds retained');
 ok(stale.proposed.geometry.reference_views.right.bounds.max_xy_mm[0]===50,'side bounds retained');
 return {test:'chamfer modifies 3D edge data without losing reference panel bounds',result:'passed'};
})()