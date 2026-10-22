(async()=>{
 const s=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await s.execute('file.new',{name:'Parametric front view width'});
 await s.execute('parameters.add',{name:'L',expression:'65 mm'});
 await s.execute('solid.box',{length:'L',width:42,height:9,body_name:'Plate'});
 const first=(await s.execute('drawing.preview',{body:'Plate'})).drawing;
 await s.execute('parameters.change',{name:'L',expression:'100 mm'});
 const st=await s.execute('drawing.preview',{body:'Plate',previous:first});
 ok(st.status==='stale'&&st.requires_confirmation,'update required after length edit');
 ok(st.drawing.geometry.reference_views.front.bounds.max_xy_mm[0]===65,'old front remains saved');
 ok(st.proposed.geometry.reference_views.front.bounds.max_xy_mm[0]===100,'front follows 3D edit');
 ok(st.proposed.geometry.reference_views.right.bounds.max_xy_mm[0]===42,'side width unchanged');
 const yes=await s.execute('drawing.preview',{body:'Plate',previous:first,accept:true});
 ok(yes.status==='updated'&&yes.drawing.geometry.reference_views.front.bounds.max_xy_mm[0]===100,'accepted new front');
 return {test:'parametric model length refreshes front only on approval',result:'passed'};
})()