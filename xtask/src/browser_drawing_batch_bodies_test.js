(async () => {
 const sc=window.solidcraft,ok=(x,m)=>{if(!x)throw Error(m)};
 const scenarios=[
  {x:20,y:10,z:2,holes:0},
  {x:80,y:40,z:6,holes:2},
  {x:150,y:80,z:10,holes:4},
  {x:200,y:100,z:14,holes:6}
 ];
 let checked=0;
 for(let i=0;i<scenarios.length;i++){
  const s=scenarios[i], name='Plate_'+i;
  await sc.execute('file.new',{name:'Batch sheet '+i});
  await sc.execute('parameters.add',{name:'ChangeX',expression:s.x+' mm'});
  await sc.execute('solid.box',{length:'ChangeX',width:s.y,height:s.z,body_name:name});
  const positions=[];
  for(let n=0;n<s.holes;n++){
   const x=8+(n%3)*(s.x-16)/2;
   const y=8+Math.floor(n/3)*(s.y-16);
   positions.push([x,y]);
   await sc.execute('solid.hole',{position:[x,y,s.z],diameter:4,name:'Drill_'+n});
  }
  const first=await sc.execute('drawing.preview',{body:name});
  ok(first.status==='created','create view '+i);
  ok(first.drawing.geometry.cylinders.length===s.holes,'count each bore '+i);
  ok(first.drawing.dimensions[0].value_mm===s.x,'outer X '+i);
  ok(first.drawing.dimensions[1].value_mm===s.y,'outer Y '+i);
  ok((first.drawing.svg.match(/<circle /g)||[]).length===s.holes,'SVG holes '+i);
  const same=await sc.execute('drawing.preview',{body:name,previous:first.drawing});
  ok(same.status==='current','same model still current '+i);
  await sc.execute('parameters.change',{name:'ChangeX',expression:(s.x+12)+' mm'});
  const changed=await sc.execute('drawing.preview',{body:name,previous:first.drawing});
  ok(changed.status==='stale','changed plate '+i);
  ok(changed.proposed.dimensions[0].value_mm===s.x+12,'proposed plate X '+i);
  const accepted=await sc.execute('drawing.preview',{body:name,previous:first.drawing,accept:true});
  ok(accepted.status==='updated','accept changed plate '+i);
  const end=await sc.execute('drawing.preview',{body:name,previous:accepted.drawing});
  ok(end.status==='current','accepted current '+i);
  checked++;
 }
 return {test:'four plate sizes and 0,2,4,6 holes with drawing updates',plates:checked,bores:12,result:'passed'};
})()
