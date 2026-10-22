(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Thickness affects front and right'});
 await s.execute('parameters.add',{name:'T',expression:'8 mm'});
 await s.execute('solid.box',{length:80,width:50,height:'T',body_name:'Plate'});
 const original=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 await s.execute('parameters.change',{name:'T',expression:'13 mm'});
 const changed=await s.execute('drawing.preview',{body:'Plate',previous:original});
 ok(changed.status==='stale','height triggers stale dialog');
 ok(changed.proposed.geometry.reference_views.front.bounds.max_xy_mm[1]===13,'front updated');
 ok(changed.proposed.geometry.reference_views.right.bounds.max_xy_mm[1]===13,'right updated');
 await s.execute('edit.undo');
 ok((await s.execute('drawing.preview',{body:'Plate',previous:original})).status==='current','undo restores all');
 await s.execute('edit.redo');
 ok((await s.execute('drawing.preview',{body:'Plate',previous:original})).status==='stale','redo stales again');
 return {test:'thickness edit, undo and redo correctly update front and side',result:'passed'};
})()