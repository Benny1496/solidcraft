(async()=>{
 const sc=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await sc.execute('file.new',{name:'Projected edges with model revisions'});
 await sc.execute('parameters.add',{name:'EdgeLength',expression:'45 mm'});
 await sc.execute('solid.box',{length:'EdgeLength',width:27,height:6,body_name:'Plate'});
 const get=async()=>sc.execute('drawing.projected_edges',{body:'Plate'});
 const first=await get();
 const unchanged=await get();
 ok(JSON.stringify(first.segments)===JSON.stringify(unchanged.segments),'repeatable ordering');
 await sc.execute('parameters.change',{name:'EdgeLength',expression:'70 mm'});
 const newer=await get();
 ok(newer.bounds.max_xy_mm[0]===70,'parametric length changes projected extent');
 ok(JSON.stringify(first.segments)!==JSON.stringify(newer.segments),'B-rep-derived segment values change');
 await sc.execute('edit.undo');
 ok(JSON.stringify((await get()).segments)===JSON.stringify(first.segments),'undo restores projection');
 await sc.execute('edit.redo');
 ok(JSON.stringify((await get()).segments)===JSON.stringify(newer.segments),'redo restores new projection');
 return {test:'projected B-rep edge geometry follows parameter change, undo and redo',result:'passed'};
})()