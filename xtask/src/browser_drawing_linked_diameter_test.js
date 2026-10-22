(async () => {
 const app=window.solidcraft,check=(c,m)=>{if(!c)throw Error(m)};
 const near=(a,b,m)=>check(Math.abs(a-b)<.02,m+' '+a+' vs '+b);
 await app.execute('file.new',{name:'Associative cylinder Ø dimension'});
 await app.execute('parameters.add',{name:'Dia',expression:'8 mm'});
 await app.execute('parameters.add',{name:'Inset',expression:'15 mm'});
 await app.execute('solid.box',{length:130,width:70,height:10,body_name:'Plate',name:'Stock'});
 await app.execute('solid.hole',{name:'NamedDrill',position:['Inset',20,10],diameter:'Dia'});
 const before=await app.execute('drawing.preview',{body:'Plate'});
 const candidate=before.drawing.geometry.cylinder_dimensions.find(d=>d.eligible_face_name);
 check(candidate,'a typical through-hole must have an eligible history-verified face');
 const face=candidate.eligible_face_name;
 const result=await app.execute('drawing.annotation.add',{body:'Plate',previous:before.drawing,
    face_name:face,kind:'diameter',position_mm:[45,42]});
 check(result.status==='annotated','create verified Ø annotation');
 let old=result.drawing;
 check(old.annotations.length===1,'one manual annotation');
 const callout=old.annotations[0];
 check(callout.reference_status==='resolved'&&callout.face_name===face,'verified face stored');
 near(callout.value_mm,8,'initial Ø');
 check(old.label_positions[callout.id][0]===45,'manual label placement stored');
 check(old.svg.includes('Ø8.00'),'placed diameter leader is rendered into the SVG drawing');
 const key='org.solidcraft.experimental_drawing';
 await app.execute('document.metadata.set',{key,value:old});
 await app.execute('parameters.change',{name:'Dia',expression:'12 mm'});
 await app.execute('parameters.change',{name:'Inset',expression:'19 mm'});
 const saved=(await app.execute('document.metadata',{})).metadata[key];
 const pending=await app.execute('drawing.preview',{body:'Plate',previous:saved});
 check(pending.status==='stale'&&pending.requires_confirmation,'3D changes require user confirmation');
 near(pending.drawing.annotations[0].value_mm,8,'original dimension remains');
 check(pending.proposed.annotations[0].face_name===face,'stable original face identity');
 check(pending.proposed.annotations[0].reference_status==='resolved','face remains resolvable after parametric edit');
 near(pending.proposed.annotations[0].value_mm,12,'new Ø measured from actual 3D');
 check(pending.proposed.label_positions[callout.id][0]===45,'manual position survives');
 const updated=await app.execute('drawing.preview',{body:'Plate',previous:saved,accept:true});
 check(updated.status==='updated','confirmed change');
 near(updated.drawing.annotations[0].value_mm,12,'accepted Ø');
 check(updated.drawing.svg.includes('Ø12.00'),'SVG reflects the accepted 3D diameter change');
 await app.execute('document.metadata.set',{key,value:updated.drawing});
 check((await app.execute('drawing.preview',{body:'Plate',previous:updated.drawing})).status==='current',
   'updated associated drawing is current');
 return {test:'verified Ø dimension follows same 3D face on diameter and XY parameter changes',
    face,original:8,updated:12,result:'passed'};
})()
