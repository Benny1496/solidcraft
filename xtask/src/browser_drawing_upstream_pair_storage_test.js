(async()=>{
 const app=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 const key='org.solidcraft.experimental_drawing';
 await app.execute('file.new',{name:'RFC860 stored multi-view workshop drawing'});
 await app.execute('parameters.add',{name:'StockL',expression:'110 mm'});
 await app.execute('solid.box',{length:'StockL',width:65,height:8,body_name:'Plate'});
 await app.execute('solid.hole',{name:'Small',position:[20,18,8],diameter:6});
 await app.execute('solid.hole',{name:'Large',position:[80,45,8],diameter:10});
 let d=(await app.execute('drawing.preview',{body:'Plate'})).drawing;
 const face=d.geometry.cylinder_dimensions.find(c=>c.diameter_mm===10).eligible_face_name;
 const added=await app.execute('drawing.annotation.add',{body:'Plate',previous:d,face_name:face,kind:'diameter',position_mm:[94,65]});
 d=added.drawing;const id=added.annotation_id;
 await app.execute('document.metadata.set',{key,value:d});
 await app.execute('parameters.change',{name:'StockL',expression:'145 mm'});
 let saved=(await app.execute('document.metadata',{})).metadata[key];
 let p=await app.execute('drawing.preview',{body:'Plate',previous:saved});
 ok(p.status==='stale'&&p.drawing.geometry.reference_views.front.bounds.max_xy_mm[0]===110,'stored front old until consent');
 ok(p.proposed.geometry.reference_views.front.bounds.max_xy_mm[0]===145,'proposed front reflects new 3D length');
 ok(p.proposed.geometry.reference_views.right.bounds.max_xy_mm[0]===65,'side stays 65 mm');
 ok(p.proposed.label_positions[id][0]===94&&p.proposed.label_positions[id][1]===65,'manual note retained');
 ok(p.proposed.annotations[0].face_name===face&&p.proposed.annotations[0].reference_status==='resolved','never relink the bore');
 const yes=await app.execute('drawing.preview',{body:'Plate',previous:saved,accept:true});
 await app.execute('document.metadata.set',{key,value:yes.drawing});
 saved=(await app.execute('document.metadata',{})).metadata[key];
 ok(saved.geometry.reference_views.front.bounds.max_xy_mm[0]===145,'confirmed front persisted');
 ok((await app.execute('drawing.preview',{body:'Plate',previous:saved})).status==='current','stored accepted drawing is current');
 return {test:'RFC860 stored three-view geometry and linked diameter remain aligned after model edit and confirmation',result:'passed'};
})()