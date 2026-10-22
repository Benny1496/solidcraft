(async()=>{
 const s=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await s.execute('file.new',{name:'Independent top contours for separate parts'});
 await s.execute('solid.box',{length:60,width:35,height:8,body_name:'One'});
 await s.execute('solid.box',{length:30,width:50,height:12,corner:[100,20,0],body_name:'Two'});
 const one=(await s.execute('drawing.preview',{body:'One'})).drawing;
 const two=(await s.execute('drawing.preview',{body:'Two'})).drawing;
 ok(one.body==='One'&&two.body==='Two','explicit body selection respected');
 ok(one.source.size_mm[0]===60&&two.source.size_mm[0]===30,'separate lengths measured');
 ok(one.geometry.top_edge_candidates.length===4&&two.geometry.top_edge_candidates.length===4,'independent contours');
 const top1=one.geometry.top_edge_candidates.flatMap(p=>p.points_xy_mm);
 const top2=two.geometry.top_edge_candidates.flatMap(p=>p.points_xy_mm);
 ok(top1.every(p=>p[0]<=60.001),'body One never includes body Two edges');
 ok(top2.every(p=>p[0]>=99.999),'body Two never includes body One edges');
 return {test:'separate body-specific XY contours without accidental cross-association',result:'passed'};
})()