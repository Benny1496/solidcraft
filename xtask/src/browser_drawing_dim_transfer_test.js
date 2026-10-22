(async () => {
  const app=window.solidcraft;
  const ok=(x,m)=>{if(!x)throw Error(m)};
  const near=(a,b,m)=>ok(typeof a==='number'&&Math.abs(a-b)<.02,m+': '+a+' vs '+b);
  await app.execute('file.new',{name:'3D to 2D dimensions Ø XY'});
  await app.execute('parameters.add',{name:'PlateL',expression:'150 mm'});
  await app.execute('parameters.add',{name:'Offset',expression:'15 mm'});
  await app.execute('parameters.add',{name:'DrillDia',expression:'8 mm'});
  await app.execute('solid.box',{length:'PlateL',width:80,height:10,body_name:'Sheet',name:'Stock'});
  const locs=[['Offset',15],['PlateL - Offset',15],['Offset',65],['PlateL - Offset',65]];
  for(let i=0;i<locs.length;i++)
    await app.execute('solid.hole',{name:'Drill'+i,position:[locs[i][0],locs[i][1],10],diameter:'DrillDia'});
  let a=await app.execute('drawing.preview',{body:'Sheet'});
  ok(a.status==='created','drawing created');
  let dims=a.drawing.geometry.cylinder_dimensions;
  ok(dims.length===4,'four measured diameter/X/Y groups');
  let expected=[[15,15],[135,15],[15,65],[135,65]];
  for(const p of expected) {
    const q=dims.find(d=>Math.abs(d.center_mm[0]-p[0])<.02&&Math.abs(d.center_mm[1]-p[1])<.02);
    ok(q,'measured circle at '+p);
    near(q.diameter_mm,8,'diameter');
    near(q.x_from_left_mm,p[0],'X from left');
    near(q.y_from_bottom_mm,p[1],'Y from bottom');
    ok(q.reference_status==='unverified','no falsely stable callout');
  }
  ok(!Object.keys(a.drawing.label_positions).some(k=>k.startsWith('cylinder_')),
    'no positional hole-label identity is persisted');
  await app.execute('parameters.change',{name:'PlateL',expression:'180 mm'});
  await app.execute('parameters.change',{name:'DrillDia',expression:'10 mm'});
  const stale=await app.execute('drawing.preview',{body:'Sheet',previous:a.drawing});
  ok(stale.status==='stale'&&stale.requires_confirmation,'drawing stale after two model edits');
  dims=stale.proposed.geometry.cylinder_dimensions;
  ok(dims.length===4,'bores retained after edit');
  for(const d of dims)near(d.diameter_mm,10,'new 3D-measured diameter');
  ok(dims.some(d=>Math.abs(d.x_from_left_mm-165)<.02&&Math.abs(d.y_from_bottom_mm-65)<.02),
    'right-hand cylinder updated to 165mm from left');
  near(stale.drawing.geometry.cylinder_dimensions[0].diameter_mm,8,'old drawing unchanged');
  const accepted=await app.execute('drawing.preview',{body:'Sheet',previous:a.drawing,accept:true});
  ok(accepted.status==='updated','confirm dimension transfer');
  near(accepted.drawing.geometry.cylinder_dimensions[0].diameter_mm,10,'new diameter accepted');
  ok((await app.execute('drawing.preview',{body:'Sheet',previous:accepted.drawing})).status==='current',
    'accepted drawing current');
  return {test:'3D measured Ø/X/Y transfer for four parametrically edited bores',count:4,diameters:[8,10],result:'passed'};
})()
