(async () => {
 const app=window.solidcraft,check=(c,m)=>{if(!c)throw Error(m)};
 const near=(a,b,m)=>check(Number.isFinite(a)&&Math.abs(a-b)<.002,m);
 await app.execute('file.new',{name:'Two referenced cylinder centres'});
 await app.execute('parameters.add',{name:'PairX',expression:'50 mm'});
 await app.execute('solid.box',{length:120,width:80,height:8,body_name:'Plate'});
 await app.execute('solid.hole',{name:'First',position:[20,20,8],diameter:6});
 await app.execute('solid.hole',{name:'Second',position:['PairX',60,8],diameter:6});
 const first=(await app.execute('drawing.preview',{body:'Plate'})).drawing;
 const [a,b]=first.geometry.cylinder_dimensions.map(c=>c.eligible_face_name);
 check(a&&b&&a!==b,'two verified distinct faces');
 const add=async(previous,face_name,second_face_name)=>app.execute('drawing.annotation.add',{
   body:'Plate',previous,face_name,second_face_name,kind:'center_distance',position_mm:[70,70]});
 const linked=await add(first,a,b),old=linked.drawing,id=linked.annotation_id;
 near(old.annotations[0].value_mm,50,'3-4-5 diagonal centre distance');
 check(old.svg.includes('M–M 50.00'),'distance rendered in SVG');
 for(const [previous,x,y] of [[old,b,a],[first,a,a],[first,a,'invalid-face'],[first,a,null]]){
   let denied=false;try{await add(previous,x,y)}catch(e){denied=true}
   check(denied,'reverse duplicate / same / unknown / absent reference refused');
 }
 await app.execute('parameters.change',{name:'PairX',expression:'80 mm'});
 const pending=await app.execute('drawing.preview',{body:'Plate',previous:old});
 check(pending.status==='stale'&&pending.requires_confirmation,'model edit needs confirmation');
 check(JSON.stringify(pending.drawing)===JSON.stringify(old),'old drawing untouched');
 near(pending.proposed.annotations[0].value_mm,Math.hypot(60,40),'new measured distance');
 let denied=false;try{await add(old,a,b)}catch(e){denied=true}
 check(denied,'no dimension insertion on stale drawing');
 const accepted=await app.execute('drawing.preview',{body:'Plate',previous:old,accept:true});
 check(accepted.status==='updated','explicit acceptance');
 check(accepted.drawing.label_positions[id][0]===70&&accepted.drawing.label_positions[id][1]===70,
   'manual label retained');
 check(accepted.drawing.annotations[0].face_name===a&&accepted.drawing.annotations[0].second_face_name===b,
   'both identities retained');
 await app.execute('edit.undo');
 const undo=await app.execute('drawing.preview',{body:'Plate',previous:accepted.drawing});
 check(undo.status==='stale','undo also requires confirmation');
 near(undo.proposed.annotations[0].value_mm,50,'undo restores measured distance');
 await app.execute('edit.redo');
 check((await app.execute('drawing.preview',{body:'Plate',previous:accepted.drawing})).status==='current',
   'redo to accepted geometry is current');
 await app.execute('edit.undo'); // undo parameter change
 await app.execute('edit.undo'); // remove second bore
 const removed=await app.execute('drawing.preview',{body:'Plate',previous:accepted.drawing});
 check(removed.status==='stale','removed second reference needs confirmation');
 check(removed.proposed.annotations[0].reference_status==='missing_reference','no neighbour reassignment');
 check(removed.proposed.annotations[0].value_mm===null,'no authoritative value after removal');
 const missing=await app.execute('drawing.preview',{body:'Plate',previous:accepted.drawing,accept:true});
 check(missing.drawing.svg.includes('? M–M'),'unresolved historical dimension is marked');
 check((await app.execute('drawing.preview',{body:'Plate',previous:missing.drawing})).status==='current',
   'accepted missing reference does not reopen warning forever');
 return {test:'two-centre distance, confirmation, layout, undo/redo, lost reference and rejections',result:'passed'};
})()
