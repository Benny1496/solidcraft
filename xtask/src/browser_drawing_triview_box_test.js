(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Three views of a workshop plate'});
 await s.execute('solid.box',{length:90,width:45,height:8,body_name:'Plate'});
 const d=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const v=d.geometry.reference_views;
 ok(d.geometry.top_edge_candidates.length===4,'top four edges');
 ok(v.front.paths.length===4&&v.right.paths.length===4,'four true plane edges each');
 ok(v.front.bounds.max_xy_mm[0]===90&&v.front.bounds.max_xy_mm[1]===8,'front width and height');
 ok(v.right.bounds.max_xy_mm[0]===45&&v.right.bounds.max_xy_mm[1]===8,'right width and height');
 ok(v.accuracy.includes('unclassified'),'views must not claim HLR');
 return {test:'3D box generates explicit top, front and right planar samples',result:'passed'};
})()