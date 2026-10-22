(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Rounded upper plate'});
 await s.execute('solid.box',{length:44,width:31,height:12,body_name:'Plate'});
 const e=(await s.execute('model.edges',{body:'Plate'})).edges.find(e=>
  Math.abs(e.length-12)<.01&&Math.abs(e.mid[0])<.01&&Math.abs(e.mid[1])<.01);
 ok(e,'corner edge found');
 await s.execute('solid.fillet',{edges:[e.mid],radius:2.5,name:'Rounded'});
 const d=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const curves=d.geometry.top_edge_candidates.filter(p=>p.kind==='sampled_curve');
 ok(curves.some(p=>p.points_xy_mm.length>=3),'fillet top arc is sampled with multiple points');
 ok(d.svg.includes('data-upper-edge-candidate'),'rounded geometry reaches SVG');
 return {test:'3D fillet represented as sampled 2D polyline (not chord)',curves:curves.length,result:'passed'};
})()