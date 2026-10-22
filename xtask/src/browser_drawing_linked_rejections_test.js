(async () => {
 const app=window.solidcraft,check=(x,m)=>{if(!x)throw Error(m)};
 await app.execute('file.new',{name:'Linked dimension validation'});
 await app.execute('solid.box',{length:100,width:50,height:8,body_name:'Plate'});
 await app.execute('solid.hole',{name:'Drill1',position:[15,15,8],diameter:6});
 let first=(await app.execute('drawing.preview',{body:'Plate'})).drawing;
 const good=first.geometry.cylinder_dimensions.find(x=>x.eligible_face_name);
 check(good,'verified cylinder face expected');
 let count=0;
 const reject=async(p,name)=>{let did=false;try{await app.execute('drawing.annotation.add',p)}catch(e){did=true}
   check(did,'reject '+name);count++};
 const base={body:'Plate',previous:first,face_name:good.eligible_face_name,kind:'diameter'};
 await reject({...base,face_name:'not-a-real-certified-face'},'unknown face');
 await reject({...base,kind:'radius'},'unsupported kind');
 await reject({...base,kind:'angle'},'not-yet-supported angle');
 await reject({...base,position_mm:['invalid',12]},'text in position');
 await reject({...base,position_mm:[1e20,0]},'out-of-range position');
 await reject({...base,previous:{...first,body:'Wrong'}},'wrong-body snapshot');
 const yes=await app.execute('drawing.annotation.add',base);
 first=yes.drawing;
 await reject({...base,previous:first},'same measure repeated');
 await app.execute('solid.hole',{name:'Drill2',position:[70,35,8],diameter:8});
 await reject({...base,previous:first,kind:'x_from_left'},'stale drawing');
 const current=await app.execute('drawing.preview',{body:'Plate',previous:first});
 check(current.status==='stale','stale drawing unchanged after rejections');
 check(first.annotations.length===1,'only one annotation saved in original snapshot');
 return {test:'eight invalid linked annotation commands refused without modifying old drawing',rejections:count,result:'passed'};
})()
