(async()=>{
 const sc=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 const sizes=[[20,12,3],[75,40,7],[150,80,10],[210,55,9]];
 const report=[];
 for(const [l,w,h] of sizes){
   await sc.execute('file.new',{name:'Projection matrix '+l});
   await sc.execute('solid.box',{length:l,width:w,height:h,body_name:'Plate'});
   const p=await sc.execute('drawing.projected_edges',{body:'Plate'});
   ok(p.segments.length===4,'four true sides for '+l+' x '+w);
   ok(p.bounds.max_xy_mm[0]===l&&p.bounds.max_xy_mm[1]===w,'bounds '+l);
   const perimeter=p.segments.reduce((t,e)=>t+Math.hypot(e.start_xy_mm[0]-e.end_xy_mm[0],e.start_xy_mm[1]-e.end_xy_mm[1]),0);
   ok(Math.abs(perimeter-2*(l+w))<.005,'perimeter '+l);
   report.push({mm:[l,w,h],segments:p.segments.length,perimeter});
 }
 return {test:'four independently evaluated B-rep rectangle dimensions',report,result:'passed'};
})()