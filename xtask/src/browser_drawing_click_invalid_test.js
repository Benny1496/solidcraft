(async () => {
 const app=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await app.execute('file.new',{name:'Reject forged drawing dimension identities'});
 await app.execute('solid.box',{length:110,width:60,height:8,body_name:'Plate'});
 await app.execute('solid.hole',{name:'Drill',position:[18,20,8],diameter:8});
 const first=(await app.execute('drawing.preview',{body:'Plate'})).drawing;
 const face=first.geometry.cylinder_dimensions.find(c=>c.eligible_face_name).eligible_face_name;
 const real=(await app.execute('drawing.annotation.add',{body:'Plate',previous:first,face_name:face,kind:'diameter'})).drawing;
 const copy=o=>JSON.parse(JSON.stringify(o));
 let rejected=0;
 const tryPreview=async(snapshot,label)=>{
  let failed=false;try{await app.execute('drawing.preview',{body:'Plate',previous:snapshot})}catch(e){failed=true}
  ok(failed,'invalid annotation must be rejected: '+label);rejected++;
 };
 let forged=copy(real);
 forged.annotations[0].id='other-unsafe-id';
 await tryPreview(forged,'invalid id');
 forged=copy(real);
 forged.annotations.push({...forged.annotations[0],id:'linked_cyl_999'});
 await tryPreview(forged,'same source and kind twice');
 forged=copy(real);
 forged.annotations[0].kind='angles';
 await tryPreview(forged,'unimplemented angles are not stored as a real dimension');
 forged=copy(real);
 forged.annotations[0].face_name='';
 await tryPreview(forged,'empty face name');
 forged=copy(real);
 forged.annotations=Array.from({length:65},(_,i)=>({
   id:'linked_cyl_'+(i+1),face_name:face+'_'+i,kind:'diameter'
 }));
 await tryPreview(forged,'more than 64 annotations');
 ok((await app.execute('drawing.preview',{body:'Plate',previous:real})).status==='current',
   'all rejected requests leave the genuine saved drawing current');
 return {test:'five corrupt named-face drawing snapshots rejected without altering saved source',rejections:rejected,result:'passed'};
})()
