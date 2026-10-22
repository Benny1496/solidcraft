(async()=>{
 const sc=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await sc.execute('file.new',{name:'Straight B-rep projection of rectangle'});
 await sc.execute('solid.box',{length:140,width:65,height:9,body_name:'Plate'});
 const before=await sc.inspect({measure:true});
 const p=await sc.execute('drawing.projected_edges',{body:'Plate'});
 ok(p.accuracy==='unclassified_straight_brep_edges_only','correctly bounded accuracy');
 ok(p.segments.length===4,'4 distinct rectangle sides');
 ok(p.bounds.max_xy_mm[0]===140&&p.bounds.max_xy_mm[1]===65,'actual measured box bounds');
 ok(p.statistics.source_edges>=12,'measured actual 3D B-rep edges');
 ok(p.statistics.point_projections_skipped>=4,'upright edges do not become points');
 ok(p.statistics.duplicate_projections_skipped>=4,'top and bottom duplicate sides removed');
 const total=p.segments.reduce((n,e)=>n+Math.hypot(e.start_xy_mm[0]-e.end_xy_mm[0],e.start_xy_mm[1]-e.end_xy_mm[1]),0);
 ok(Math.abs(total-410)<.005,'XY perimeter 410 mm from true 3D segments');
 ok((await sc.inspect({measure:true})).timeline.length===before.timeline.length,'read-only command');
 return {test:'four real rectangle edge projections, deduplication, no timeline changes',lines:p.segments.length,result:'passed'};
})()