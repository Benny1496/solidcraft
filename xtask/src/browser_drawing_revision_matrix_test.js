(async () => {
  const sc=window.solidcraft;
  const ok=(c,m)=>{if(!c)throw Error(m)};
  const eq=(a,b,m)=>ok(a===b,m+' got '+JSON.stringify(a)+' expected '+JSON.stringify(b));
  await sc.execute('file.new',{name:'2D revision matrix'});
  await sc.execute('parameters.add',{name:'RevLength',expression:'150 mm'});
  await sc.execute('parameters.add',{name:'RevWidth',expression:'80 mm'});
  await sc.execute('solid.box',{length:'RevLength',width:'RevWidth',height:10,
      body_name:'RevisionPlate',name:'BaseStock'});
  let first=await sc.execute('drawing.preview',{body:'RevisionPlate'});
  eq(first.status,'created','created');
  const old=first.drawing;
  old.label_positions.overall_x=[72,-24];
  old.label_positions.overall_y=[-21,40];
  let statuses=[];
  const verify=async(l,w,expected,stage,previous=old)=>{
    let res=await sc.execute('drawing.preview',{body:'RevisionPlate',previous});
    eq(res.status,expected,stage);
    eq(res.requires_confirmation,expected==='stale',stage+' requires confirmation');
    eq(res.drawing.dimensions[0].value_mm,previous.dimensions[0].value_mm,stage+' retained stored length');
    if(expected==='stale'){
      eq(res.proposed.dimensions[0].value_mm,l,stage+' proposed length');
      eq(res.proposed.dimensions[1].value_mm,w,stage+' proposed width');
      eq(res.proposed.label_positions.overall_x[0],72,stage+' label x');
      eq(res.proposed.label_positions.overall_y[0],-21,stage+' label y');
    }
    statuses.push({stage,status:res.status,current_mm:[l,w]});
    return res;
  };
  await verify(150,80,'current','initial');
  await sc.execute('parameters.change',{name:'RevLength',expression:'175 mm'});
  await verify(175,80,'stale','length extended');
  await sc.execute('edit.undo');
  await verify(150,80,'current','undo length');
  await sc.execute('edit.redo');
  await verify(175,80,'stale','redo length');
  const accepted=await sc.execute('drawing.preview',{body:'RevisionPlate',previous:old,accept:true});
  eq(accepted.status,'updated','accept first update');
  eq(accepted.drawing.dimensions[0].value_mm,175,'accepted length');
  await verify(175,80,'current','accepted is current',accepted.drawing);
  await sc.execute('parameters.change',{name:'RevWidth',expression:'95 mm'});
  const st=await sc.execute('drawing.preview',{body:'RevisionPlate',previous:accepted.drawing});
  eq(st.status,'stale','width changes alone cause stale');
  eq(st.proposed.dimensions[1].value_mm,95,'new width');
  await sc.execute('edit.undo');
  await verify(175,80,'current','undo width',accepted.drawing);
  await sc.execute('edit.redo');
  const upd=await sc.execute('drawing.preview',{body:'RevisionPlate',previous:accepted.drawing,accept:true});
  eq(upd.status,'updated','accept width');
  eq(upd.drawing.dimensions[1].value_mm,95,'accepted width');
  eq(upd.drawing.label_positions.overall_y[0],-21,'labels survive second update');
  statuses.push({stage:'width accepted',status:upd.status,current_mm:[175,95]});
  return {test:'revisions, multi-axis edits, undo, redo, reject/accept protocol',checks:statuses,result:'passed'};
})()
