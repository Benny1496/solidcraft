(async () => {
 const sc=window.solidcraft,ok=(v,m)=>{if(!v)throw Error(m)};
 await sc.execute('file.new',{name:'Measured drawing matrix many bores'});
 await sc.execute('solid.box',{length:160,width:100,height:10,body_name:'Matrix'});
 let count=0;
 for(let row=0;row<3;row++)for(let col=0;col<4;col++){
   await sc.execute('solid.hole',{name:'B'+count,position:[20+col*40,20+row*30,10],diameter:4+2*row});
   count++;
 }
 const result=await sc.execute('drawing.preview',{body:'Matrix'});
 ok(result.status==='created','matrix preview');
 const dims=result.drawing.geometry.cylinder_dimensions;
 ok(dims.length===count,'all twelve hole circles dimensioned');
 ok(result.drawing.geometry.cylinders.length===count,'circle projection count consistent');
 for(let row=0;row<3;row++){
   const diam=4+row*2;
   const rowItems=dims.filter(d=>Math.abs(d.diameter_mm-diam)<.02);
   ok(rowItems.length===4,'four measured circles diameter '+diam);
   ok(rowItems.every(d=>Math.abs(d.y_from_bottom_mm-(20+row*30))<.02),'all Y offsets in row '+row);
 }
 ok((await sc.execute('drawing.preview',{body:'Matrix',previous:result.drawing})).status==='current',
   '12 cylinder dimensions do not cause false stale');
 return {test:'12 real 3D cylinders with three different diameters, XY/dimension count',count,result:'passed'};
})()
