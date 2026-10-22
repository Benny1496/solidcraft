(async()=>{
 const s=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await s.execute('file.new',{name:'Negative XY contour projection'});
 await s.execute('solid.box',{length:95,width:55,height:8,corner:[-45,-25,0],body_name:'OffsetPlate'});
 const d=(await s.execute('drawing.preview',{body:'OffsetPlate'})).drawing;
 const paths=d.geometry.top_edge_candidates;
 ok(paths.length===4&&paths.every(p=>p.kind==='straight'),'four translated true top edges');
 const coords=paths.flatMap(p=>p.points_xy_mm);
 ok(coords.some(p=>p[0]===-45)&&coords.some(p=>p[0]===50),'negative and positive X bounds represented');
 ok(coords.some(p=>p[1]===-25)&&coords.some(p=>p[1]===30),'negative and positive Y bounds represented');
 ok(d.source.lower_xy_mm[0]===-45&&d.source.lower_xy_mm[1]===-25,'actual shifted source signature');
 return {test:'XY projected upper paths use world-space negative and positive coordinates',result:'passed'};
})()