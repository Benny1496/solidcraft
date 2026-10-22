(async () => {
 const app=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await app.execute('file.new',{name:'Selected cylinder dimensions matrix'});
 await app.execute('solid.box',{length:220,width:100,height:10,body_name:'Grid'});
 const coords=[[20,20],[80,20],[140,20],[20,70],[80,70],[140,70]];
 for(let i=0;i<coords.length;i++)
   await app.execute('solid.hole',{name:'GridHole'+i,position:[...coords[i],10],diameter:6+2*(i%3)});
 let drawing=(await app.execute('drawing.preview',{body:'Grid'})).drawing;
 const measured=drawing.geometry.cylinder_dimensions;
 ok(measured.length===6,'six projected cylinders');
 const ids=new Set(),faces=new Set();
 for(const hole of measured){
   const face=hole.eligible_face_name;
   ok(typeof face==='string'&&face.length>0,'every projected hole must have a verified unique 3D name');
   ok(!faces.has(face),'no shared persistent hole face name');
   faces.add(face);
   for(const kind of ['diameter','x_from_left','y_from_bottom']){
     const added=await app.execute('drawing.annotation.add',{
       body:'Grid',previous:drawing,face_name:face,kind,
       position_mm:[hole.center_mm[0]+20,hole.center_mm[1]+20]
     });
     ok(added.status==='annotated','dimension '+kind+' added to '+face);
     ok(!ids.has(added.annotation_id),'unique manual callout id');
     ids.add(added.annotation_id);
     drawing=added.drawing;
   }
 }
 ok(drawing.annotations.length===18,'six times three independent selected dimensions');
 ok(drawing.annotations.every(a=>a.reference_status==='resolved'),'all references fully verified');
 ok(Object.keys(drawing.label_positions).filter(k=>k.startsWith('linked_cyl_')).length===18,'18 independent text positions');
 ok((await app.execute('drawing.preview',{body:'Grid',previous:drawing})).status==='current','18 stored dimensions remain current');
 return {test:'six independently selected 3D cylinders, 18 exact named-face dimensions, no collisions',count:18,result:'passed'};
})()
