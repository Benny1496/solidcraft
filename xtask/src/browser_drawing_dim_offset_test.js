(async () => {
 const a=window.solidcraft,ok=(x,m)=>{if(!x)throw Error(m)};
 const near=(x,y,m)=>ok(Math.abs(x-y)<.02,m+' '+x+'/'+y);
 await a.execute('file.new',{name:'XY dimensions translated plate'});
 await a.execute('solid.box',{length:100,width:60,height:9,corner:[-40,-25,0],body_name:'Translated',name:'Base'});
 await a.execute('solid.hole',{position:[-27,-14,9],diameter:6,name:'First'});
 await a.execute('solid.hole',{position:[37,21,9],diameter:10,name:'Second'});
 const start=await a.execute('drawing.preview',{body:'Translated'});
 ok(start.status==='created','created translated drawing');
 const d=start.drawing;
 near(d.geometry.outline.min_xy_mm[0],-40,'min x');
 near(d.geometry.outline.min_xy_mm[1],-25,'min y');
 near(d.dimensions[0].value_mm,100,'length');
 near(d.dimensions[1].value_mm,60,'width');
 const c=d.geometry.cylinder_dimensions;
 ok(c.length===2,'two measured cylinders');
 const p=c.find(v=>Math.abs(v.diameter_mm-6)<.02);
 const q=c.find(v=>Math.abs(v.diameter_mm-10)<.02);
 ok(p&&q,'both diameters present');
 near(p.center_mm[0],-27,'first actual world X');
 near(p.x_from_left_mm,13,'first offset X from left');
 near(p.y_from_bottom_mm,11,'first offset Y from bottom');
 near(q.x_from_left_mm,77,'second X');
 near(q.y_from_bottom_mm,46,'second Y');
 ok(d.accuracy==='bbox_envelope_and_measured_cylinders_only',
   'preview declares geometry accuracy limit');
 return {test:'XY coordinates relative to shifted physical edges, mixed diameters',result:'passed'};
})()
