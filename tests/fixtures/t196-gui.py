import json, os, pathlib, socket, struct, time
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
def click(label):
    for attempt in range(35):
        handles=call("ui.handles",dict(prefix="t194:"))
        hits=[h for h in handles if h["name"]==label]
        if len(hits)==1:
            h=hits[0]
            x,y=h["x"],h["y"]
            if not (0 <= x < 1280): raise RuntimeError("Target outside horizontal window: "+str(h))
            if not (180 <= y <= 670):
                print("SCROLL to reveal",label,h,flush=True)
                call("ui.scroll",dict(x=x,y=490,delta=-140 if y>670 else 140,unit="point"))
                time.sleep(.5)
                continue
            print("CLICK",label,h,flush=True)
            call("ui.click",dict(x=x,y=y))
            time.sleep(.5)
            print("AFTER",label,call("ui.handles",dict(prefix="t194:")),flush=True)
            return
        # A clipped control may not be registered yet; retain diagnostics.
        time.sleep(.3)
    shot("failure_missing_or_clipped_handle",(1280,720))
    raise RuntimeError("Missing or inaccessible native handle: "+label)
for _ in range(180):
    try:
        os.kill(int(os.environ["APP_PID"]),0)
    except ProcessLookupError:
        raise RuntimeError("Native application exited: "+(report/"app.txt").read_text(errors="replace"))
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
print("BEFORE",json.dumps(before,ensure_ascii=False),flush=True)
assert before["status"]=="lost" and before["candidates"]
shot("02_lost",(1280,720))
click("t194:repair:"+dim)
shot("03_repair",(1280,720))
assert inspect()["status"]=="lost"
# Try only actual visible candidate controls. Any wrong repair is undone before the next attempt.
for i,pick in enumerate(before["candidates"]):
    print("TRY CANDIDATE",i,json.dumps(pick,ensure_ascii=False),flush=True)
    if i:
        click("t194:repair:"+dim)
        click("t194:combo")
        click("t194:candidate:"+str(i))
    click("t194:review")
    assert any(h["name"]=="t194:confirm" for h in call("ui.handles",dict(prefix="t194:"))), "Review click did not expose confirmation"
    print("Before inspect after review",call("ui.handles",dict(prefix="t194:")),flush=True)
    assert inspect()["status"]=="lost"
    print("After inspect after review",call("ui.handles",dict(prefix="t194:")),flush=True)
    shot("04_review_"+str(i),(1280,720))
    click("t194:confirm")
    after=inspect()
    print("AFTER CONFIRM",i,json.dumps(after,ensure_ascii=False),flush=True)
    shot("candidate_result_"+str(i),(1280,720))
    if after.get("status")=="ok" and after.get("text")=="Ø10.00": break
    # A mutation can leave a dimension lost; undo if its stored record changed.
    if after.get("record")!=before.get("record") or after.get("status")!="lost":
        engine("edit.undo")
        restored=inspect()
        assert restored["status"]=="lost" and restored["record"]==before["record"], restored
    else:
        print("Candidate rejected without record change; trying next",flush=True)
else: raise RuntimeError("No candidate repaired the 10 mm bore")
shot("05_repaired",(1280,720))
engine("edit.undo")
assert inspect()["status"]=="lost"
shot("06_undo",(1280,720))
call("ui.resize",dict(width=1024,height=640))
time.sleep(2)
shot("07_small_window",(1024,640))
print("Native GUI repair, review, confirmation and undo passed")
