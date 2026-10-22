(async () => {
 const app=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 const near=(a,b,m)=>ok(Math.abs(a-b)<.02,m+' '+a+' vs '+b);
 await app.execute('file.new',{name:'Face-linked dimensions on shifted coordinate plate'});
 await app.execute('solid.box',{length:120,width:70,height:10,corner:[-40,-35,0],body_name:'Translated'});
 await app.execute('solid.hole',{name:'LeftDrill',position:[-23,-18,10],diameter:6});
 await app.execute('solid.hole',{name:'RightDrill',position:[54,21,10],diameter:10});
 let drawing=(await app.execute('drawing.preview',{body:'Translated'})).drawing;
 const holes=drawing.geometry.cylinder_dimensions;
 ok(holes.length===2,'two real cylinders');
 for(const item of holes) {
  const face=item.eligible_face_name;
  ok(!!face,'history-certified face exists');
  for(const kind of ['x_from_left','y_from_bottom']){
   const res=await app.execute('drawing.annotation.add',{body:'Translated',
       previous:drawing,face_name:face,kind});
   drawing=res.drawing;
  }
 }
 const left=holes.find(x=>x.diameter_mm===6),right=holes.find(x=>x.diameter_mm===10);
 const values=face=>drawing.annotations.filter(a=>a.face_name===face).reduce((out,a)=>(out[a.kind]=a.value_mm,out),{});
 near(values(left.eligible_face_name).x_from_left,17,'left hole relative X');
 near(values(left.eligible_face_name).y_from_bottom,17,'left hole relative Y');
 near(values(right.eligible_face_name).x_from_left,94,'right hole relative X');
 near(values(right.eligible_face_name).y_from_bottom,56,'right hole relative Y');
 ok(drawing.annotations.every(x=>x.reference_status==='resolved'),'four named-face dimensions current');
 ok((await app.execute('drawing.preview',{body:'Translated',previous:drawing})).status==='current',
   'no false stale for shifted geometry');
 return {test:'two direct reference-linked holes measured from translated bounding-edge datums',result:'passed'};
})()
