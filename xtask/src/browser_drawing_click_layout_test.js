(async () => {
 const app=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 const near=(a,b,m)=>ok(Math.abs(a-b)<.02,m+' '+a+'/'+b);
 await app.execute('file.new',{name:'Manually positioned selected face callouts'});
 await app.execute('parameters.add',{name:'PlateW',expression:'100 mm'});
 await app.execute('parameters.add',{name:'BoreD',expression:'6 mm'});
 await app.execute('solid.box',{length:'PlateW',width:60,height:10,body_name:'Plate'});
 await app.execute('solid.hole',{name:'StableHole',position:[25,25,10],diameter:'BoreD'});
 let drawing=(await app.execute('drawing.preview',{body:'Plate'})).drawing;
 const face=drawing.geometry.cylinder_dimensions.find(c=>c.eligible_face_name).eligible_face_name;
 for(const [i,kind] of ['diameter','x_from_left','y_from_bottom'].entries()){
   const r=await app.execute('drawing.annotation.add',{body:'Plate',previous:drawing,
     face_name:face,kind,position_mm:[42+i*11,47+i*7]});
   drawing=r.drawing;
 }
 const positions=JSON.parse(JSON.stringify(drawing.label_positions));
 const key='org.solidcraft.experimental_drawing';
 await app.execute('document.metadata.set',{key,value:drawing});
 await app.execute('parameters.change',{name:'BoreD',expression:'12 mm'});
 await app.execute('parameters.change',{name:'PlateW',expression:'130 mm'});
 const pending=await app.execute('drawing.preview',{body:'Plate',previous:drawing});
 ok(pending.status==='stale'&&pending.requires_confirmation,'changes require confirmation');
 near(pending.drawing.annotations.find(x=>x.kind==='diameter').value_mm,6,'saved Ø remains unchanged');
 near(pending.proposed.annotations.find(x=>x.kind==='diameter').value_mm,12,'proposed Ø changes from actual 3D');
 for(const [id,xy] of Object.entries(positions)) {
   near(pending.proposed.label_positions[id][0],xy[0],id+' X preserved');
   near(pending.proposed.label_positions[id][1],xy[1],id+' Y preserved');
 }
 const updated=await app.execute('drawing.preview',{body:'Plate',previous:drawing,accept:true});
 ok(updated.status==='updated','explicit acceptance');
 ok(updated.drawing.svg.includes('Ø12.00')&&updated.drawing.svg.includes('X 25.00')&&updated.drawing.svg.includes('Y 25.00'),'all three selected leader labels persist');
 ok(updated.drawing.annotations.every(x=>x.face_name===face&&x.reference_status==='resolved'),'all exact identities preserved');
 return {test:'manual Ø X Y dimension positions and face provenance persist over two model edits',result:'passed'};
})()
