(async()=>{
 const sc=window.solidcraft,ok=(x,m)=>{if(!x)throw Error(m)};
 const near=(v,w,m)=>ok(Number.isFinite(v)&&Math.abs(v-w)<.002,m+': '+v+' != '+w);
 await sc.execute('file.new',{name:'Many history branches do not heal a lost cylinder reference'});
 await sc.execute('solid.box',{length:150,width:80,height:10,body_name:'Plate'});
 await sc.execute('solid.hole',{name:'BoreB',position:[70,50,10],diameter:8});
 await sc.execute('solid.hole',{name:'BoreA',position:[30,20,10],diameter:8});
 let drawing=(await sc.execute('drawing.preview',{body:'Plate'})).drawing;
 const faceAt=(d,x)=>d.geometry.cylinder_dimensions.find(c=>
    Math.abs(c.center_mm[0]-x)<.01)?.eligible_face_name;
 const a=faceAt(drawing,30),b=faceAt(drawing,70);
 ok(a&&b&&a!==b,'two separate baseline faces');
 let added=await sc.execute('drawing.annotation.add',{
   body:'Plate',previous:drawing,face_name:a,kind:'diameter',position_mm:[17,97]});
 drawing=added.drawing;const idA=added.annotation_id;
 added=await sc.execute('drawing.annotation.add',{
   body:'Plate',previous:drawing,face_name:b,kind:'diameter',position_mm:[110,95]});
 drawing=added.drawing;const idB=added.annotation_id;
 added=await sc.execute('drawing.annotation.add',{
   body:'Plate',previous:drawing,face_name:a,second_face_name:b,
   kind:'center_distance',position_mm:[75,105]});
 drawing=added.drawing;const idPair=added.annotation_id;
 const snap=JSON.stringify(drawing);
 const seen=new Set([a,b]);
 const note=(d,id)=>d.annotations.find(x=>x.id===id);
 await sc.execute('edit.undo'); // remove only BoreA

 for(let attempt=1;attempt<=3;attempt++){
   const name='Replacement_'+attempt;
   await sc.execute('solid.hole',{name,position:[30,20,10],diameter:8});
   let next=await sc.execute('drawing.preview',{body:'Plate',previous:JSON.parse(snap)});
   ok(next.status==='stale'&&next.requires_confirmation,'stale revision '+attempt);
   const fresh=faceAt(next.proposed,30);
   ok(fresh&&!seen.has(fresh),'never recycle face on history branch '+attempt);
   seen.add(fresh);
   ok(faceAt(next.proposed,70)===b,'unaffected B retains exact source identity '+attempt);
   ok(note(next.proposed,idA).reference_status==='missing_reference' &&
     note(next.proposed,idA).value_mm===null,'A stays lost '+attempt);
   near(note(next.proposed,idA).last_value_mm,8,'A historical diameter '+attempt);
   ok(note(next.proposed,idPair).reference_status==='missing_reference',
     'pair not silently repaired '+attempt);
   near(note(next.proposed,idPair).last_value_mm,50,'pair last measured distance '+attempt);
   ok(note(next.proposed,idB).reference_status==='resolved',
     'B still resolved '+attempt);
   ok(JSON.stringify(next.proposed.label_positions)===JSON.stringify(drawing.label_positions),
     'all manual labels survive branch '+attempt);
   if(attempt===3){
     const accepted=await sc.execute('drawing.preview',{
       body:'Plate',previous:JSON.parse(snap),accept:true});
     ok(accepted.status==='updated','final replacement accepted as lost');
     ok(note(accepted.drawing,idA).reference_status==='missing_reference',
       'acceptance does not repair absent source');
     ok((await sc.execute('drawing.preview',{
       body:'Plate',previous:accepted.drawing})).status==='current',
       'accepted missing reference is stable');
     await sc.execute('edit.undo'); // remove replacement 3
     await sc.execute('edit.redo'); // original replacement 3 restored
     const repeated=await sc.execute('drawing.preview',{
       body:'Plate',previous:JSON.parse(snap)});
     ok(faceAt(repeated.proposed,30)===fresh,
       'redo recreates the same last replacement ID, not a new one');
   }else{
     await sc.execute('edit.undo'); // remove replacement, leave B only
   }
 }
 return {test:'three successive identical replacement bores allocate distinct face identities and never steal stored A dimensions',
   replacements:3,result:'passed'};
})()
