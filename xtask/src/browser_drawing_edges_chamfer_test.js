(async()=>{
 const sc=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await sc.execute('file.new',{name:'Chamfered plate 2D segment candidates'});
 await sc.execute('solid.box',{length:40,width:30,height:12,body_name:'Plate'});
 const stock=await sc.execute('model.edges',{body:'Plate'});
 const vertical=stock.edges.find(e=>Math.abs(e.length-12)<.01 &&
   Math.abs(e.mid[0])<.01 && Math.abs(e.mid[1])<.01);
 ok(vertical,'real vertical corner edge exists');
 await sc.execute('solid.chamfer',{edges:[vertical.mid],distance:2.5,name:'CornerChamfer'});
 const out=await sc.execute('drawing.projected_edges',{body:'Plate'});
 const diag=out.segments.filter(e=>{
   const a=e.start_xy_mm,b=e.end_xy_mm;
   return Math.abs(a[0]-b[0])>.01&&Math.abs(a[1]-b[1])>.01;
 });
 ok(diag.length>=1,'beveled corner projects as a real diagonal 2D line');
 ok(out.segments.length>=5,'nonrectangular silhouette has extra straight edges');
 ok(out.accuracy.includes('unclassified'),'not falsely certified as complete silhouette');
 const prev=await sc.execute('drawing.preview',{body:'Plate'});
 ok(prev.drawing.geometry.outline.type==='rectangle_envelope','existing drawing contract unchanged');
 return {test:'chamfer diagonal is extracted; legacy view protected',lines:out.segments.length,result:'passed'};
})()