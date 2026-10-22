(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Upper bores'});
 await s.execute('solid.box',{length:120,width:70,height:8,body_name:'Plate'});
 await s.execute('solid.hole',{name:'BoreA',position:[25,25,8],diameter:8});
 await s.execute('solid.hole',{name:'BoreB',position:[90,45,8],diameter:12});
 const d=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 ok(d.geometry.cylinders.length===2,'two verified cylinder circles retained');
 const curves=d.geometry.top_edge_candidates.filter(p=>p.kind==='sampled_curve');
 ok(curves.length>=2,'top rim circular arcs survive as sampled edges');
 ok(curves.every(c=>c.points_xy_mm.length>=3),'not fake two-point chords');
 return {test:'top level rim curves coexist with certified 3D cylinder dimensions',curves:curves.length,result:'passed'};
})()