(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Upper XY rectangle'});
 await s.execute('solid.box',{length:80,width:50,height:10,body_name:'Plate'});
 const d=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const paths=d.geometry.top_edge_candidates;
 ok(paths.length===4&&paths.every(p=>p.kind==='straight'),'four sampled top plane lines');
 ok(d.svg.includes('data-upper-edge-candidate'),'SVG explicitly renders path overlay');
 ok(d.accuracy.includes('unclassified'),'accuracy not oversold');
 ok(d.source.top_edge_paths_xy_mm.length===4,'source signature stores paths');
 return {test:'sampled B-rep top level lines in linked drawing and SVG',result:'passed'};
})()