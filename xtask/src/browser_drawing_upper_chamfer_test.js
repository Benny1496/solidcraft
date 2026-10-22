(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Upper chamfer'});
 await s.execute('solid.box',{length:50,width:35,height:12,body_name:'Plate'});
 const first=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const edges=(await s.execute('model.edges',{body:'Plate'})).edges;
 const e=edges.find(x=>Math.abs(x.length-12)<.01&&Math.abs(x.mid[0])<.01&&Math.abs(x.mid[1])<.01);
 ok(e,'vertical corner edge');
 await s.execute('solid.chamfer',{edges:[e.mid],distance:3,name:'Chamfer'});
 const st=await s.execute('drawing.preview',{body:'Plate',previous:first});
 ok(st.status==='stale'&&st.requires_confirmation,'same-bounds chamfer prompts update');
 const paths=st.proposed.geometry.top_edge_candidates;
 ok(paths.length>=5,'corner requires extra top level contour line');
 const diag=paths.find(p=>p.points_xy_mm.length===2&&
  Math.abs(p.points_xy_mm[0][0]-p.points_xy_mm[1][0])>.02&&
  Math.abs(p.points_xy_mm[0][1]-p.points_xy_mm[1][1])>.02);
 ok(diag,'diagonal chamfer contour shown');
 ok(JSON.stringify(st.drawing)===JSON.stringify(first),'old sheet retained');
 const accepted=await s.execute('drawing.preview',{body:'Plate',previous:first,accept:true});
 ok(accepted.status==='updated'&&accepted.drawing.svg.includes('data-upper-edge-candidate'),'explicit accept draws');
 return {test:'chamfer top shape, unchanged bounds, stale dialog protocol',result:'passed'};
})()