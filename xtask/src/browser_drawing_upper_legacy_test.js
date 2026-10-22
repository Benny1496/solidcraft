(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Legacy drawing without upper paths'});
 await s.execute('solid.box',{length:75,width:45,height:7,body_name:'Plate'});
 const first=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const legacy=JSON.parse(JSON.stringify(first));
 delete legacy.source.top_edge_paths_xy_mm;
 delete legacy.geometry.top_edge_candidates;
 ok((await s.execute('drawing.preview',{body:'Plate',previous:legacy})).status==='current','legacy compatible');
 const rejected=JSON.parse(JSON.stringify(first));
 rejected.source.top_edge_paths_xy_mm=[[[1,2],['bad',3]]];
 const stale=await s.execute('drawing.preview',{body:'Plate',previous:rejected});
 ok(stale.status==='stale'&&stale.requires_confirmation,'invalid path signatures never falsely current');
 return {test:'old documents compatible; malformed new signatures rejected as stale',result:'passed'};
})()