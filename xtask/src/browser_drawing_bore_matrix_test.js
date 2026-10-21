(async () => {
 const app=window.solidcraft, assert=(v,m)=>{if(!v)throw Error(m)};
 const near=(a,b,m)=>assert(Math.abs(a-b)<0.02,m+': got '+a+', want '+b);
 await app.execute('file.new',{name:'Drawings: cylindrical geometry'});
 await app.execute('parameters.add',{name:'BoreLength',expression:'120 mm'});
 await app.execute('parameters.add',{name:'BoreInset',expression:'12 mm'});
 await app.execute('solid.box',{length:'BoreLength',width:80,height:12,body_name:'BorePlate',name:'Stock'});
 const inputs=[
   {name:'BoreA',position:['BoreInset',12,12],diameter:4,xy:[12,12,4]},
   {name:'BoreB',position:['BoreLength - BoreInset',12,12],diameter:8,xy:[108,12,8]},
   {name:'BoreC',position:[25,65,12],diameter:10,xy:[25,65,10]}
 ];
 for(const b of inputs)await app.execute('solid.hole',{name:b.name,position:b.position,diameter:b.diameter});
 const base=await app.execute('drawing.preview',{body:'BorePlate'});
 assert(base.status==='created','drawing created');
 assert(base.drawing.geometry.cylinders.length===3,'three distinct cylindrical openings');
 assert((base.drawing.svg.match(/<circle /g)||[]).length===3,'SVG circles measured from solid');
 for(const b of inputs){
   assert(base.drawing.geometry.cylinders.some(h=>
    Math.abs(h.center_mm[0]-b.xy[0])<.02&&
    Math.abs(h.center_mm[1]-b.xy[1])<.02&&
    Math.abs(h.diameter_mm-b.xy[2])<.02),'measured diameter and centre for '+b.name);
 }
 let old=base.drawing;
 old.label_positions.overall_x=[63,-20];
 await app.execute('parameters.change',{name:'BoreLength',expression:'150 mm'});
 await app.execute('parameters.change',{name:'BoreInset',expression:'15 mm'});
 const newer=await app.execute('drawing.preview',{body:'BorePlate',previous:old});
 assert(newer.status==='stale'&&newer.requires_confirmation,'multi-parameter change causes stale warning');
 near(newer.proposed.dimensions[0].value_mm,150,'new plate length');
 near(newer.proposed.geometry.cylinders.find(h=>Math.abs(h.diameter_mm-8)<.02).center_mm[0],135,'parametric bore follows plate length');
 near(newer.proposed.geometry.cylinders.find(h=>Math.abs(h.diameter_mm-4)<.02).center_mm[0],15,'parametric bore follows inset');
 assert(newer.proposed.label_positions.overall_x[0]===63,'label layout retained');
 assert(newer.drawing.source.size_mm[0]===120,'original drawing stays unchanged');
 const yes=await app.execute('drawing.preview',{body:'BorePlate',previous:old,accept:true});
 assert(yes.status==='updated','explicit confirmation works');
 assert(yes.drawing.svg.includes('150.000 mm'),'SVG reflects accepted width');
 const again=await app.execute('drawing.preview',{body:'BorePlate',previous:yes.drawing});
 assert(again.status==='current','rechecked drawing is current');
 return {test:'mixed bore sizes and positions, source geometry, 2-step parametric edits',circles:3,result:'passed'};
})()
