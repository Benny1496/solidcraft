(async () => {
  const sc=window.solidcraft;
  const ok=(v,m)=>{if(!v)throw Error(m)};
  const near=(x,y,m)=>ok(Number.isFinite(x)&&Math.abs(x-y)<.002,m+': '+x+' != '+y);
  const note=(d,id)=>d.annotations.find(a=>a.id===id);
  // JSON string is an independent mock handoff payload for future separate
  // drawing-file APIs. This test does NOT pretend to save a .solidcraft-drawing.
  await sc.execute('file.new',{name:'Deferred closed drawing handoff contract'});
  await sc.execute('parameters.add',{name:'StockWidth',expression:'80 mm'});
  await sc.execute('solid.box',{length:100,width:'StockWidth',height:20,body_name:'Plate'});
  await sc.execute('solid.hole',{name:'BoreA',position:[20,20,20],diameter:10});
  await sc.execute('solid.hole',{name:'BoreB',position:[60,50,20],diameter:10});
  let d=(await sc.execute('drawing.preview',{body:'Plate'})).drawing;
  const a=d.geometry.cylinder_dimensions.find(x=>Math.abs(x.center_mm[0]-20)<.01);
  const b=d.geometry.cylinder_dimensions.find(x=>Math.abs(x.center_mm[0]-60)<.01);
  ok(a&&b&&a.eligible_face_name&&b.eligible_face_name&&
    a.eligible_face_name!==b.eligible_face_name,'independent certified A/B faces');
  let x=await sc.execute('drawing.annotation.add',{
    body:'Plate',previous:d,face_name:a.eligible_face_name,
    kind:'diameter',position_mm:[18,112]});
  d=x.drawing;const aId=x.annotation_id;
  x=await sc.execute('drawing.annotation.add',{
    body:'Plate',previous:d,face_name:b.eligible_face_name,
    kind:'diameter',position_mm:[98,112]});
  d=x.drawing;const bId=x.annotation_id;
  x=await sc.execute('drawing.annotation.add',{
    body:'Plate',previous:d,face_name:a.eligible_face_name,
    second_face_name:b.eligible_face_name,kind:'center_distance',
    position_mm:[53,120]});
  d=x.drawing;const pairId=x.annotation_id;
  const externalJSON=JSON.stringify(d); // "closed" drawing: not in document.metadata
  ok(JSON.stringify(JSON.parse(externalJSON))===externalJSON,'JSON handoff deterministic in memory');
  ok((await sc.execute('document.metadata',{})).metadata['org.solidcraft.experimental_drawing']===undefined,
    'drawing is not stored in model metadata for this handoff test');

  await sc.execute('parameters.change',{name:'StockWidth',expression:'100 mm'});
  const reopen=JSON.parse(externalJSON);
  let pending=await sc.execute('drawing.preview',{body:'Plate',previous:reopen});
  ok(pending.status==='stale'&&pending.requires_confirmation,'open handed-off payload as stale');
  near(pending.drawing.dimensions[1].value_mm,80,'closed snapshot old width');
  near(pending.proposed.dimensions[1].value_mm,100,'live model width');
  near(note(pending.proposed,aId).value_mm,10,'A diameter unchanged by width');
  near(note(pending.proposed,bId).value_mm,10,'B diameter unchanged by width');
  near(note(pending.proposed,pairId).value_mm,50,'centre distance unchanged by width');
  ok(JSON.stringify(pending.proposed.label_positions)===JSON.stringify(d.label_positions),
    'all manually positioned labels retained');

  await sc.execute('edit.undo');
  ok((await sc.execute('drawing.preview',{body:'Plate',previous:JSON.parse(externalJSON)})).status==='current',
    'Undo model width restores old external drawing revision');
  await sc.execute('edit.redo');
  pending=await sc.execute('drawing.preview',{body:'Plate',previous:JSON.parse(externalJSON)});
  ok(pending.status==='stale','Redo correctly invalidates unopened payload again');

  const accepted=await sc.execute('drawing.preview',{
    body:'Plate',previous:JSON.parse(externalJSON),accept:true});
  ok(accepted.status==='updated','deferred update confirmed');
  const acceptedJSON=JSON.stringify(accepted.drawing);
  ok((await sc.execute('drawing.preview',{body:'Plate',previous:JSON.parse(acceptedJSON)})).status==='current',
    'accepted payload roundtrips independently');

  await sc.execute('timeline.delete',{feature:'BoreA'});
  const removed=await sc.execute('drawing.preview',{body:'Plate',previous:JSON.parse(acceptedJSON)});
  ok(removed.status==='stale','BoreA deletion stales external drawing');
  ok(removed.proposed.geometry.cylinder_dimensions.length===1,'only BoreB remains in 3D');
  ok(note(removed.proposed,aId).reference_status==='missing_reference',
    'historical A reference lost rather than reassociated');
  ok(note(removed.proposed,pairId).reference_status==='missing_reference',
    'historical pair lost, not reassociated');
  near(note(removed.proposed,aId).last_value_mm,10,'historical diameter');
  near(note(removed.proposed,pairId).last_value_mm,50,'historical centre distance');
  ok(note(removed.proposed,aId).value_mm===null&&
    note(removed.proposed,pairId).value_mm===null,'lost values never presented as current');
  ok(note(removed.proposed,bId).reference_status==='resolved','B remains resolved');
  ok(JSON.stringify(removed.proposed.label_positions)===JSON.stringify(d.label_positions),
    'no label movement after deletion');
  const lost=await sc.execute('drawing.preview',{
    body:'Plate',previous:JSON.parse(acceptedJSON),accept:true});
  ok(lost.status==='updated' &&
    (await sc.execute('drawing.preview',{body:'Plate',previous:lost.drawing})).status==='current',
    'user may keep lost references with their last values');
  await sc.execute('edit.undo');
  const restored=await sc.execute('drawing.preview',{body:'Plate',previous:lost.drawing});
  ok(restored.status==='stale','Undo after acceptance reflects restored model');
  near(note(restored.proposed,aId).value_mm,10,'original A geometry restored');
  ok(note(restored.proposed,aId).reference_status==='resolved','original A identity restored');
  return {test:'externally serialized (not yet on-disk) drawing snapshots support reopen-style stale/accept/lost/undo',
    result:'passed'};
})()
