(async()=>{
 const app=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await app.execute('file.new',{name:'RFC860 lost reference without neighbouring reassociation'});
 await app.execute('solid.box',{length:120,width:65,height:9,body_name:'Plate'});
 await app.execute('solid.hole',{name:'First',position:[20,20,9],diameter:8});
 await app.execute('solid.hole',{name:'Second',position:[95,45,9],diameter:8});
 let d=(await app.execute('drawing.preview',{body:'Plate'})).drawing;
 const c=d.geometry.cylinder_dimensions;
 ok(c.length===2&&c[0].eligible_face_name&&c[1].eligible_face_name,'two distinct true 3D bore references');
 const first=c.find(x=>Math.abs(x.center_mm[0]-20)<.02).eligible_face_name;
 const second=c.find(x=>Math.abs(x.center_mm[0]-95)<.02).eligible_face_name;
 ok(first!==second,'not the same named face');
 d=(await app.execute('drawing.annotation.add',{body:'Plate',previous:d,face_name:first,kind:'diameter',position_mm:[15,60]})).drawing;
 d=(await app.execute('drawing.annotation.add',{body:'Plate',previous:d,face_name:second,kind:'diameter',position_mm:[95,63]})).drawing;
 d=(await app.execute('drawing.annotation.add',{body:'Plate',previous:d,face_name:first,second_face_name:second,kind:'center_distance',position_mm:[65,65]})).drawing;
 const by=(kind,face)=>d.annotations.find(x=>x.kind===kind&&x.face_name===face);
 const originalPositions=JSON.parse(JSON.stringify(d.label_positions));
 await app.execute('edit.undo'); // last *3D* command removed second bore, annotation commands are noundo
 const next=await app.execute('drawing.preview',{body:'Plate',previous:d});
 ok(next.status==='stale'&&next.requires_confirmation,'deleted 3D bore stales drawing');
 ok(JSON.stringify(next.drawing)===JSON.stringify(d),'defer keeps old drawing intact');
 const x=next.proposed.annotations;
 ok(x.length===3,'all user dimensions retained');
 const keep=x.find(a=>a.kind==='diameter'&&a.face_name===first);
 const lost=x.find(a=>a.kind==='diameter'&&a.face_name===second);
 const dist=x.find(a=>a.kind==='center_distance');
 ok(keep.reference_status==='resolved'&&keep.value_mm===8,'remaining bore stays verified');
 ok(lost.reference_status==='missing_reference'&&lost.value_mm===null&&lost.last_value_mm===8,'deleted bore retains historical diameter only');
 ok(dist.reference_status==='missing_reference'&&dist.value_mm===null,'centre distance not silently remapped');
 for(const a of x)ok(JSON.stringify(next.proposed.label_positions[a.id])===JSON.stringify(originalPositions[a.id]),'manual placement of '+a.id);
 const yes=await app.execute('drawing.preview',{body:'Plate',previous:d,accept:true});
 ok(yes.drawing.annotations.length===3&&yes.drawing.svg.includes('? Ø8.00'),'saved output identifies unresolved dimension');
 ok((await app.execute('drawing.preview',{body:'Plate',previous:yes.drawing})).status==='current','accepted lost refs do not trigger loop');
 return {test:'RFC860 deletion of one of two same-size bores keeps neighbour and marks historic diameter and centre distance unresolved',result:'passed'};
})()