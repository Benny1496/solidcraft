(async()=>{
 const sc=window.solidcraft,ok=(c,m)=>{if(!c)throw Error(m)};
 await sc.execute('file.new',{name:'Projected edge errors and drawing isolation'});
 let refusals=0;
 for(const params of [{}, {body:'absent'}, {body:''}, {body:7}]){
   try{await sc.execute('drawing.projected_edges',params)}catch(e){refusals++}
 }
 ok(refusals===4,'all invalid body references rejected');
 await sc.execute('solid.box',{length:60,width:35,height:6,body_name:'Plate'});
 const first=(await sc.execute('drawing.preview',{body:'Plate'})).drawing;
 const before=JSON.stringify(first);
 const p=await sc.execute('drawing.projected_edges',{body:'Plate'});
 ok(p.segments.length===4,'valid projected lines');
 const again=await sc.execute('drawing.preview',{body:'Plate',previous:first});
 ok(again.status==='current'&&JSON.stringify(again.drawing)===before,'existing drawing and revision remain unchanged');
 return {test:'input rejection and preserved existing 2D revision',refusals,result:'passed'};
})()