(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Workshop plate with two bores'});
 await s.execute('solid.box',{length:130,width:80,height:10,body_name:'Plate'});
 await s.execute('solid.hole',{name:'First',position:[20,25,10],diameter:8});
 await s.execute('solid.hole',{name:'Second',position:[105,55,10],diameter:12});
 const d=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 ok(d.geometry.cylinder_dimensions.length===2,'bore measurement count preserved');
 ok(d.geometry.reference_views.front.projection==='xz_from_negative_y','front has explicit projection');
 ok(d.geometry.reference_views.right.projection==='yz_from_positive_x','right has explicit projection');
 ok(d.geometry.reference_views.front.paths.length>=4&&d.geometry.reference_views.right.paths.length>=4,'physical outside planes retained');
 ok(d.warnings.some(w=>w.includes('NOT complete')),'hidden lines not fabricated');
 return {test:'two-bore part retains 3D measures plus explicitly limited side views',result:'passed'};
})()