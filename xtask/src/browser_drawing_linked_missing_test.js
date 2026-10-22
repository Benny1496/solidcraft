(async () => {
 const app=window.solidcraft,check=(x,m)=>{if(!x)throw Error(m)};
 await app.execute('file.new',{name:'No silent hole dimension reassociation'});
 await app.execute('solid.box',{length:120,width:60,height:8,body_name:'Plate'});
 await app.execute('solid.hole',{name:'Drill1',position:[20,20,8],diameter:8});
 await app.execute('solid.hole',{name:'Drill2',position:[80,35,8],diameter:8});
 const first=(await app.execute('drawing.preview',{body:'Plate'})).drawing;
 const selected=first.geometry.cylinder_dimensions.find(d=>
    d.eligible_face_name&&Math.abs(d.center_mm[0]-80)<.02);
 check(selected,'second 3D hole must have a verified face');
 const linked=(await app.execute('drawing.annotation.add',{
    body:'Plate',previous:first,face_name:selected.eligible_face_name,kind:'diameter'
 })).drawing;
 await app.execute('edit.undo');
 const pending=await app.execute('drawing.preview',{body:'Plate',previous:linked});
 check(pending.status==='stale'&&pending.requires_confirmation,'hole removal flagged');
 check(pending.drawing.annotations[0].reference_status==='resolved','saved copy unchanged');
 check(pending.proposed.annotations.length===1,'unresolved annotation not silently removed');
 check(pending.proposed.annotations[0].reference_status==='missing_reference','lost face detected');
 check(pending.proposed.annotations[0].value_mm===null,'lost face has no falsely current dimension');
 check(pending.proposed.annotations[0].last_value_mm===8,'old historic diameter retained');
 const accepted=await app.execute('drawing.preview',{body:'Plate',previous:linked,accept:true});
 check(accepted.status==='updated','user can accept removal without auto reattachment');
 check(accepted.drawing.annotations[0].face_name===selected.eligible_face_name,'face identity retained for inspection');
 check(accepted.drawing.annotations[0].reference_status==='missing_reference','still unresolved');
 check((await app.execute('drawing.preview',{body:'Plate',previous:accepted.drawing})).status==='current',
    'accepted missing annotation does not endlessly retrigger a warning');
 return {test:'deleted hole leaves old annotation unresolved, not matched to another same-sized circle',result:'passed'};
})()
