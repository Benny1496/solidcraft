(async()=>{
 const s=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 const cases=[[23,16,5],[40,35,9],[125,75,7],[180,90,15]];
 for(const [x,y,z] of cases) {
   await s.execute('file.new',{name:'Contour case '+x});
   await s.execute('solid.box',{length:x,width:y,height:z,body_name:'Plate'});
   const d=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
   ok(d.geometry.top_edge_candidates.length===4,'four top-level lines: '+x);
   ok(d.geometry.top_edge_candidates.every(e=>e.kind==='straight'&&e.points_xy_mm.length===2),'straight samples: '+x);
   const list=d.geometry.top_edge_candidates.flatMap(p=>p.points_xy_mm);
   const min=[Math.min(...list.map(p=>p[0])),Math.min(...list.map(p=>p[1]))];
   const max=[Math.max(...list.map(p=>p[0])),Math.max(...list.map(p=>p[1]))];
   ok(min[0]===0&&min[1]===0&&max[0]===x&&max[1]===y,'measured XY extent: '+x);
 }
 return {test:'four sizes of real XY upper-level contour paths',cases:cases.length,result:'passed'};
})()