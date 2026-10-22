(async () => {
 const sc=window.solidcraft,ok=(x,m)=>{if(!x)throw Error(m)};
 await sc.execute('file.new',{name:'Three manual linked drawing dimensions'});
 await sc.execute('solid.box',{length:130,width:70,height:8,body_name:'Panel'});
 await sc.execute('solid.hole',{name:'Hole1',position:[18,24,8],diameter:6});
 let v=(await sc.execute('drawing.preview',{body:'Panel'})).drawing;
 const c=v.geometry.cylinder_dimensions.find(x=>x.eligible_face_name);
 ok(c,'verified face for hole required');
 const f=c.eligible_face_name;
 const ids=[],values={diameter:6,x_from_left:18,y_from_bottom:24};
 for(const kind of ['diameter','x_from_left','y_from_bottom']){
   const r=await sc.execute('drawing.annotation.add',{body:'Panel',previous:v,
     face_name:f,kind,position_mm:[40+ids.length*10,48]});
   ok(r.status==='annotated','add '+kind);
   ids.push(r.annotation_id);v=r.drawing;
   const added=v.annotations.find(a=>a.id===r.annotation_id);
   ok(added&&added.reference_status==='resolved'&&
      Math.abs(added.value_mm-values[kind])<.02,'correct 3D value of '+kind);
 }
 ok(new Set(ids).size===3,'each dimension must have a unique id');
 ok(Object.keys(v.label_positions).filter(x=>x.startsWith('linked_cyl_')).length===3,
   'all manual placements persisted');
 const same=await sc.execute('drawing.preview',{body:'Panel',previous:v});
 ok(same.status==='current'&&same.drawing.annotations.length===3,'same model retains all annotations');
 let denied=false;try{await sc.execute('drawing.annotation.add',
  {body:'Panel',previous:v,face_name:f,kind:'diameter'})}catch(e){denied=true}
 ok(denied,'duplicate same diameter rejected');
 return {test:'user-selected Ø / X / Y dimensions share one verified face, separate placements',count:3,result:'passed'};
})()
