(async()=>{
 const s=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await s.execute('file.new',{name:'Browser numeric contour roundtrip'});
 await s.execute('solid.box',{length:96,width:40,height:6,body_name:'Plate'});
 const first=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const parsed=JSON.parse(JSON.stringify(first));
 ok((await s.execute('drawing.preview',{body:'Plate',previous:parsed})).status==='current','serialized numeric signature remains current');
 const mutated=JSON.parse(JSON.stringify(parsed));
 mutated.source.top_edge_paths_xy_mm[0][0][0]+=0.5;
 const result=await s.execute('drawing.preview',{body:'Plate',previous:mutated});
 ok(result.status==='stale'&&result.requires_confirmation,'actual numeric source change detected');
 ok(JSON.stringify(result.drawing)===JSON.stringify(mutated),'declined snapshot not modified');
 return {test:'numeric JSON serialization of contour signature and changed coordinate',result:'passed'};
})()