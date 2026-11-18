import json, os, pathlib, socket, struct, time
import pyatspi
report=pathlib.Path(os.environ["REPORT"])
port=int(os.environ["CONTROL_PORT"])
token=os.environ["SOLIDCRAFT_CONTROL_TOKEN"]
def call(method, params=None):
    with socket.create_connection(("127.0.0.1",port),timeout=180) as s:
        s.settimeout(180)
        s.sendall((json.dumps(dict(id=1,method=method,params=params or {},token=token))+"\n").encode())
        r=json.loads(s.makefile("rb").readline())
    with (report/"control.jsonl").open("a") as out:
        out.write(json.dumps(dict(method=method,params=params or {},reply=r))+"\n")
    if not r.get("ok"): raise RuntimeError(f"{method}: {r.get('error')}")
    return r.get("result")
def engine(cmd,params=None): return call("engine.execute",dict(command=cmd,params=params or {}))
def shot(name, size):
    p=report/(name+".png")
    call("ui.screenshot",dict(path=str(p)))
    with p.open("rb") as f: header=f.read(24)
    assert header[:8]==b"\x89PNG\r\n\x1a\n" and struct.unpack(">II",header[16:24])==size, (name,header)
def find(label):
    stack=[pyatspi.Registry.getDesktop(0)]
    found=[]
    while stack:
        n=stack.pop()
        try:
            if n.name==label and n.getState().contains(pyatspi.STATE_SHOWING):
                b=n.queryComponent().getExtents(pyatspi.WINDOW_COORDS)
                if b.width>0 and b.height>0: found.append((b.x+b.width/2,b.y+b.height/2))
            stack.extend(n[i] for i in range(n.childCount))
        except Exception: continue
    return found
def click(label):
    for _ in range(20):
        hits=find(label)
        if len(hits)==1:
            x,y=hits[0]; call("ui.click",dict(x=x,y=y)); time.sleep(.4); return
        time.sleep(.3)
    raise RuntimeError("Cannot uniquely locate visible native accessible widget: "+label)
for _ in range(180):
    try:
        if call("ui.inspect").get("viewport"): break
    except (OSError,RuntimeError): pass
    time.sleep(1)
else: raise RuntimeError("Native control channel never ready")
call("ui.agent_cursor",dict(show=False))
call("ui.home",dict(open=False))
call("ui.resize",dict(width=1280,height=720))
time.sleep(2)
recipe=json.load(open("examples/acceptance/drawing_m2.json"))["commands"]
assert len(recipe)==27
for step in recipe: result=engine(step["command"],step["params"])
view=result["view"]
drawing=engine("drawing.inspect")["id"]
added=engine("drawing.dimension.add",dict(view=view,kind=dict(radial=dict(edge=[25,20,20]))))
dim=added["dimension"]
assert added["status"]=="ok"
call("ui.workspace",dict(workspace="drawing"))
time.sleep(2)
shot("01_intact",(1280,720))
engine("drawing.close")
call("ui.workspace",dict(workspace="design"))
engine("solid.fillet",dict(body="Plate",radius=1,edges=[[25,20,20]]))
engine("drawing.open",dict(drawing=drawing))
call("ui.workspace",dict(workspace="drawing"))
time.sleep(2)
def inspect(): return engine("drawing.dimension.inspect",dict(dimension=dim))
before=inspect()
assert before["status"]=="lost" and before["candidates"]
shot("02_lost",(1280,720))
click("Repair "+dim)
shot("03_repair",(1280,720))
assert inspect()["status"]=="lost"
# Try only actual visible candidate controls. Any wrong repair is undone before the next attempt.
for i,pick in enumerate(before["candidates"]):
    if i:
        click("Repair "+dim)
        click("Candidate 1")
        click(json.dumps(pick,separators=(",",":"),ensure_ascii=False,sort_keys=True))
    click("Review repair")
    assert inspect()["status"]=="lost"
    shot("04_review_"+str(i),(1280,720))
    click("Confirm repair")
    after=inspect()
    if after.get("status")=="ok" and after.get("text")=="Ø10.00": break
    if after.get("status")!="lost":
        engine("edit.undo")
        assert inspect()["status"]=="lost"
    else: raise RuntimeError("Candidate confirmation did not repair reference")
else: raise RuntimeError("No candidate repaired the 10 mm bore")
shot("05_repaired",(1280,720))
engine("edit.undo")
assert inspect()["status"]=="lost"
shot("06_undo",(1280,720))
call("ui.resize",dict(width=1024,height=640))
time.sleep(2)
shot("07_small_window",(1024,640))
print("Native GUI repair, review, confirmation and undo passed")
