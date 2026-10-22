(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Top contour revision'});
 await s.execute('parameters.add',{name:'Len',expression:'60 mm'});
 await s.execute('solid.box',{length:'Len',width:45,height:8,body_name:'Plate'});
 const first=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 first.label_positions.overall_x=[36,-21];
 await s.execute('parameters.change',{name:'Len',expression:'90 mm'});
 const stale=await s.execute('drawing.preview',{body:'Plate',previous:first});
 ok(stale.status==='stale'&&stale.proposed.geometry.top_edge_candidates.length===4,'revised paths proposed only');
 ok(stale.drawing.label_positions.overall_x[1]===-21,'manual label retained');
 await s.execute('edit.undo');
 ok((await s.execute('drawing.preview',{body:'Plate',previous:first})).status==='current','undo same snapshot');
 await s.execute('edit.redo');
 const yes=await s.execute('drawing.preview',{body:'Plate',previous:first,accept:true});
 ok(yes.status==='updated'&&yes.drawing.label_positions.overall_x[1]===-21,'confirmed revision preserves layout');
 return {test:'top path signature participates in undo redo and confirmation',result:'passed'};
})()