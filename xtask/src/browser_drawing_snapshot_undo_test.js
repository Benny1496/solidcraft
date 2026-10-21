(async () => {
 const app=window.solidcraft, assert=(x,m)=>{if(!x)throw Error(m)};
 const KEY='org.solidcraft.experimental_drawing';
 await app.execute('file.new',{name:'Drawing stored snapshot undo/redo'});
 await app.execute('solid.box',{length:90,width:40,height:9,body_name:'Plate'});
 const first=await app.execute('drawing.preview',{body:'Plate'});
 assert(first.status==='created','preview created');
 const initial=first.drawing;
 initial.label_positions.overall_x=[40,-17];
 await app.execute('document.metadata.set',{key:KEY,value:initial});
 let snap=(await app.execute('document.metadata',{})).metadata[KEY];
 assert(snap.dimensions[0].value_mm===90,'saved snapshot');
 const same=await app.execute('document.metadata.set',{key:KEY,value:initial});
 assert(same.changed.length===0,'repeated identical snapshot is a no-op');
 await app.execute('edit.undo');
 assert((await app.execute('document.metadata',{})).metadata[KEY]===undefined,
   'Undo removes original saved snapshot, not some later model edit');
 await app.execute('edit.redo');
 snap=(await app.execute('document.metadata',{})).metadata[KEY];
 assert(snap.dimensions[0].value_mm===90,'Redo restores stored drawing');
 assert(snap.label_positions.overall_x[1]===-17,'Redo restores label placement');
 const rev=await app.execute('drawing.preview',{body:'Plate',previous:snap});
 assert(rev.status==='current','Undo/Redo returned to current drawing');
 await app.execute('document.metadata.remove',{key:KEY});
 assert((await app.execute('document.metadata',{})).metadata[KEY]===undefined,
   'can remove experimental sheet without changing 3D body');
 await app.execute('edit.undo');
 snap=(await app.execute('document.metadata',{})).metadata[KEY];
 assert(snap.dimensions[0].value_mm===90,'Undo removal restores original drawing');
 const model=await app.inspect({measure:true});
 assert(model.bodies.length===1&&model.timeline.length===1,
   'snapshot history does not remove or create CAD features');
 return {test:'drawing metadata save/no-op/undo/redo/remove/restore',result:'passed'};
})()
