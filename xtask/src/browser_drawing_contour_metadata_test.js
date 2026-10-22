(async()=>{
 const s=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 const key='org.solidcraft.experimental_drawing';
 await s.execute('file.new',{name:'Stored contour with bore'});
 await s.execute('solid.box',{length:88,width:51,height:9,body_name:'Plate'});
 await s.execute('solid.hole',{name:'Drill',position:[25,20,9],diameter:8});
 const d=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 d.label_positions.overall_x=[44,-19];
 await s.execute('document.metadata.set',{key,value:d});
 const saved=(await s.execute('document.metadata',{})).metadata[key];
 ok(saved.geometry.top_edge_candidates.length>=4,'top paths persisted in document metadata');
 ok(saved.source.top_edge_paths_xy_mm.length===d.source.top_edge_paths_xy_mm.length,'source contour signature persisted');
 const current=await s.execute('drawing.preview',{body:'Plate',previous:saved});
 ok(current.status==='current','stored contour matches the actual model');
 ok(current.drawing.label_positions.overall_x[1]===-19,'manual dimension preserved');
 return {test:'top curve metadata and revision signatures remain stored and current',result:'passed'};
})()