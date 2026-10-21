(async () => {
 const app=window.solidcraft, ok=(v,m)=>{if(!v)throw Error(m)};
 await app.execute('file.new',{name:'Drawing validation and refused operations'});
 await app.execute('solid.box',{length:100,width:40,height:8,body_name:'Plate'});
 const start=await app.execute('drawing.preview',{body:'Plate'});
 ok(start.status==='created','base drawing created');
 const old=start.drawing, cases=[];
 const fail=async(name,arg)=>{
   let rejected=false;
   try{await app.execute('drawing.preview',arg)}catch(e){rejected=true}
   ok(rejected,'must reject '+name);
   cases.push(name);
 };
 await fail('nonexistent body',{body:'Body-that-does-not-exist'});
 await fail('missing body',{});
 await fail('accept without previous',{body:'Plate',accept:true});
 await fail('invalid accept type',{body:'Plate',accept:'yes'});
 await fail('invalid previous format',{body:'Plate',previous:{...old,format:'untrusted/3'}});
 await fail('previous of a different body',{body:'Plate',previous:{...old,body:'Different'}});
 await fail('missing source signature',{body:'Plate',previous:{format:old.format,body:'Plate'}});
 await fail('incomplete measured source',{body:'Plate',previous:{...old,source:{size_mm:[100,40,8]}}});
 await fail('invalid numeric measurement',{body:'Plate',previous:{...old,source:{...old.source,size_mm:[100,'invalid',8]}}});
 await fail('invalid label coordinate',{body:'Plate',previous:{...old,label_positions:{...old.label_positions,overall_x:['invalid',-15]}}});
 await fail('out-of-range label coordinate',{body:'Plate',previous:{...old,label_positions:{...old.label_positions,overall_x:[1e20,-15]}}});
 const tooMany={};for(let i=0;i<130;i++)tooMany['invalid_'+i]=[i,0];
 await fail('too many label positions',{body:'Plate',previous:{...old,label_positions:tooMany}});
 const after=await app.execute('drawing.preview',{body:'Plate',previous:old});
 ok(after.status==='current','invalid operations did not damage the valid drawing');
 const model=await app.inspect({measure:true});
 ok(model.bodies.length===1&&model.timeline.length===1,'no CAD features were accidentally added');
 return {test:'12 rejected drawing inputs are non-mutating',rejected:cases,result:'passed'};
})()
