(async () => {
  const sc=window.solidcraft;
  const ok=(x,m)=>{if(!x)throw Error(m)};
  const near=(x,y,m)=>ok(Number.isFinite(x)&&Math.abs(x-y)<.002,m+': '+x+' vs '+y);
  const external=s=>JSON.parse(JSON.stringify(s)); // independent drawing payload, not model metadata
  const annotation=(d,id)=>d.annotations.find(a=>a.id===id);

  await sc.execute('file.new',{name:'Two independent external drawing snapshots'});
  await sc.execute('parameters.add',{name:'StockLength',expression:'120 mm'});
  await sc.execute('parameters.add',{name:'SharedDiameter',expression:'8 mm'});
  await sc.execute('solid.box',{length:'StockLength',width:80,height:10,body_name:'Plate'});
  await sc.execute('solid.hole',{name:'BoreA',position:[25,20,10],diameter:'SharedDiameter'});
  await sc.execute('solid.hole',{name:'BoreB',position:[85,55,10],diameter:'SharedDiameter'});
  const base=(await sc.execute('drawing.preview',{body:'Plate'})).drawing;
  const bores=base.geometry.cylinder_dimensions;
  const faceA=bores.find(c=>Math.abs(c.center_mm[0]-25)<.01)?.eligible_face_name;
  const faceB=bores.find(c=>Math.abs(c.center_mm[0]-85)<.01)?.eligible_face_name;
  ok(faceA&&faceB&&faceA!==faceB,'two certified distinct bores');

  let x=await sc.execute('drawing.annotation.add',{
    body:'Plate',previous:base,face_name:faceA,kind:'diameter',position_mm:[20,96]});
  const idA=x.annotation_id;
  const serialA=JSON.stringify(x.drawing);
  let y=await sc.execute('drawing.annotation.add',{
    body:'Plate',previous:base,face_name:faceB,kind:'diameter',position_mm:[101,97]});
  const idB=y.annotation_id;
  y=await sc.execute('drawing.annotation.add',{
    body:'Plate',previous:y.drawing,face_name:faceA,second_face_name:faceB,
    kind:'center_distance',position_mm:[67,104]});
  const idPair=y.annotation_id;
  const serialB=JSON.stringify(y.drawing);
  ok(serialA!==serialB,'each drawing has independent serialized content');
  ok(!(('org.solidcraft.experimental_drawing') in
    (await sc.execute('document.metadata',{})).metadata),
    'no experimental drawing snapshot inserted into model metadata');

  await sc.execute('parameters.change',{name:'StockLength',expression:'145 mm'});
  const pendingA=await sc.execute('drawing.preview',{body:'Plate',previous:external(JSON.parse(serialA))});
  const pendingB=await sc.execute('drawing.preview',{body:'Plate',previous:external(JSON.parse(serialB))});
  for(const [name,p,old] of [['A',pendingA,serialA],['B',pendingB,serialB]]){
    ok(p.status==='stale'&&p.requires_confirmation,name+': deferred on model edit');
    near(p.drawing.dimensions[0].value_mm,120,name+': saved dimensions stay old');
    near(p.proposed.dimensions[0].value_mm,145,name+': proposed length');
    ok(JSON.stringify(p.drawing)===old,name+': no implicit snapshot update');
    ok(p.proposed.annotations.every(a=>a.reference_status==='resolved'),
      name+': links survive size edit');
  }
  ok(JSON.stringify(pendingA.proposed.label_positions[idA])===JSON.stringify([20,96]),
    'drawing A label position kept');
  ok(JSON.stringify(pendingB.proposed.label_positions[idB])===JSON.stringify([101,97]),
    'drawing B diameter label kept');
  ok(JSON.stringify(pendingB.proposed.label_positions[idPair])===JSON.stringify([67,104]),
    'drawing B distance label kept');

  const acceptedA=await sc.execute('drawing.preview',{
    body:'Plate',previous:JSON.parse(serialA),accept:true});
  ok(acceptedA.status==='updated','drawing A accepted');
  near(acceptedA.drawing.dimensions[0].value_mm,145,'A updated length');
  ok((await sc.execute('drawing.preview',{body:'Plate',previous:acceptedA.drawing})).status==='current',
    'drawing A is current');
  ok((await sc.execute('drawing.preview',{body:'Plate',previous:JSON.parse(serialB)})).status==='stale',
    'drawing B stays stale independently of A acceptance');
  ok(JSON.stringify(JSON.parse(serialB))===serialB,'drawing B payload remains bytewise unchanged');
  await sc.execute('parameters.change',{name:'SharedDiameter',expression:'10 mm'});

  const afterA=await sc.execute('drawing.preview',{body:'Plate',previous:acceptedA.drawing});
  const afterB=await sc.execute('drawing.preview',{body:'Plate',previous:JSON.parse(serialB)});
  ok(afterA.status==='stale'&&afterB.status==='stale','both detect later diameter edit');
  near(annotation(afterA.proposed,idA).value_mm,10,'A refreshed diameter');
  near(annotation(afterB.proposed,idB).value_mm,10,'B refreshed diameter');
  near(annotation(afterB.proposed,idPair).value_mm,Math.hypot(60,35),
    'B preserves real centre-to-centre distance');
  near(annotation(afterB.drawing,idB).value_mm,8,'B saved diameter still 8');
  const yesB=await sc.execute('drawing.preview',{
    body:'Plate',previous:JSON.parse(serialB),accept:true});
  ok(yesB.status==='updated'&&
    (await sc.execute('drawing.preview',{body:'Plate',previous:yesB.drawing})).status==='current',
    'independent B acceptance now current');
  near(annotation(yesB.drawing,idB).value_mm,10,'accepted B measured diameter');
  ok(JSON.stringify(yesB.drawing.label_positions[idPair])===JSON.stringify([67,104]),
    'B centre-distance label survives two unsaved model edits');
  ok((await sc.execute('drawing.preview',{body:'Plate',previous:acceptedA.drawing})).status==='stale',
    'accepting B did not mutate A');
  return {test:'two independently serialized drawing snapshots remain isolated on shared 3D changes',
    stages:['length','independent update','diameter','independent update'],result:'passed'};
})()
