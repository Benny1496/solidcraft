(async()=>{
 const app=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await app.execute('file.new',{name:'RFC860 parametric two-bore undo redo'});
 await app.execute('parameters.add',{name:'BoreX',expression:'90 mm'});
 await app.execute('solid.box',{length:135,width:75,height:11,body_name:'Plate'});
 await app.execute('solid.hole',{name:'First',position:[20,20,11],diameter:6});
 await app.execute('solid.hole',{name:'Moving',position:['BoreX',55,11],diameter:6});
 let d=(await app.execute('drawing.preview',{body:'Plate'})).drawing;
 const faces=d.geometry.cylinder_dimensions.map(x=>x.eligible_face_name);
 const reply=await app.execute('drawing.annotation.add',{body:'Plate',previous:d,face_name:faces[0],second_face_name:faces[1],kind:'center_distance',position_mm:[74,68]});
 d=reply.drawing;const id=reply.annotation_id;
 const original=d.annotations[0].value_mm;
 await app.execute('parameters.change',{name:'BoreX',expression:'110 mm'});
 let pending=await app.execute('drawing.preview',{body:'Plate',previous:d});
 ok(pending.status==='stale'&&pending.drawing.annotations[0].value_mm===original,'change stale without mutation');
 const updated=pending.proposed.annotations[0].value_mm;
 ok(Math.abs(updated-Math.hypot(90,35))<.002,'new centre distance from real 3D geometry');
 ok(pending.proposed.label_positions[id][0]===74,'label unchanged after update');
 await app.execute('edit.undo');
 ok((await app.execute('drawing.preview',{body:'Plate',previous:d})).status==='current','undo restores true old geometry');
 await app.execute('edit.redo');
 pending=await app.execute('drawing.preview',{body:'Plate',previous:d});
 ok(pending.status==='stale'&&pending.proposed.annotations[0].value_mm===updated,'redo makes same revision pending');
 const accepted=await app.execute('drawing.preview',{body:'Plate',previous:d,accept:true});
 ok(accepted.status==='updated'&&accepted.drawing.annotations[0].reference_status==='resolved','explicit acceptance');
 ok(accepted.drawing.annotations[0].second_face_name===faces[1],'second historical ID kept');
 return {test:'RFC860 twin-bore parameter changes, undo/redo, explicit refresh and stable centre annotations',result:'passed'};
})()