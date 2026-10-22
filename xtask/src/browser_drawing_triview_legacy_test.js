(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Pre-three-view drawing compatibility'});
 await s.execute('solid.box',{length:60,width:38,height:9,body_name:'Plate'});
 const orig=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 const old=JSON.parse(JSON.stringify(orig));
 delete old.geometry.reference_views;delete old.source.orthographic_edge_paths;
 ok((await s.execute('drawing.preview',{body:'Plate',previous:old})).status==='current','legacy saved record remains readable');
 const bad=JSON.parse(JSON.stringify(orig));
 bad.source.orthographic_edge_paths.front[0][0][0]=1.111;
 const stale=await s.execute('drawing.preview',{body:'Plate',previous:bad});
 ok(stale.status==='stale'&&stale.requires_confirmation,'new signature detects mismatch');
 return {test:'new three-view signature detects changes without breaking old saved drawings',result:'passed'};
})()