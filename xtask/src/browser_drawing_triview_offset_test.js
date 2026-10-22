(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Offset three views'});
 await s.execute('solid.box',{length:70,width:38,height:11,corner:[-20,-15,6],body_name:'Offset'});
 const d=(await s.execute('drawing.preview',{body:'Offset'})).drawing;
 const v=d.geometry.reference_views;
 ok(JSON.stringify(v.front.bounds.min_xy_mm)===JSON.stringify([-20,6]),'front physical min');
 ok(JSON.stringify(v.front.bounds.max_xy_mm)===JSON.stringify([50,17]),'front physical max');
 ok(JSON.stringify(v.right.bounds.min_xy_mm)===JSON.stringify([-15,6]),'right physical min');
 ok(JSON.stringify(v.right.bounds.max_xy_mm)===JSON.stringify([23,17]),'right physical max');
 ok(v.front.paths.length===4&&v.right.paths.length===4,'real edges from shifted model');
 return {test:'three views preserve nonzero and negative world coordinates',result:'passed'};
})()