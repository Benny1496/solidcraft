(async()=>{
 const sc=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await sc.execute('file.new',{name:'Straight edges are not false circular arcs'});
 await sc.execute('solid.box',{length:95,width:70,height:8,body_name:'Plate'});
 for(const [i,x,y] of [[1,20,20],[2,75,50]])await sc.execute('solid.hole',{
   name:'Bore'+i,position:[x,y,8],diameter:8});
 const p=await sc.execute('drawing.projected_edges',{body:'Plate'});
 ok(p.segments.length>=4,'stock still has projected straight edges');
 ok(p.statistics.curved_or_nonstraight_skipped>=2,'circular B-rep edges are excluded, not drawn as false straight chords');
 const d=await sc.execute('drawing.preview',{body:'Plate'});
 ok(d.drawing.geometry.cylinders.length===2,'existing measured hole circles retained');
 ok(!('edge_candidates' in d.drawing.geometry),'experimental candidates do not silently enter stored drawing');
 return {test:'true bore circles handled by old drawing, curved edges not faked',result:'passed'};
})()