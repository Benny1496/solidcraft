(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)},key='org.solidcraft.experimental_drawing';
 await s.execute('file.new',{name:'Persisted workshop three views'});
 await s.execute('solid.box',{length:110,width:60,height:10,body_name:'Plate'});
 const first=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 first.label_positions.overall_x=[55,-20];
 await s.execute('document.metadata.set',{key,value:first});
 const saved=(await s.execute('document.metadata',{})).metadata[key];
 ok(saved.geometry.reference_views.front.paths.length===4,'saved front geometry');
 ok(saved.geometry.reference_views.right.paths.length===4,'saved right geometry');
 ok(saved.label_positions.overall_x[1]===-20,'label placement saved');
 ok((await s.execute('drawing.preview',{body:'Plate',previous:saved})).status==='current','stored view is current');
 return {test:'front and side reference panels persist with manual top label',result:'passed'};
})()